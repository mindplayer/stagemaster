#![cfg(feature = "audio")]
mod support;
use serde_json::json;
use stagemaster_execution_client::{Client, MediaAction, MediaCompletion, Reader};
use std::{fs, sync::atomic::Ordering};
use support::{
    client_observation::{BUSY, settled, wait},
    client_proxy::install,
    *,
};

#[test]
fn admitted_media_survives_a_busy_observation_without_replay_or_invented_completion() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        let mut reader = Reader::open(&path).await.unwrap();
        client.acquire(false).await.unwrap();
        let view = settled(&mut client).await;
        let state = view.observation.snapshot.unwrap().state;
        let old = json!({"phase":"running", "fault":null, "snapshot":h.snapshot().await});
        *proxy.old_observation.lock().unwrap() = Some(old);
        proxy.reject_observations.store(true, Ordering::SeqCst);
        let error = client.apply_media(&view.host_id, &state.revision, &state.media[0].id, &state.media[0].generation,
            MediaAction::Seek { position_ms: 2500, playing: false }).await.unwrap_err();
        assert_eq!(error, BUSY);
        assert_eq!(reader.sample().await.unwrap_err(), BUSY);
        let serial = client.view().record.unwrap().serial;
        assert_eq!(serial, "2");
        assert!(client.view().pending);
        proxy.reject_observations.store(false, Ordering::SeqCst);
        let accepted = settled(&mut client).await;
        let record = accepted.record.as_ref().unwrap();
        assert_eq!(record.serial, serial);
        let outcome = record.outcome.as_ref().unwrap();
        assert_eq!(outcome.kind, "accepted", "{outcome:?}");
        let control = outcome.state.as_ref().unwrap().media[0].control.as_ref().unwrap();
        assert_eq!(control.status, MediaCompletion::Pending);
        let request = control.request.clone();
        group::until(&h, |v| v["state"]["media"][0]["control"]["request"] == request && v["state"]["media"][0]["control"]["status"] == "applied").await;
        // Receipt remains immutable and the frozen old cycle cannot invent provider completion.
        let frozen = client.refresh().await.unwrap();
        assert_eq!(frozen.observation.snapshot.unwrap().state.media[0].control.as_ref().unwrap().status, MediaCompletion::Pending);
        *proxy.old_observation.lock().unwrap() = None;
        let completed = wait(&mut client, |v| v.observation.snapshot.as_ref().unwrap().state.media[0].control.as_ref()
            .is_some_and(|c| c.request == request && c.status == MediaCompletion::Applied)).await;
        assert_eq!(completed.record.unwrap().serial, serial);
        assert_eq!(reader.sample().await.unwrap().slots[1], 40);
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 2);
        eprintln!("EXEC-016 post-admission: serial={serial}, accepted pending request={request}, same request applied, attempts=2 forwarded=2; readonly 503 not completion");
        task.abort();h.close().await;
    });
}

#[test]
fn accepted_preparation_can_fail_and_must_not_be_reported_as_applied() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(audio::write(p)));
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        client.acquire(false).await.unwrap();
        client_audio::settled(&mut client).await;
        let before = client_audio::apply(&mut client, MediaAction::Play {}).await;
        let state = before.observation.snapshot.unwrap().state;
        let copy = fs::read_dir(h.directory.path().join("run/store/media")).unwrap().next().unwrap().unwrap().path();
        fs::write(copy, b"fault injection into owned temporary test copy").unwrap();
        client.apply_media(&before.host_id, &state.revision, &state.media[0].id, &state.media[0].generation,
            MediaAction::Seek { position_ms: 3000, playing: false }).await.unwrap();
        let accepted = settled(&mut client).await;
        let record = accepted.record.as_ref().unwrap();
        assert_eq!(record.serial, "3");
        let outcome = record.outcome.as_ref().unwrap();
        assert_eq!(outcome.kind, "accepted", "{outcome:?}");
        let control = outcome.state.as_ref().unwrap().media[0].control.as_ref().unwrap();
        assert_eq!(control.status, MediaCompletion::Pending);
        let request = control.request.clone();
        let failed = wait(&mut client, |v| v.observation.snapshot.as_ref().unwrap().state.media[0].control.as_ref()
            .is_some_and(|c| c.request == request && c.status == MediaCompletion::Failed)).await;
        let actual = failed.observation.snapshot.unwrap().state;
        assert!(actual.media[0].control.as_ref().unwrap().problem.is_some());
        assert_eq!(failed.record.unwrap().serial, "3");
        assert!(!actual.fault);
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 3);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 3);
        eprintln!("EXEC-016 provider-failure: serial=3 accepted pending request={request}, same request failed, attempts=3 forwarded=3, no fake Applied");
        task.abort();h.close().await;
    });
}
