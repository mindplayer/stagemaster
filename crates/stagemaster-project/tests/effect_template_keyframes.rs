#[path = "support/effect_template.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{Document, EffectTemplateFile};
use support::{edit, raw, setup, template};

fn keyframe_json() -> Value {
    let mut value: Value = serde_json::from_slice(&template().encode().unwrap()).unwrap();
    value["formatVersion"] = json!(2);
    value["definition"]["recipe"] = json!({"kind":"intensity-keyframes","keyframes":[
        {"position":0,"value":0,"transition":"hold"},
        {"position":2500,"value":65535,"transition":"linear"},
        {"position":5000,"value":20000,"transition":"smooth"},
        {"position":7500,"value":1000,"transition":"smooth"}
    ]});
    value
}
fn decode(raw: &Value) -> Result<EffectTemplateFile, String> {
    EffectTemplateFile::decode(&serde_json::to_vec(raw).unwrap())
}

#[test]
fn authored_keyframes_export_and_bind_another_project_without_changing_output() {
    let (mut original, scene, ids) = setup();
    let frames = keyframe_json()["definition"]["recipe"]["keyframes"].clone();
    edit(
        &mut original,
        json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{
            "id":"19999999-0000-4000-8000-000000000010","name":"原有追逐","enabled":true,
            "fixtureIds":ids,"waveform":"keyframes","periodMs":1000,"phaseDegrees":0,
            "spreadDegrees":0,"reverse":false,"dutyPercent":25,
            "channels":[{"attribute":"dimmer","keyframes":frames}]
        }}}),
    );
    let file = original
        .effect_template_file(&scene, "19999999-0000-4000-8000-000000000010")
        .unwrap();
    assert_eq!(file.template().format_version, 2);
    assert_eq!(
        serde_json::to_value(&file.template().definition.recipe).unwrap()["keyframes"],
        frames
    );
    let text = String::from_utf8(file.encode().unwrap()).unwrap();
    for id in ids.iter().chain([&scene]) {
        assert!(!text.contains(id));
    }
    assert!(!text.contains("attribute"));
    assert!(!text.contains("address"));

    let (mut target, target_scene, target_ids) = setup();
    assert_ne!(ids, target_ids);
    let before = raw(&target);
    let review = target
        .review_effect_template(&file, &target_scene, &target_ids)
        .unwrap();
    assert_eq!(raw(&target), before);
    assert_eq!(review.view().usage.keyframes, 8);
    target.apply_effect_template(review).unwrap();
    let compiled = original.compile_scene(&scene).unwrap();
    let bound = target.compile_scene(&target_scene).unwrap();
    let mut source_player = Player::new(compiled.plan, 0);
    let mut target_player = Player::new(bound.plan, 0);
    source_player.execute(0, 0).unwrap();
    target_player.execute(0, 0).unwrap();
    for time in [
        0, 249, 250, 251, 400, 499, 500, 650, 749, 750, 950, 999, 1000, 1250, 2000,
    ] {
        source_player.advance(time).unwrap();
        target_player.advance(time).unwrap();
        let expected = compiled.output.render(source_player.values()).unwrap();
        let actual = bound.output.render(target_player.values()).unwrap();
        assert_eq!(actual.slots, expected.slots, "time {time}");
        assert_eq!(actual.slots[12], 48);
        if let Some(value) = match time {
            0 | 249 | 1000 | 2000 => Some(0u16),
            250 | 1250 => Some(65535),
            500 => Some(20000),
            750 => Some(1000),
            _ => None,
        } {
            assert_eq!(
                u16::from_be_bytes([actual.slots[103], actual.slots[101]]),
                value
            );
        }
    }
    let after = raw(&target);
    assert_eq!(
        after["lighting"]["scenes"][0]["assignments"],
        before["lighting"]["scenes"][0]["assignments"]
    );
    assert_eq!(target, Document::decode(&target.encode().unwrap()).unwrap());
    let exported = target
        .effect_template_file(&target_scene, &target.view().scenes[0].effects[0].id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&exported.template().definition).unwrap(),
        serde_json::to_value(&file.template().definition).unwrap()
    );
}

