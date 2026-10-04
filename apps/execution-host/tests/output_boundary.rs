mod support;
use serde_json::json;
use stagemaster_execution_client::{Client, OutputAction, View};
use support::*;

async fn settled(client: &mut Client) -> View {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(6);
    loop {
        let view = client.refresh().await.unwrap();
        if !view.pending {
            return view;
        }
        assert!(std::time::Instant::now() < end, "{view:?}");
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

async fn reject_local_invalid_input(client: &mut Client, view: &View) {
    let revision = &view.observation.snapshot.as_ref().unwrap().state.revision;
    for (host, rev, action) in [
        (
            "other",
            revision.as_str(),
            OutputAction::Level { percent: 50 },
        ),
        (
            view.host_id.as_str(),
            revision.as_str(),
            OutputAction::Level { percent: 101 },
        ),
        (
            view.host_id.as_str(),
            "01",
            OutputAction::Blackout { enabled: true },
        ),
    ] {
        assert!(client.output(host, rev, action).await.is_err());
        assert!(!client.view().pending);
        assert_eq!(client.view().record.as_ref().unwrap().serial, "1");
    }
}

#[test]
fn output_commands_share_revision_lease_and_serial_but_invalid_input_never_advances_them() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let mut client = Client::open(&path).await.unwrap();
        client.acquire(false).await.unwrap();
        let view = settled(&mut client).await;
        let revision = &view.observation.snapshot.as_ref().unwrap().state.revision;
        reject_local_invalid_input(&mut client, &view).await;
        client
            .output(&view.host_id, revision, OutputAction::Level { percent: 37 })
            .await
            .unwrap();
        let changed = settled(&mut client).await;
        assert_eq!(changed.record.as_ref().unwrap().serial, "2");
        assert_eq!(
            changed
                .observation
                .snapshot
                .as_ref()
                .unwrap()
                .state
                .output
                .as_ref()
                .unwrap()
                .percent,
            37
        );
        let mut other = Client::open(&path).await.unwrap();
        let observation = other.view();
        other
            .output(
                &observation.host_id,
                revision,
                OutputAction::Blackout { enabled: true },
            )
            .await
            .unwrap_err();
        other.acquire(true).await.unwrap();
        settled(&mut other).await;
        client
            .output(
                &view.host_id,
                &changed
                    .observation
                    .snapshot
                    .as_ref()
                    .unwrap()
                    .state
                    .revision,
                OutputAction::Level { percent: 100 },
            )
            .await
            .unwrap();
        let replaced = settled(&mut client).await;
        assert_eq!(replaced.record.unwrap().outcome.unwrap().kind, "rejected");
        assert_eq!(
            h.state().await["output"],
            json!({"percent":37,"blackout":false})
        );
        other
            .output(&view.host_id, "0", OutputAction::Blackout { enabled: true })
            .await
            .unwrap();
        let stale = settled(&mut other).await;
        assert_eq!(stale.record.unwrap().outcome.unwrap().kind, "rejected");
        let current = settled(&mut other).await;
        other
            .output(
                &view.host_id,
                &current
                    .observation
                    .snapshot
                    .as_ref()
                    .unwrap()
                    .state
                    .revision,
                OutputAction::Blackout { enabled: true },
            )
            .await
            .unwrap();
        let actual = settled(&mut other).await;
        assert_eq!(actual.record.unwrap().outcome.unwrap().kind, "applied");
        drop(other);
        drop(client);
        let readonly = Client::open(&path).await.unwrap();
        assert!(!readonly.view().controlling);
        let kept = readonly
            .view()
            .observation
            .snapshot
            .unwrap()
            .state
            .output
            .unwrap();
        assert_eq!(kept.percent, 37);
        assert!(kept.blackout);
        h.close().await;
    });
}

#[test]
fn expired_input_lease_keeps_master_and_does_not_authorize_an_old_controller() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let session = h.session().await;
        let grant = h
            .command(
                &session,
                1,
                json!({"kind":"acquire","durationMs":1000,"takeover":false}),
            )
            .await;
        let change = h
            .command(
                &session,
                2,
                json!({"kind":"submit","expectedRevision":grant["state"]["revision"],
            "action":{"kind":"output","action":{"kind":"level","percent":37}}}),
            )
            .await;
        assert_eq!(change["kind"], "applied");
        let expired = support::group::until(&h, |s| s["state"]["owner"].is_null()).await;
        assert_eq!(
            expired["state"]["output"],
            json!({"percent":37,"blackout":false})
        );
        let denied = h
            .command(
                &session,
                3,
                json!({"kind":"submit","expectedRevision":expired["state"]["revision"],
            "action":{"kind":"output","action":{"kind":"level","percent":100}}}),
            )
            .await;
        assert_eq!(denied["kind"], "rejected");
        assert_eq!(
            h.state().await["output"],
            json!({"percent":37,"blackout":false})
        );
        h.close().await;
    });
}

#[test]
fn invalid_output_wire_and_v1_profile_cannot_admit_group_master_commands() {
    runtime().block_on(async {
        for grouped in [true, false] {
            let mut h = if grouped { Harness::start_group() } else { Harness::start() };
            let session = h.session().await;
            let initial = h.acquire(&session, false).await;
            let serial = 2;
            for action in [
                json!({"kind":"level","percent":101}),
                json!({"kind":"level","percent":1.5}),
                json!({"kind":"blackout","enabled":"true"}),
                json!({"kind":"level","percent":50,"enabled":false}),
            ] {
                let response = h.post(&format!("/sessions/{session}/commands")).json(&json!({"serial":serial.to_string(),"ttlMs":100,
                    "command":{"kind":"submit","expectedRevision":initial["state"]["revision"],"action":{"kind":"output","action":action}}})).send().await.unwrap();
                assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
            }
            let result = h.command(&session, serial, json!({"kind":"submit","expectedRevision":initial["state"]["revision"],
                "action":{"kind":"output","action":{"kind":"level","percent":50}}})).await;
            assert_eq!(result["kind"], if grouped { "applied" } else { "rejected" });
            h.close().await;
        }
    });
}
