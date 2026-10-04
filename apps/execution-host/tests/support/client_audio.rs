pub use super::client_observation::{settled, wait};
use stagemaster_execution_client::{Client, MediaAction, MediaCompletion, View};
pub async fn apply(client: &mut Client, action: MediaAction) -> View {
    let view = settled(client).await;
    let state = &view.observation.snapshot.as_ref().unwrap().state;
    let action_label = format!("{action:?}");
    let submitted = client
        .apply_media(
            &view.host_id,
            &state.revision,
            &state.media[0].id,
            &state.media[0].generation,
            action,
        )
        .await;
    if let Err(error) = submitted {
        assert_eq!(error, super::client_observation::BUSY, "{action_label}");
        // Only resolve this serial. A failed admission cannot produce an accepted receipt.
    }
    let serial = client.view().record.unwrap().serial;
    let accepted = settled(client).await;
    let record = accepted.record.as_ref().unwrap();
    assert_eq!(record.serial, serial);
    let outcome = record.outcome.as_ref().unwrap();
    assert_eq!(
        outcome.kind, "accepted",
        "action={action_label}, revision={}, generation={}, serial={serial}, outcome={outcome:?}",
        state.revision, state.media[0].generation
    );
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
