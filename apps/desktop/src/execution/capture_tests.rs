use super::tests::{Cleanup, binary, connected, document, runtime};
use super::*;
use stagemaster_execution_client::{ManualEdit, ManualValue};
#[test]
fn actual_backend_capture_keeps_zero_level_readonly_and_frozen_values() {
    runtime().block_on(async {
        let temp =
            tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp"))
                .unwrap();
        let doc = two_lights();
        let mut manager = Manager::new(temp.path().join("run"), binary());
        manager
            .prepare(
                doc.clone(),
                vec![Selection::Scene {
                    id: doc.view().scenes[0].id.clone(),
                }],
                None,
            )
            .await
            .unwrap();
        let _cleanup = Cleanup(manager.child.take().unwrap());
        let view = connected(&mut manager).await.runtime.unwrap();
        let host = view.host_id;
        manager.acquire(false).await.unwrap();
        let view = connected(&mut manager).await.runtime.unwrap();
        let source = view
            .catalog
            .sources
            .iter()
            .find(|s| matches!(s.selection, Selection::Manual {}))
            .unwrap()
            .id
            .clone();
        let fixtures: Vec<_> = view
            .catalog
            .fixtures
            .as_ref()
            .unwrap()
            .iter()
            .map(|f| f.id.clone())
            .collect();
        seed(&mut manager, &host, &source, &fixtures, view).await;
        assert_receipt_survives_renew(&mut manager).await;
        let view = connected(&mut manager).await.runtime.unwrap();
        manager
            .apply(
                &host,
                &view.observation.snapshot.unwrap().state.revision,
                &source,
                Action::Level { value: 0 },
            )
            .await
            .unwrap();
        connected(&mut manager).await;
        manager.release().await.unwrap();
        connected(&mut manager).await;
        let all = manager.capture(&host, &source, None).await.unwrap();
        assert_eq!(all.capture.readings().len(), 2);
        assert_eq!(all.capture.readings()[0].value, 0);
        assert_eq!(all.capture.readings()[1].value, 32768);
        let selected = manager
            .capture(&host, &source, Some(&fixtures[1..2]))
            .await
            .unwrap();
        assert_eq!(selected.capture.readings().len(), 1);
        assert!(manager.capture("wrong", &source, None).await.is_err());
        assert!(manager.capture(&host, &source, Some(&[])).await.is_err());
        manager.acquire(false).await.unwrap();
        let view = connected(&mut manager).await.runtime.unwrap();
        manager
            .apply(
                &host,
                &view.observation.snapshot.unwrap().state.revision,
                &source,
                Action::Stop {},
            )
            .await
            .unwrap();
        connected(&mut manager).await;
        assert!(manager.capture(&host, &source, None).await.is_err());
        let mut recorded = doc.clone();
        recorded
            .record_manual_scene(&all.capture, "冻结记录")
            .unwrap();
        assert_eq!(recorded.view().scenes.last().unwrap().values.len(), 2);
        manager.shutdown(&host).await.unwrap();
    });
}

fn two_lights() -> Document {
    let mut doc = document();
    let initial = doc.view();
    doc.edit(stagemaster_project::EditCommand::AddFixture {
        name: "第二台灯".into(),
        profile_id: initial.fixtures[0].profile_id.clone(),
        domain_id: initial.domains[0].id.clone(),
        universe: 1,
        address: 100,
    })
    .unwrap();
    doc
}

async fn seed(manager: &mut Manager, host: &str, source: &str, fixtures: &[String], view: View) {
    let revision = &view.observation.snapshot.unwrap().state.revision;
    manager
        .apply(
            host,
            revision,
            source,
            Action::Patch {
                changes: vec![
                    ManualEdit {
                        fixture_id: fixtures[0].clone(),
                        attribute: "dimmer".into(),
                        value: ManualValue::Normalized { value: 0 },
                    },
                    ManualEdit {
                        fixture_id: fixtures[1].clone(),
                        attribute: "dimmer".into(),
                        value: ManualValue::Normalized { value: 32768 },
                    },
                ],
            },
        )
        .await
        .unwrap();
}

async fn assert_receipt_survives_renew(manager: &mut Manager) {
    let before = connected(manager).await.runtime.unwrap();
    let record = before.record.unwrap();
    assert_eq!(record.status, "complete");
    assert_eq!(record.outcome.as_ref().unwrap().kind, "applied");
    let expiry = before
        .observation
        .snapshot
        .unwrap()
        .state
        .owner
        .unwrap()
        .expires_ms;
    // Cross the real 30-second renewal threshold, not a manually fabricated receipt.
    tokio::time::sleep(Duration::from_secs(31)).await;
    let after = connected(manager).await.runtime.unwrap();
    assert!(after.controlling);
    let renewed = after
        .observation
        .snapshot
        .unwrap()
        .state
        .owner
        .unwrap()
        .expires_ms;
    assert!(renewed.parse::<u64>().unwrap() > expiry.parse::<u64>().unwrap());
    assert_eq!(
        serde_json::to_value(after.record.unwrap()).unwrap(),
        serde_json::to_value(record).unwrap()
    );
}