#[test]
fn versions_and_keyframe_boundaries_are_strict_without_reordering_or_dropping_fields() {
    let valid = keyframe_json();
    for bad in [
        {
            let mut v = valid.clone();
            v["formatVersion"] = json!(1);
            v
        },
        {
            let mut v = valid.clone();
            v["formatVersion"] = json!(3);
            v
        },
        {
            let mut v = valid.clone();
            v["definition"]["recipe"]["channel"] = json!(1);
            v
        },
        {
            let mut v = valid.clone();
            v["definition"]["recipe"]["keyframes"][0]["position"] = json!(1);
            v
        },
        {
            let mut v = valid.clone();
            v["definition"]["recipe"]["keyframes"][1]["position"] = json!(0);
            v
        },
        {
            let mut v = valid.clone();
            v["definition"]["recipe"]["keyframes"][3]["position"] = json!(4999);
            v
        },
        {
            let mut v = valid.clone();
            v["definition"]["recipe"]["keyframes"][3]["position"] = json!(10000);
            v
        },
        {
            let mut v = valid.clone();
            v["definition"]["recipe"]["keyframes"][0]["value"] = json!(65536);
            v
        },
        {
            let mut v = valid.clone();
            v["definition"]["recipe"]["keyframes"][0]["transition"] = json!("bezier");
            v
        },
        {
            let mut v = valid.clone();
            v["definition"]["recipe"]["keyframes"] = json!([]);
            v
        },
        {
            let mut v = valid.clone();
            v["definition"]["recipe"]["keyframes"] =
                json!([{"position":0,"value":0,"transition":"hold"}]);
            v
        },
    ] {
        assert!(decode(&bad).is_err(), "accepted {bad}");
    }
    let mut boundary = valid;
    boundary["definition"]["recipe"]["keyframes"] = json!(
        (0..32)
            .map(|i| json!({"position":i*300,"value":i*2000,"transition":"linear"}))
            .collect::<Vec<_>>()
    );
    assert!(decode(&boundary).is_ok());
    boundary["definition"]["recipe"]["keyframes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"position":9999,"value":65535,"transition":"hold"}));
    assert!(decode(&boundary).is_err());
    let mut old: Value = serde_json::from_slice(&template().encode().unwrap()).unwrap();
    assert!(decode(&old).is_ok());
    old["formatVersion"] = json!(2);
    assert!(decode(&old).is_err());
}

#[test]
fn keyframe_source_capability_survives_local_edits_and_cannot_be_omitted() {
    let (mut document, scene, ids) = setup();
    let file = decode(&keyframe_json()).unwrap();
    let review = document
        .review_effect_template(&file, &scene, &ids)
        .unwrap();
    document.apply_effect_template(review).unwrap();
    let source = raw(&document)["lighting"]["scenes"][0]["effects"][0]["templateSource"].clone();
    let mut effect = raw(&document)["lighting"]["scenes"][0]["effects"][0].clone();
    effect["waveform"] = json!("smooth");
    effect["channels"] = json!([{"attribute":"dimmer","low":0,"high":1000}]);
    edit(
        &mut document,
        json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":effect}}),
    );
    let mut edited = raw(&document);
    assert_eq!(
        edited["lighting"]["scenes"][0]["effects"][0]["templateSource"],
        source
    );
    edited["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|cap| cap["key"] != "lighting.effects.template-keyframes");
    assert!(
        Document::decode(&serde_json::to_vec(&edited).unwrap())
            .unwrap_err()
            .contains("能力声明")
    );
}

#[test]
fn hold_edges_retain_the_existing_no_early_fixed_point_boundary() {
    let mut value = keyframe_json();
    value["definition"]["recipe"]["keyframes"] = json!([
        {"position":0,"value":0,"transition":"hold"},
        {"position":2000,"value":65535,"transition":"hold"}
    ]);
    let file = decode(&value).unwrap();
    let (mut document, scene, ids) = setup();
    let review = document
        .review_effect_template(&file, &scene, &ids)
        .unwrap();
    document.apply_effect_template(review).unwrap();
    let compiled = document.compile_scene(&scene).unwrap();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    // 20% is rounded up to the next fixed-point phase in the existing compiler.
    // The template must not shift the edge earlier or introduce another sampler.
    for (time, expected) in [(199, 0u16), (200, 0), (201, 65535), (999, 65535), (1000, 0)] {
        player.advance(time).unwrap();
        let output = compiled.output.render(player.values()).unwrap();
        assert_eq!(
            u16::from_be_bytes([output.slots[103], output.slots[101]]),
            expected
        );
    }
}
