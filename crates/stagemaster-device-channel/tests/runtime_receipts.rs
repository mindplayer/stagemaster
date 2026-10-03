mod runtime_support;
mod support;
use runtime_support::{connected, reply, rights};
use stagemaster_device_session::Kind;
use stagemaster_runtime_protocol::{Body, Failure, Operation, Request, Response};
use std::time::Duration;

#[tokio::test]
async fn retry_keeps_full_request_and_original_receipt_and_late_duplicate_cannot_complete_next() {
    let (_dir, mut client, mut server) = connected(rights()).await;
    let request = client
        .send(
            Operation::Acquire {
                duration_ms: 1000,
                takeover: false,
            },
            server.device.state().revision,
        )
        .await
        .unwrap();
    assert!(client.send(Operation::Status, 0).await.is_err());
    let (_, bytes) = server.peer.receive().await.unwrap();
    let lost = server.process(&bytes); // Worker finished; deliberately do not seal/send a reply yet.
    let owner = server.device.state().owner;
    assert_eq!(client.retry_pending().await.unwrap(), request);
    let (_, retried) = server.peer.receive().await.unwrap();
    assert_eq!(Request::decode(&retried).unwrap(), request);
    let repeat = server.process(&retried);
    assert_eq!(repeat, lost);
    assert_eq!(server.device.state().owner, owner);
    server
        .peer
        .send(Kind::Message, repeat.bytes())
        .await
        .unwrap();
    assert_eq!(
        reply(&mut client).await,
        Response::decode(lost.bytes()).unwrap()
    );
    // A queued old exact duplicate is ignored, never used as the next command's result.
    let next = client.send(Operation::Status, 0).await.unwrap();
    server.peer.send(Kind::Message, lost.bytes()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(client.receive().unwrap().is_none());
    assert_eq!(client.pending(), Some(next));
    let (_, bytes) = server.peer.receive().await.unwrap();
    let current = server.process(&bytes);
    server
        .peer
        .send(Kind::Message, current.bytes())
        .await
        .unwrap();
    assert_eq!(reply(&mut client).await.request, next);
}

#[tokio::test]
async fn wrong_reply_context_closes_channel_but_preserves_uncertain_request() {
    for variant in 0..3 {
        let (_dir, mut client, mut server) = connected(rights()).await;
        let expected = client.send(Operation::Status, 0).await.unwrap();
        let (_, bytes) = server.peer.receive().await.unwrap();
        let original = server.process(&bytes);
        let mut changed = Response::decode(original.bytes()).unwrap();
        match variant {
            0 => changed.request.id += 1,
            1 => changed.request.session[0] ^= 1,
            _ => changed.request.expected_revision += 1,
        }
        server
            .peer
            .send(Kind::Message, changed.encode().unwrap().bytes())
            .await
            .unwrap();
        let result = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                match client.receive() {
                    Ok(None) => tokio::task::yield_now().await,
                    other => break other,
                }
            }
        })
        .await
        .unwrap();
        assert!(result.is_err());
        assert!(client.peer().is_none());
        assert_eq!(client.pending(), Some(expected));
        assert!(client.last_response().is_none());
        assert!(client.retry_pending().await.is_err());
    }
}

#[tokio::test]
async fn business_failure_is_a_confirmed_reply_without_automatic_retry_or_disconnect() {
    let (_dir, mut client, mut server) = connected(rights()).await;
    let expected = client
        .send(Operation::Release, server.device.state().revision)
        .await
        .unwrap();
    let (_, bytes) = server.peer.receive().await.unwrap();
    let frame = server.process(&bytes);
    server
        .peer
        .send(Kind::Message, frame.bytes())
        .await
        .unwrap();
    let response = reply(&mut client).await;
    assert_eq!(response.request, expected);
    assert!(matches!(
        response.body,
        Body::State {
            result: Err(Failure::Runtime(stagemaster_runtime::Code::Lease)),
            ..
        }
    ));
    assert!(client.pending().is_none());
    assert!(client.peer().is_some());
    assert_eq!(client.last_response(), Some(&response));
    assert!(client.retry_pending().await.is_err());
}
