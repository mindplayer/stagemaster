mod support;
use stagemaster_execution_client::{Action, Client, View};
use std::time::{Duration, Instant};
use support::*;

async fn settled(client: &mut Client) -> View {
    let end = Instant::now() + Duration::from_secs(6);
    loop {
        let view = client.refresh().await.unwrap();
        if !view.pending {
            return view;
        }
        assert!(Instant::now() < end);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
fn revision(view: &View) -> String {
    view.observation
        .snapshot
        .as_ref()
        .unwrap()
        .state
        .revision
        .clone()
}
#[test]
fn desktop_client_observes_controls_and_reconnects_without_automatic_takeover() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let mut first = Client::open(&path).await.unwrap();
        assert!(!first.view().controlling);
        assert!(first.view().session_id.is_none());
        first.acquire(false).await.unwrap();
        let view = settled(&mut first).await;
        assert!(view.controlling);
        let source = &view.catalog.sources[0];
        first
            .apply(
                &view.host_id,
                &revision(&view),
                &source.id,
                Action::Start {
                    step: source.steps[0].id.clone(),
                },
            )
            .await
            .unwrap();
        let started = settled(&mut first).await;
        assert_eq!(
            started.observation.snapshot.as_ref().unwrap().state.sources[0]
                .status
                .as_deref(),
            Some("Running")
        );
        assert!(
            first
                .apply(
                    "wrong-host",
                    &revision(&started),
                    &source.id,
                    Action::Stop {}
                )
                .await
                .is_err()
        );
        assert!(!first.view().pending);
        let mut second = Client::open(&path).await.unwrap();
        assert!(!second.view().controlling);
        second.acquire(false).await.unwrap();
        assert_eq!(
            settled(&mut second)
                .await
                .record
                .unwrap()
                .outcome
                .unwrap()
                .kind,
            "rejected"
        );
        second.acquire(true).await.unwrap();
        let owned = settled(&mut second).await;
        assert!(owned.controlling);
        // An operation from the previous owner's session is rejected, not translated to new authority.
        first
            .apply(
                &view.host_id,
                &revision(&owned),
                &source.id,
                Action::Stop {},
            )
            .await
            .unwrap();
        assert_eq!(
            settled(&mut first)
                .await
                .record
                .unwrap()
                .outcome
                .unwrap()
                .kind,
            "rejected"
        );
        let before = revision(&owned);
        second
            .apply(&owned.host_id, &before, &source.id, Action::Pause {})
            .await
            .unwrap();
        let paused = settled(&mut second).await;
        second
            .apply(&owned.host_id, &before, &source.id, Action::Stop {})
            .await
            .unwrap();
        let refused = settled(&mut second).await;
        assert_eq!(refused.record.unwrap().outcome.unwrap().kind, "rejected");
        assert_eq!(revision(&paused), revision(&settled(&mut second).await));
        second.release().await.unwrap();
        assert!(!settled(&mut second).await.controlling);
        h.close().await;
    });
}

#[test]
fn unresolved_command_is_retained_and_never_resent_by_polling() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let mut client = Client::open(&path).await.unwrap();
        client.acquire(false).await.unwrap();
        let view = settled(&mut client).await;
        let source = &view.catalog.sources[0];
        // The protocol rejects malformed step identity before admission. No receipt exists:
        // this exercises the same durable uncertainty path as a lost HTTP reply.
        assert!(
            client
                .apply(
                    &view.host_id,
                    &revision(&view),
                    &source.id,
                    Action::Start { step: "bad".into() }
                )
                .await
                .is_err()
        );
        let pending = client.view().record.unwrap().serial;
        assert!(client.refresh().await.is_err());
        assert!(client.view().pending);
        assert!(
            client
                .apply(&view.host_id, &revision(&view), &source.id, Action::Stop {})
                .await
                .is_err()
        );
        assert_eq!(client.view().record.unwrap().serial, pending);
        let state = h.state().await;
        assert_eq!(state["revision"], revision(&view));
        h.close().await;
    });
}

#[test]
fn client_refuses_public_credentials_remote_endpoints_and_wrong_protocol() {
    runtime().block_on(async {
        use std::{fs, os::unix::fs::PermissionsExt};
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let original = fs::read(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(Client::open(&path).await.is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        for url in [
            "http://localhost:80/v2/a",
            "http://example.com:80/v2/a",
            "http://127.0.0.1:1/v2/a?token=x",
        ] {
            let mut data = h.discovery.clone();
            data["url"] = url.into();
            fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
            assert!(Client::open(&path).await.is_err());
        }
        fs::write(&path, original).unwrap();
        assert!(Client::open(&path).await.is_ok());
        let mut old = Harness::start();
        assert!(
            Client::open(&old.directory.path().join("run/discovery.json"))
                .await
                .is_err()
        );
        old.close().await;
        h.close().await;
    });
}
