mod support;
use serde_json::{Value, json};
use support::{Harness, group::*, *};

async fn output(h: &Harness, session: &str, serial: u64, revision: &Value, action: Value) -> Value {
    h.command(
        session,
        serial,
        json!({"kind":"submit","expectedRevision":revision,
        "action":{"kind":"output","action":action}}),
    )
    .await
}

#[test]
fn master_blackout_and_release_preserve_two_programs_and_manual_ownership() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let catalog = ok(h.get("/source")).await;
        assert!(catalog["capabilities"].as_array().unwrap().contains(&json!("outputMaster")));
        assert_eq!(catalog["output"]["uncontrolledFixtures"], 0);
        let initial = h.state().await;
        assert_eq!(initial["output"], json!({"percent":100,"blackout":false}));
        let session = h.session().await;
        let acquired = h.acquire(&session, false).await;
        let one = start(&h, &session, 2, &acquired["state"]["revision"], 0).await;
        let two = start(&h, &session, 3, &one["state"]["revision"], 1).await;
        let fixture = catalog["fixtures"][0]["id"].clone();
        let manual = apply(&h, &session, 4, &two["state"]["revision"], 3,
            json!({"kind":"patch","changes":[
                {"fixtureId":fixture,"attribute":"dimmer","value":{"kind":"normalized","value":65535}},
                {"fixtureId":fixture,"attribute":"color-wheel","value":{"kind":"function","functionKey":"red","position":0}}
            ]})).await;
        until(&h, |s| s["frame"]["slots"][0] == 255 && s["frame"]["slots"][1] == 20).await;
        let half = output(&h, &session, 5, &manual["state"]["revision"], json!({"kind":"level","percent":50})).await;
        assert_eq!(half["kind"], "applied");
        assert_eq!(output(&h, &session, 5, &manual["state"]["revision"], json!({"kind":"level","percent":50})).await, half);
        until(&h, |s| s["frame"]["slots"][0] == 128 && s["frame"]["slots"][1] == 20).await;
        let black = output(&h, &session, 6, &half["state"]["revision"], json!({"kind":"blackout","enabled":true})).await;
        assert_eq!(black["state"]["output"], json!({"percent":50,"blackout":true}));
        let dark = until(&h, |s| s["frame"]["slots"][0] == 0).await;
        let later = until(&h, |s| s["state"]["observedMs"] != dark["state"]["observedMs"]).await;
        assert_eq!(later["frame"]["slots"][1], 20);
        assert_eq!(later["state"]["sources"][2]["heldValues"], json!([65535,20*257]));
        assert_eq!(later["state"]["sources"][0]["status"], "Running");
        assert_eq!(later["state"]["sources"][1]["status"], "Running");
        let zero = output(&h, &session, 7, &black["state"]["revision"], json!({"kind":"level","percent":0})).await;
        assert_eq!(zero["state"]["output"], json!({"percent":0,"blackout":true}));
        let unblack = output(&h, &session, 8, &zero["state"]["revision"], json!({"kind":"blackout","enabled":false})).await;
        until(&h, |s| s["frame"]["slots"][0] == 0).await;
        let full = output(&h, &session, 9, &unblack["state"]["revision"], json!({"kind":"level","percent":100})).await;
        until(&h, |s| s["frame"]["slots"][0] == 255).await;
        let release = apply(&h, &session, 10, &full["state"]["revision"], 3, json!({"kind":"stop"})).await;
        assert!(release["state"]["sources"][2]["held"].as_array().unwrap().is_empty());
        until(&h, |s| s["frame"]["slots"][0] != 255 && s["frame"]["slots"][1] == 40).await;
        let retained = output(&h, &session, 11, &release["state"]["revision"], json!({"kind":"level","percent":37})).await;
        assert_eq!(retained["kind"], "applied");
        assert_eq!(h.command(&session, 12, json!({"kind":"release"})).await["kind"], "released");
        let mut reader = stagemaster_execution_client::Reader::open(&h.directory.path().join("run/discovery.json")).await.unwrap();
        assert_eq!(reader.sample().await.unwrap().slots[1], 40);
        let client = stagemaster_execution_client::Client::open(&h.directory.path().join("run/discovery.json")).await.unwrap();
        assert!(!client.view().controlling);
        assert_eq!(h.state().await["output"], json!({"percent":37,"blackout":false}));
        h.close().await;
    });
}
