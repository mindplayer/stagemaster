#[path = "support/world_line.rs"]
mod support;
use serde_json::json;
use support::{
    common::{decode, edit, put, raw},
    effect, setup,
};

#[test]
fn world_curves_are_charged_to_existing_budget_and_inactive_copies_cost_nothing() {
    let (mut doc, scene, mut ids) = setup(true);
    let view = doc.view();
    edit(&mut doc, json!({"op":"addFixture","name":"灯 11","profileId":view.profiles.last().unwrap().id,"domainId":view.domains[0].id,"universe":1,"address":11})).unwrap();
    let id = doc.view().fixtures.last().unwrap().id.clone();
    let mut placement = serde_json::to_value(&view.stage.placements[0]).unwrap();
    placement["fixtureId"] = json!(id);
    edit(
        &mut doc,
        json!({"op":"stage","command":{"op":"putPlacement","placement":placement}}),
    )
    .unwrap();
    ids.push(id);
    put(&mut doc, &scene, &effect(&ids, true)).unwrap();
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"轨迹容量","sceneId":scene}}),
    )
    .unwrap();
    let mut root = raw(&doc);
    let sequence = &mut root["lighting"]["sequences"][0];
    let template = sequence["steps"][0].clone();
    sequence["steps"] = (0..683)
        .map(|i| {
            let mut step = template.clone();
            step["id"] = json!(format!("49999999-0000-4000-8001-{i:012x}"));
            step["number"] = json!((i + 1).to_string());
            step
        })
        .collect();
    let id = sequence["id"].as_str().unwrap().to_owned();
    assert!(
        decode(&root)
            .compile_sequence(&id)
            .err()
            .unwrap()
            .contains("关键帧超出计划容量")
    );
    root["lighting"]["sequences"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert_eq!(
        decode(&root)
            .compile_sequence(&id)
            .unwrap()
            .plan
            .keyframe_count(),
        682 * 192
    );
    root["lighting"]["scenes"][0]["effects"][0]["enabled"] = json!(false);
    assert_eq!(
        decode(&root)
            .compile_sequence(&id)
            .unwrap()
            .plan
            .keyframe_count(),
        0
    );
}

#[test]
fn path_axis_crossing_is_contextual_error_not_a_flip_or_a_silent_disabled_effect() {
    let (mut doc, scene, ids) = setup(true);
    let mut moving = effect(&ids, true);
    moving["targetPath"]["fromMeters"] = json!({"x":"-2","y":"-2","z":"0"});
    moving["targetPath"]["toMeters"] = json!({"x":"0","y":"-2","z":"0"});
    put(&mut doc, &scene, &moving).unwrap();
    let before = doc.encode().unwrap();
    let error = doc.compile_scene(&scene).err().unwrap();
    assert!(
        error.contains("运动场景")
            && error.contains("共同直线")
            && error.contains("灯 1")
            && error.contains("转轴"),
        "{error}"
    );
    assert_eq!(doc.encode().unwrap(), before);
}
