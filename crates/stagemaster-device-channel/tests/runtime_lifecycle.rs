mod runtime_support;
mod support;
use runtime_support::{Server, connected, description, installed, reply, rights};
use stagemaster_device_auth::application::Role;
use stagemaster_device_channel::{
    Channel, Error, StreamRecords,
    runtime::{REQUEST_TIMEOUT, RuntimeClient},
};
use stagemaster_device_session::Kind;
use stagemaster_runtime_protocol::{Operation, Request};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};
use support::{config, faults::FaultIo, temporary};
use tokio::{io::duplex, time::Instant};

#[tokio::test(start_paused = true)]
async fn keepalive_and_retries_do_not_extend_pending_request_deadline() {
    let (_dir, mut client, mut server) = connected(rights()).await;
    let request = client.send(Operation::Status, 0).await.unwrap();
    let started = Instant::now();
    let (_, bytes) = server.peer.receive().await.unwrap();
    server.process(&bytes); // No reply sent; the secure heartbeat remains healthy.
    for i in 0..14 {
        tokio::time::advance(Duration::from_secs(2)).await;
        if i == 9 {
            assert_eq!(client.retry_pending().await.unwrap(), request);
            let (_, bytes) = server.peer.receive().await.unwrap();
            assert_eq!(Request::decode(&bytes).unwrap(), request);
            server.process(&bytes);
        }
        let (result, ()) = tokio::join!(client.heartbeat(), async {
            assert_eq!(server.peer.receive().await.unwrap().0, Kind::Heartbeat);
            server.peer.send(Kind::HeartbeatReply, &[]).await.unwrap();
        });
        result.unwrap();
        assert!(client.peer().is_some());
    }
    tokio::time::advance(REQUEST_TIMEOUT.saturating_sub(started.elapsed())).await;
    assert!(matches!(client.receive(), Err(Error::Timeout)));
    assert!(client.peer().is_none());
    assert_eq!(client.pending(), Some(request));
    assert!(client.retry_pending().await.is_err());
}

#[tokio::test]
async fn cancelled_or_failed_send_retains_uncertainty_and_cannot_reuse_session() {
    for mode in [1, 2] {
        let directory = temporary();
        let device = installed(directory.path());
        let (client, server) = duplex(4096);
        let fault = Arc::new(AtomicU8::new(0));
        let io = FaultIo {
            inner: StreamRecords::new(client),
            mode: fault.clone(),
        };
        let config = config(Role::Controller);
        let (channel, _server) = tokio::join!(
            Channel::prepare_runtime(io, description(7), &config, rights()),
            Server::accept(
                StreamRecords::new(server),
                description(7),
                device,
                Instant::now(),
                rights()
            ),
        );
        let mut client = RuntimeClient::new(channel.unwrap()).unwrap();
        fault.store(mode, Ordering::Release);
        let result =
            tokio::time::timeout(Duration::from_millis(20), client.send(Operation::Status, 0))
                .await;
        if mode == 1 {
            assert!(result.is_err());
        } else {
            assert!(result.unwrap().is_err());
        }
        let pending = client.pending().unwrap();
        assert_eq!(pending.id, 1);
        fault.store(0, Ordering::Release);
        assert!(client.peer().is_none());
        assert!(client.retry_pending().await.is_err());
        assert_eq!(client.pending(), Some(pending));
    }
}

#[tokio::test]
async fn identical_application_replies_during_keepalive_share_one_slot() {
    let (_dir, mut client, mut server) = connected(rights()).await;
    let request = client.send(Operation::Status, 0).await.unwrap();
    let (_, bytes) = server.peer.receive().await.unwrap();
    let frame = server.process(&bytes);
    let (result, ()) = tokio::join!(client.heartbeat(), async {
        assert_eq!(server.peer.receive().await.unwrap().0, Kind::Heartbeat);
        server
            .peer
            .send(Kind::Message, frame.bytes())
            .await
            .unwrap();
        server
            .peer
            .send(Kind::Message, frame.bytes())
            .await
            .unwrap();
        server.peer.send(Kind::HeartbeatReply, &[]).await.unwrap();
    });
    result.unwrap();
    assert_eq!(reply(&mut client).await.request, request);
    assert!(client.pending().is_none());
}

#[tokio::test(start_paused = true)]
async fn duplicate_messages_cannot_keep_an_unanswered_heartbeat_alive_forever() {
    let (_dir, mut client, mut server) = connected(rights()).await;
    client.send(Operation::Status, 0).await.unwrap();
    let (_, bytes) = server.peer.receive().await.unwrap();
    let frame = server.process(&bytes);
    let began = Instant::now();
    let (result, ()) = tokio::join!(client.heartbeat(), async {
        assert_eq!(server.peer.receive().await.unwrap().0, Kind::Heartbeat);
        for _ in 0..4 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            server
                .peer
                .send(Kind::Message, frame.bytes())
                .await
                .unwrap();
        }
    });
    assert!(matches!(result, Err(Error::Timeout)));
    assert_eq!(began.elapsed(), Duration::from_secs(5));
    assert!(client.peer().is_none());
    assert!(client.pending().is_some());
}
