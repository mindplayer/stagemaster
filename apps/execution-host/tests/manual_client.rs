mod support;
use serde_json::json;
use stagemaster_execution_client::{Action, Client, ManualEdit, ManualValue, Reader, View};
use support::{Harness, group::*, runtime};
fn revision(v: &View) -> String {
    v.observation
        .snapshot
        .as_ref()
        .unwrap()
        .state
        .revision
        .clone()
}
fn held(v: &View) -> usize {
    v.observation.snapshot.as_ref().unwrap().state.sources[2]
        .held
        .as_ref()
        .unwrap()
        .len()
}
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
async fn apply(client: &mut Client, action: Action) -> View {
    let v = settled(client).await;
    client
        .apply(&v.host_id, &revision(&v), &id(3), action)
        .await
        .unwrap();
    let result = settled(client).await;
    assert_eq!(
        result
            .record
            .as_ref()
            .unwrap()
            .outcome
            .as_ref()
            .unwrap()
            .kind,
        "applied"
    );
    result
}
#[test]
fn typed_manual_controls_hold_zero_release_individually_and_survive_client_reopen() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let mut client = Client::open(&path).await.unwrap();
        client.acquire(false).await.unwrap();
        let v = settled(&mut client).await;
        assert_eq!(held(&v), 0);
        let source = &v.catalog.sources[1];
        client
            .apply(
                &v.host_id,
                &revision(&v),
                &source.id,
                Action::Start {
                    step: source.steps[0].id.clone(),
                },
            )
            .await
            .unwrap();
        until(&h, |s| s["frame"]["slots"][1] == 40).await;
        let fixture = &v.catalog.fixtures.as_ref().unwrap()[0].id;
        let edit = |a: &str, value| ManualEdit {
            fixture_id: fixture.clone(),
            attribute: a.into(),
            value,
        };
        let v = apply(
            &mut client,
            Action::Patch {
                changes: vec![
                    edit("dimmer", ManualValue::Normalized { value: 0 }),
                    edit(
                        "color-wheel",
                        ManualValue::Function {
                            function_key: "red".into(),
                            position: 0,
                        },
                    ),
                ],
            },
        )
        .await;
        assert_eq!(held(&v), 2);
        until(&h, |s| {
            s["frame"]["slots"][0] == 0 && s["frame"]["slots"][1] == 20
        })
        .await;
        let mut reader = Reader::open(&path).await.unwrap();
        assert_eq!(reader.sample().await.unwrap().slots[1], 20);
        for changes in [
            vec![edit("color-wheel", ManualValue::Normalized { value: 0 })],
            vec![edit(
                "color-wheel",
                ManualValue::Function {
                    function_key: "red".into(),
                    position: 1,
                },
            )],
            vec![edit("missing", ManualValue::Release {})],
            vec![
                edit("dimmer", ManualValue::Release {}),
                edit("dimmer", ManualValue::Release {}),
            ],
        ] {
            assert!(
                client
                    .apply(&v.host_id, &revision(&v), &id(3), Action::Patch { changes })
                    .await
                    .is_err()
            );
            assert!(!client.view().pending);
            assert_eq!(held(&client.refresh().await.unwrap()), 2);
        }
        let v = apply(&mut client, Action::Level { value: 0 }).await;
        assert_eq!(held(&v), 2); // fader zero keeps wheel and ownership
        let v = apply(
            &mut client,
            Action::Patch {
                changes: vec![edit("color-wheel", ManualValue::Release {})],
            },
        )
        .await;
        assert_eq!(held(&v), 1);
        until(&h, |s| s["frame"]["slots"][1] == 40).await;
        drop(client);
        let mut client = Client::open(&path).await.unwrap();
        assert!(!client.view().controlling);
        assert_eq!(held(&client.view()), 1);
        client.acquire(true).await.unwrap();
        let v = apply(&mut client, Action::Stop {}).await;
        assert_eq!(held(&v), 0);
        assert_eq!(
            v.observation.snapshot.as_ref().unwrap().state.sources[1]
                .status
                .as_deref(),
            Some("Running")
        );
        h.close().await;
    });
}
#[test]
fn oversized_manual_batch_refuses_before_sequence_admission_and_next_small_edit_succeeds() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|path| {
            let manifest = write(path);
            let mut raw: serde_json::Value =
                serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            let mut fixture = raw["lighting"]["fixtures"][0].clone();
            let mut patch = raw["lighting"]["patches"][0].clone();
            for index in 1..100_u16 {
                let key = uuid::Uuid::new_v4().to_string();
                fixture["id"] = json!(key);
                fixture["name"] = json!(format!("灯具{index}"));
                patch["fixtureId"] = json!(key);
                patch["address"] = json!(1 + index * 5);
                raw["lighting"]["fixtures"]
                    .as_array_mut()
                    .unwrap()
                    .push(fixture.clone());
                raw["lighting"]["patches"]
                    .as_array_mut()
                    .unwrap()
                    .push(patch.clone());
            }
            std::fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
            Some(manifest)
        });
        let mut client = Client::open(&h.directory.path().join("run/discovery.json"))
            .await
            .unwrap();
        client.acquire(false).await.unwrap();
        let v = settled(&mut client).await;
        let changes: Vec<_> = v
            .catalog
            .fixtures
            .as_ref()
            .unwrap()
            .iter()
            .map(|f| ManualEdit {
                fixture_id: f.id.clone(),
                attribute: "dimmer".into(),
                value: ManualValue::Normalized { value: 0 },
            })
            .collect();
        let small = changes.last().unwrap().clone();
        let last_fixture = small.fixture_id.clone();
        let error = client
            .apply(&v.host_id, &revision(&v), &id(3), Action::Patch { changes })
            .await
            .unwrap_err();
        assert!(error.contains("请求容量"), "{error}");
        assert!(!client.view().pending);
        assert_eq!(held(&client.refresh().await.unwrap()), 0);
        let v = apply(
            &mut client,
            Action::Patch {
                changes: vec![small],
            },
        )
        .await;
        assert_eq!(held(&v), 1);
        assert_eq!(
            v.observation.snapshot.as_ref().unwrap().state.sources[2]
                .held
                .as_ref()
                .unwrap()[0]
                .fixture_id,
            last_fixture
        );
        h.close().await;
    });
}
