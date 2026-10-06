#![cfg(feature = "audio")]
mod support;
use serde_json::json;
use stagemaster_execution_client::{Client, MediaAction, View};
use support::{client_audio::*, client_observation::confirmed_control, *};

async fn refusal(
    client: &mut Client,
    generation: &str,
    action: MediaAction,
    code: &str,
    message: &str,
) {
    let action_label = format!("{action:?}");
    let before = settled(client).await;
    let state = &before.observation.snapshot.as_ref().unwrap().state;
    let original = json!({"audio":state.audio,"media":state.media});
    let previous_serial = before
        .record
        .as_ref()
        .unwrap()
        .serial
        .parse::<u64>()
        .unwrap();
    let submitted = client
        .apply_media(
            &before.host_id,
            &state.revision,
            &state.media[0].id,
            generation,
            action,
        )
        .await;
    let completed = confirmed_control(client, submitted, "rejected").await;
    assert!(completed.controlling);
    assert!(!completed.pending);
    let record = completed.record.as_ref().unwrap();
    assert_eq!(record.serial.parse::<u64>().unwrap(), previous_serial + 1);
    let outcome = record.outcome.as_ref().unwrap();
    assert_eq!(
        outcome.code.as_deref(),
        Some(code),
        "action={action_label}, generation={generation}, original={original}, evidence={:?}",
        completed.media_operation
    );
    assert_eq!(outcome.message.as_deref(), Some(message));
    assert!(outcome.state.is_none());
    let current = &completed.observation.snapshot.as_ref().unwrap().state;
    assert_eq!(
        json!({"audio":current.audio,"media":current.media}),
        original
    );
    // Resolving this original receipt does not submit another action or consume another serial.
    let refreshed = settled(client).await;
    assert_eq!(refreshed.record.as_ref().unwrap().serial, record.serial);
}
fn exit(view: &View, requested: bool) -> MediaAction {
    let audio = view
        .observation
        .snapshot
        .as_ref()
        .unwrap()
        .state
        .audio
        .as_ref()
        .unwrap();
    let current = audio.loop_state.as_ref().unwrap();
    MediaAction::ExitLoop {
        instance: audio.instance.clone().unwrap(),
        region: current.region,
        pass: current.pass.clone(),
        requested,
    }
}

#[test]
fn original_typed_client_preserves_target_refusal_and_can_explicitly_control_the_current_loop() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let mut client = Client::open(&h.directory.path().join("run/discovery.json"))
            .await
            .unwrap();
        let acquired = client.acquire(false).await;
        confirmed_control(&mut client, acquired, "acquired").await;
        let paused = apply(
            &mut client,
            MediaAction::Seek {
                position_ms: 2500,
                playing: false,
            },
        )
        .await;
        let state = &paused.observation.snapshot.as_ref().unwrap().state;
        let mut expired_target = exit(&paused, true);
        if let MediaAction::ExitLoop { pass, .. } = &mut expired_target {
            *pass = "2".into();
        }
        refusal(
            &mut client,
            &state.media[0].generation,
            expired_target,
            "loopTargetChanged",
            "循环播放目标已变化，请确认当前区段和遍次后重新操作",
        )
        .await;
        refusal(
            &mut client,
            "0",
            exit(&paused, true),
            "mediaTargetChanged",
            "音乐运行目标已变化，请核对当前状态后重新操作",
        )
        .await;
        let exiting = apply(&mut client, exit(&paused, true)).await;
        assert!(
            exiting
                .observation
                .snapshot
                .unwrap()
                .state
                .audio
                .unwrap()
                .loop_state
                .unwrap()
                .exit_requested
        );
        let cancelled = apply(&mut client, exit(&paused, false)).await;
        let actual = cancelled.observation.snapshot.unwrap().state.audio.unwrap();
        assert!(!actual.loop_state.unwrap().exit_requested);
        assert_eq!(actual.instance, state.audio.as_ref().unwrap().instance);
        assert_eq!(
            actual.position_ms,
            state.audio.as_ref().unwrap().position_ms
        );
        h.close().await;
    });
}
