use stagemaster_execution_client::{Client, MediaAction, MediaCompletion, View};
use std::time::{Duration, Instant};

pub async fn wait(client: &mut Client, predicate: impl Fn(&View) -> bool) -> View {
    let end = Instant::now() + Duration::from_secs(6);
    loop {
        let view = match client.refresh().await {
            Ok(view) => view,
            // Observation uses a non-blocking slot. Retry only its explicit busy response,
            // under the original deadline; never resend a control command.
            Err(error) if error == "后台请求未成功（503），请核对连接与原回执" =>
            {
                assert!(Instant::now() < end, "{error}");
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }
            Err(error) => panic!("{error}"),
        };
        if predicate(&view) {
            return view;
        }
        assert!(Instant::now() < end, "{view:?}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
pub async fn settled(client: &mut Client) -> View {
    wait(client, |v| !v.pending).await
}
pub async fn apply(client: &mut Client, action: MediaAction) -> View {
    let view = settled(client).await;
    let state = &view.observation.snapshot.as_ref().unwrap().state;
    client
        .apply_media(
            &view.host_id,
            &state.revision,
            &state.media[0].id,
            &state.media[0].generation,
            action,
        )
        .await
        .unwrap();
    let accepted = settled(client).await;
    let outcome = accepted.record.as_ref().unwrap().outcome.as_ref().unwrap();
    assert_eq!(outcome.kind, "accepted");
    let expected = outcome.state.as_ref().unwrap().media[0]
        .control
        .as_ref()
        .unwrap();
    assert_eq!(expected.status, MediaCompletion::Pending);
    wait(client, |v| {
        v.observation.snapshot.as_ref().is_some_and(|s| {
            s.state.media[0].control.as_ref().is_some_and(|c| {
                c.request == expected.request && c.status == MediaCompletion::Applied
            })
        })
    })
    .await
}
