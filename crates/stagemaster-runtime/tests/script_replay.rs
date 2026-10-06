#[path = "../examples/script_replay/drive.rs"]
mod drive;
#[allow(dead_code)] // Reuse the established profile-authoring fixture, not a second semantic table.
#[path = "../../stagemaster-project/tests/support/fixture_function.rs"]
mod functions;
#[path = "../examples/script_replay/prepare.rs"]
mod prepare;
#[path = "../examples/script_replay/schedule.rs"]
mod schedule;
#[path = "../examples/script_replay/store.rs"]
mod store;
use serde_json::{Value, json};
use stagemaster_project::{Document, PackageSelection};

const ID: &str = "39999999-0000-4000-8000-000000000001";
fn fixture() -> (Document, Vec<u8>) {
    let mut doc = Document::new("源工程完整剧本核验").unwrap();
    let mut definition = functions::definition(true);
    definition["footprint"] = json!(7);
    definition["channels"][0]["fine"] = json!(7);
    definition["channels"][0]["defaultValue"] = json!(0x1234);
    functions::edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition}}),
    )
    .unwrap();
    let view = doc.view();
    functions::edit(
        &mut doc,
        json!({"op":"addFixture","name":"软件验收灯","profileId":view.profiles.last().unwrap().id,
        "domainId":view.domains[0].id,"universe":1,"address":37}),
    )
    .unwrap();
    for name in ["入场", "定时变换", "末步保持"] {
        functions::edit(&mut doc, json!({"op":"addScene","name":name})).unwrap();
    }
    let view = doc.view();
    for (scene, key) in view.scenes.iter().zip(["open", "closed", "strobe"]) {
        functions::choose(
            &mut doc,
            &scene.id,
            &view.fixtures[0].id,
            "shutter",
            key,
            if key == "strobe" { 65535 } else { 0 },
        )
        .unwrap();
    }
    functions::edit(&mut doc,json!({"op":"effect","command":{"kind":"put","sceneId":view.scenes[1].id,"effect":{
        "id":"39999999-0000-4000-8000-000000000002","name":"受控亮度","enabled":true,"fixtureIds":[view.fixtures[0].id],
        "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,
        "channels":[{"attribute":"dimmer","low":0,"high":65535}]}}})).unwrap();
    let mut raw = functions::raw(&doc);
    let duration = |ms: u64| json!({"ticks":ms.to_string(),"ticksPerSecond":"1000"});
    let steps = view.scenes.iter().enumerate().map(|(i,s)| json!({
        "id":format!("39999999-0000-4000-8000-{:012}",i+10),"name":s.name,"number":(i+1).to_string(),
        "sceneId":s.id,"actionIds":[],"delay":duration([50,100,0][i]),"fade":duration([200,400,0][i]),
        "advance":if i==1 {json!({"kind":"after","wait":duration(300)})} else {json!({"kind":"manual"})}
    })).collect::<Vec<_>>();
    raw["lighting"]["sequences"] = json!([{"id":ID,"name":"三步核心剧本","tracking":"inherited","repeat":"once",
        "release":"profile-defaults","steps":steps}]);
    let doc = functions::decode(&raw).unwrap();
    let bytes = doc
        .build_package(&[PackageSelection::Sequence { id: ID.into() }])
        .unwrap()
        .bytes;
    (doc, bytes)
}
fn checkpoint(report: &Value, time: u64) -> &Value {
    report["checkpoints"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["atMs"] == time)
        .unwrap()
}
#[test]
fn source_and_installed_full_script_match_through_every_boundary_and_authority_change() {
    let (doc, bytes) = fixture();
    let mut prepared = prepare::prepare(&doc, &bytes, ID).unwrap();
    assert_eq!(prepared.runtime.state().instance, None);
    let report = drive::run(&mut prepared).unwrap();
    assert_eq!(report["framesCompared"], 2551);
    assert_eq!(report["stepsVisited"], json!([0, 1, 2]));
    assert_eq!(report["storageReadsDuringExecution"], 0);
    assert_eq!(report["physicalOutput"], false);
    assert_eq!(report["authorizationQualified"], false);
    assert_eq!(
        checkpoint(&report, 749)["step"],
        uuid::Uuid::from_bytes(prepared.step_ids[0]).to_string()
    );
    assert_eq!(
        checkpoint(&report, 2049)["step"],
        uuid::Uuid::from_bytes(prepared.step_ids[1]).to_string()
    );
    assert_eq!(checkpoint(&report, 2049)["elapsedMs"], 799);
    assert_eq!(
        checkpoint(&report, 2050)["step"],
        uuid::Uuid::from_bytes(prepared.step_ids[2]).to_string()
    );
    assert_eq!(checkpoint(&report, 2050)["elapsedMs"], 0);
    assert_eq!(checkpoint(&report, 2051)["elapsedMs"], 1);
    assert_eq!(checkpoint(&report, 2050)["controlled"], false);
    assert_eq!(checkpoint(&report, 1549)["elapsedMs"], 300);
    assert_eq!(checkpoint(&report, 1549)["status"], "Paused");
    let mut slots = [0; 512];
    prepared.runtime.render(&mut slots).unwrap();
    assert_eq!(slots[36], 0x12);
    assert_eq!(slots[42], 0x34);
    assert_eq!(slots[39], 20); // Profile's open-shutter default, not universal zero.
    assert!(slots[..36].iter().all(|v| *v == 0));
}
#[test]
fn source_snapshot_mismatch_and_invalid_or_oversized_packages_never_start() {
    let (doc, bytes) = fixture();
    let mut raw = functions::raw(&doc);
    raw["project"]["name"] = json!("另一个已应用快照");
    let changed = functions::decode(&raw).unwrap();
    assert!(
        prepare::prepare(&changed, &bytes, ID)
            .err()
            .unwrap()
            .to_string()
            .contains("快照")
    );
    let mut corrupt = bytes.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(prepare::prepare(&doc, &corrupt, ID).is_err());
    assert!(
        prepare::prepare(
            &doc,
            &vec![0; stagemaster_package::MAX_PACKAGE_BYTES + 1],
            ID
        )
        .err()
        .unwrap()
        .to_string()
        .contains("容量")
    );
}
#[test]
fn unsupported_script_shape_is_refused_instead_of_claiming_partial_coverage() {
    let (doc, bytes) = fixture();
    let mut raw = functions::raw(&doc);
    raw["lighting"]["sequences"][0]["repeat"] = json!("loop");
    let looped = functions::decode(&raw).unwrap();
    assert!(
        prepare::prepare(&looped, &bytes, ID)
            .err()
            .unwrap()
            .to_string()
            .contains("三步列表")
    );
}
#[test]
fn comparison_detects_different_source_targets_instead_of_trusting_the_package_player() {
    let (doc, bytes) = fixture();
    let mut prepared = prepare::prepare(&doc, &bytes, ID).unwrap();
    let mut plan = doc.compile_sequence(ID).unwrap().plan;
    let mut steps = plan.steps().to_vec();
    steps[0].target[0] = 65535;
    plan = stagemaster_playback::Plan::with_snap_attributes(
        plan.defaults().to_vec(),
        steps,
        false,
        plan.effects().to_vec(),
        plan.snap_attributes().to_vec(),
    )
    .unwrap();
    prepared.reference = stagemaster_playback::Player::new(plan, 0);
    let error = drive::run(&mut prepared).unwrap_err().to_string();
    assert!(error.contains("通道与源工程输出不一致"), "{error}");
}

#[test]
fn zero_wait_and_two_millisecond_fade_retain_exact_boundary() {
    let (doc, _) = fixture();
    let mut raw = functions::raw(&doc);
    let second = &mut raw["lighting"]["sequences"][0]["steps"][1];
    second["delay"]["ticks"] = json!("0");
    second["fade"]["ticks"] = json!("2");
    second["advance"]["wait"]["ticks"] = json!("0");
    let doc = functions::decode(&raw).unwrap();
    let bytes = doc
        .build_package(&[PackageSelection::Sequence { id: ID.into() }])
        .unwrap()
        .bytes;
    let mut prepared = prepare::prepare(&doc, &bytes, ID).unwrap();
    let report = drive::run(&mut prepared).unwrap();
    assert_eq!(report["automaticTransitionAtMs"], 1252);
    assert_eq!(
        checkpoint(&report, 1251)["step"],
        uuid::Uuid::from_bytes(prepared.step_ids[1]).to_string()
    );
    assert_eq!(
        checkpoint(&report, 1252)["step"],
        uuid::Uuid::from_bytes(prepared.step_ids[2]).to_string()
    );
    assert_eq!(checkpoint(&report, 1252)["elapsedMs"], 0);
}
