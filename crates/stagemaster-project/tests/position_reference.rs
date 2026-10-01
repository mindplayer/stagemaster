#[path = "support/position_effect.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_project::{Document, ReferenceCheckView};
use support::{decode, edit, motion, put, raw, set, setup};
fn place(doc: &mut Document, id: &str) {
    edit(doc,json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":id,"spaceId":null,"positionMeters":{"x":"0","y":"0","z":"5"},"rotationDegreesXYZ":{"x":"0","y":"0","z":"0"}}}})).unwrap();
}
fn capture(doc: &mut Document, scene: &str, id: &str, name: &str) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"position","command":{"op":"captureReference","sceneId":scene,"fixtureId":id,"name":name,"targetMeters":{"x":"0","y":"0","z":"0"}}}),
    )
}
fn record(doc: &Document) -> Value {
    raw(doc)["lighting"]["fixtures"][0]["positionReference"].clone()
}
fn miss(doc: &Document) -> f64 {
    let v = doc.view();
    let r = v.fixtures[0].position_reference.as_ref().unwrap();
    if let ReferenceCheckView::Checked { miss_meters, .. } = r.points[0].check {
        miss_meters
    } else {
        panic!("expected geometry")
    }
}
#[test]
fn records_canonical_setpoint_precision_without_changing_program_or_output() {
    for fine in [false, true] {
        let (mut doc, scene, ids) = setup(fine);
        place(&mut doc, &ids[0]);
        set(&mut doc, &scene, &ids[0], "pan", 0x80ff);
        set(&mut doc, &scene, &ids[0], "tilt", 0x70ab);
        let before = raw(&doc);
        let compiled = doc.compile_scene(&scene).unwrap();
        let frame = compiled
            .output
            .render(&compiled.plan.steps()[0].target)
            .unwrap();
        capture(&mut doc, &scene, &ids[0], "台口中心").unwrap();
        let r = record(&doc);
        assert_eq!(
            r["points"][0]["panValue"],
            if fine { 0x80ff } else { 0x8080 }
        );
        assert_eq!(
            r["points"][0]["tiltValue"],
            if fine { 0x70ab } else { 0x7070 }
        );
        assert_eq!(r["points"][0]["source"], "sceneSetpoint");
        let after = raw(&doc);
        for key in ["profiles", "scenes", "presets", "patches"] {
            assert_eq!(before["lighting"][key], after["lighting"][key]);
        }
        let c = doc.compile_scene(&scene).unwrap();
        assert_eq!(
            c.output.render(&c.plan.steps()[0].target).unwrap().slots,
            frame.slots
        );
        assert_eq!(decode(&after), doc);
        assert!(miss(&doc).is_finite());
        set(&mut doc, &scene, &ids[0], "pan", 12345);
        assert_eq!(record(&doc), r);
        edit(&mut doc, json!({"op":"removeScene","id":scene})).unwrap();
        assert_eq!(record(&doc), r);
    }
}
#[test]
fn zero_and_installation_recompute_but_profile_exchange_preserves_and_invalidates_history() {
    let (mut doc, scene, ids) = setup(true);
    place(&mut doc, &ids[0]);
    capture(&mut doc, &scene, &ids[0], "台口").unwrap();
    let old = record(&doc);
    let first = miss(&doc);
    edit(&mut doc,json!({"op":"position","command":{"op":"calibrate","fixtureId":ids[0],"correction":{"panDegrees":"0","tiltDegrees":"30"}}})).unwrap();
    assert!(miss(&doc) > first + 1.0);
    assert_eq!(record(&doc), old);
    let mut root = raw(&doc);
    root["stage"]["placements"][0]["rotationDegreesXYZ"]["x"] = json!("180");
    doc = decode(&root);
    assert_eq!(record(&doc), old);
    assert!(
        doc.view().fixtures[0]
            .position_reference
            .as_ref()
            .unwrap()
            .compatible
    );
    let profile = doc.view().fixtures[0].profile_id.clone();
    let def = serde_json::to_value(doc.profile_file(&profile).unwrap().definition()).unwrap();
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":def}}),
    )
    .unwrap();
    let target = doc.view().profiles.last().unwrap().id.clone();
    edit(&mut doc,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":[ids[0]],"profileId":target,"layout":null}})).unwrap();
    assert_eq!(record(&doc), old);
    assert!(
        !doc.view().fixtures[0]
            .position_reference
            .as_ref()
            .unwrap()
            .compatible
    );
    assert!(
        capture(&mut doc, &scene, &ids[0], "新点")
            .unwrap_err()
            .contains("档案已改变")
    );
    edit(
        &mut doc,
        json!({"op":"position","command":{"op":"clearReferences","fixtureId":ids[0]}}),
    )
    .unwrap();
    capture(&mut doc, &scene, &ids[0], "新点").unwrap();
    let point = record(&doc)["points"][0]["id"].clone();
    edit(&mut doc,json!({"op":"position","command":{"op":"removeReference","fixtureId":ids[0],"pointId":point}})).unwrap();
    assert!(record(&doc).is_null());
}
#[test]
fn capture_rejects_missing_installation_effects_pivot_duplicates_and_invalid_records_atomically() {
    let (mut doc, scene, ids) = setup(false);
    let before = doc.clone();
    assert!(capture(&mut doc, &scene, &ids[0], "无灯位").is_err());
    assert_eq!(doc, before);
    place(&mut doc, &ids[0]);
    put(&mut doc, &scene, &motion(&ids)).unwrap();
    let before = doc.clone();
    assert!(
        capture(&mut doc, &scene, &ids[0], "动态位置")
            .unwrap_err()
            .contains("效果")
    );
    assert_eq!(doc, before);
    let mut r = raw(&doc);
    r["lighting"]["scenes"][0]["effects"][0]["enabled"] = json!(false);
    doc = decode(&r);
    let before = doc.clone();
    assert!(edit(&mut doc,json!({"op":"position","command":{"op":"captureReference","sceneId":scene,"fixtureId":ids[0],"name":"轴心","targetMeters":{"x":"0","y":"0","z":"5"}}})).is_err());
    assert_eq!(doc, before);
    capture(&mut doc, &scene, &ids[0], "台口").unwrap();
    let before = doc.clone();
    assert!(capture(&mut doc, &scene, &ids[0], " 台口 ").is_err());
    assert_eq!(doc, before);
    let base = raw(&doc);
    for (pointer, value) in [
        (
            "/lighting/fixtures/0/positionReference/points/0/source",
            json!("deviceFeedback"),
        ),
        (
            "/lighting/fixtures/0/positionReference/points/0/panValue",
            json!(65536),
        ),
        (
            "/lighting/fixtures/0/positionReference/points/0/targetMeters/x",
            json!("100001"),
        ),
        (
            "/lighting/fixtures/0/positionReference/points/0/name",
            json!("  "),
        ),
    ] {
        let mut r = base.clone();
        *r.pointer_mut(pointer).unwrap() = value;
        assert!(Document::decode(&serde_json::to_vec(&r).unwrap()).is_err());
    }
    let mut r = base.clone();
    r["lighting"]["fixtures"][0]["positionReference"]["extra"] = json!(true);
    assert!(Document::decode(&serde_json::to_vec(&r).unwrap()).is_err());
    let mut r = base;
    r["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["key"] != "lighting.position-reference");
    assert!(Document::decode(&serde_json::to_vec(&r).unwrap()).is_err());
}
#[test]
fn per_fixture_and_project_limits_are_enforced_before_acceptance() {
    let (mut doc, scene, ids) = setup(true);
    place(&mut doc, &ids[0]);
    for i in 0..16 {
        capture(&mut doc, &scene, &ids[0], &format!("点 {i}")).unwrap();
    }
    let before = doc.clone();
    assert!(capture(&mut doc, &scene, &ids[0], "第 17 点").is_err());
    assert_eq!(doc, before);
    let mut r = raw(&doc);
    let template = r["lighting"]["fixtures"][0].clone();
    for _ in 0..64 {
        let mut f = template.clone();
        f["id"] = json!(uuid::Uuid::new_v4().to_string());
        for point in f["positionReference"]["points"].as_array_mut().unwrap() {
            point["id"] = json!(uuid::Uuid::new_v4().to_string());
        }
        r["lighting"]["fixtures"].as_array_mut().unwrap().push(f);
    }
    assert!(
        Document::decode(&serde_json::to_vec(&r).unwrap())
            .unwrap_err()
            .contains("1024")
    );
    r["lighting"]["fixtures"].as_array_mut().unwrap().pop();
    decode(&r);
}
