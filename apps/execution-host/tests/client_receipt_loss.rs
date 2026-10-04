mod support;
use stagemaster_execution_client::{Action, Client, MediaAction, MediaCompletion};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use support::client_proxy::install as install_proxy;
use support::*;
#[test]
fn admitted_command_with_lost_reply_recovers_only_through_original_receipt() {
    runtime().block_on(async { run(Harness::start_group(), false).await });
}
#[cfg(feature = "audio")]
#[test]
fn accepted_audio_with_lost_reply_is_not_replayed_or_confused_with_actual_completion() {
    runtime().block_on(async {
        run(Harness::prepared(|p| Some(support::audio::write(p))), true).await;
    });
}
async fn run(mut h: Harness, audio: bool) {
    let discovery_path = h.directory.path().join("run/discovery.json");
    let (proxy, task) = install_proxy(&h, &discovery_path).await;
    let mut control = Client::open(&discovery_path).await.unwrap();
    control.acquire(false).await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(6);
    while control.refresh().await.unwrap().pending {
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let before = control.view();
    let source = &before.catalog.sources[0];
    let revision = &before.observation.snapshot.as_ref().unwrap().state.revision;
    let older = ok(h.get("/state")).await;
    *proxy.old_observation.lock().unwrap() = Some(older);
    proxy.corrupt.store(true, Ordering::SeqCst);
    let result = if audio {
        let state = &before.observation.snapshot.as_ref().unwrap().state;
        control
            .apply_media(
                &before.host_id,
                revision,
                &state.media[0].id,
                &state.media[0].generation,
                MediaAction::Play {},
            )
            .await
    } else {
        control
            .apply(
                &before.host_id,
                revision,
                &source.id,
                Action::Start {
                    step: source.steps[0].id.clone(),
                },
            )
            .await
    };
    assert!(result.is_err());
    assert!(control.view().pending);
    assert!(control.release().await.is_err());
    let pending = control.view().record.unwrap().serial;
    loop {
        let view = control.refresh().await.unwrap();
        if !view.pending {
            assert_eq!(view.record.as_ref().unwrap().serial, pending);
            if audio {
                let state = view.observation.snapshot.unwrap().state;
                assert_eq!(
                    state.media[0].control.as_ref().unwrap().status,
                    MediaCompletion::Pending
                );
                assert_eq!(
                    view.record.as_ref().unwrap().outcome.as_ref().unwrap().kind,
                    "accepted"
                );
            } else {
                assert_eq!(
                    view.observation.snapshot.unwrap().state.sources[0]
                        .status
                        .as_deref(),
                    Some("Running")
                );
            }
            break;
        }
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2); // acquire + start, no re-send.
    if audio {
        *proxy.old_observation.lock().unwrap() = None;
        loop {
            let view = control.refresh().await.unwrap();
            if view.observation.snapshot.unwrap().state.media[0]
                .control
                .as_ref()
                .is_some_and(|c| c.status == MediaCompletion::Applied)
            {
                break;
            }
            assert!(Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
    }
    task.abort();
    h.close().await;
}
