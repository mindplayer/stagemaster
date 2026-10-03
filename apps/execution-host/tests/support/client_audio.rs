use stagemaster_execution_client::{Client, MediaAction, MediaCompletion, View};
use std::time::{Duration, Instant};

pub async fn wait(client: &mut Client, predicate: impl Fn(&View) -> bool) -> View {
    let end = Instant::now() + Duration::from_secs(6);
    loop {
        let view = client.refresh().await.unwrap();
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
