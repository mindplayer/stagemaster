use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{Document, EffectTemplateFile, MAX_EFFECT_TEMPLATE_BYTES};
fn edit(d: &mut Document, command: Value) {
    d.edit(serde_json::from_value(command).unwrap()).unwrap();
}
fn setup() -> (Document, String, Vec<String>) {
    let mut d = Document::new("模板绑定").unwrap();
    // Different native slot locations and resolutions, with unrelated static RGB.
    for (name, coarse, fine, address) in [("8 位帕灯", 1, None, 11), ("16 位射灯", 4, Some(2), 101)]
    {
        edit(
            &mut d,
            json!({"op":"fixture","command":{"op":"saveProfile","id":null,"definition":{
                "name":name,"manufacturer":"测试","model":name,"mode":"六通道","footprint":6,
                "channels":[{"attribute":"dimmer","coarse":coarse,"fine":fine,"defaultValue":0},
                {"attribute":"red","coarse":3,"defaultValue":12336},
            {"attribute":"green","coarse":5,"defaultValue":0},
            {"attribute":"blue","coarse":6,"defaultValue":0}]
            }}}),
        );
        let v = d.view();
        edit(
            &mut d,
            json!({"op":"addFixture","name":name,"profileId":v.profiles.last().unwrap().id,
            "domainId":v.domains[0].id,"universe":1,"address":address}),
        );
    }
    edit(&mut d, json!({"op":"addScene","name":"场景"}));
    let v = d.view();
    (
        d,
        v.scenes[0].id.clone(),
        v.fixtures.iter().map(|f| f.id.clone()).collect(),
    )
}
fn template() -> EffectTemplateFile {
    EffectTemplateFile::decode(&serde_json::to_vec(&json!({
        "format":"stagemaster-effect-template","formatVersion":1,
        "templateId":"19999999-0000-4000-8000-000000000001",
        "revision":"19999999-0000-4000-8000-000000000002",
        "definition":{"name":"跨型号亮度呼吸","recipe":{
            "kind":"intensity-wave","waveform":"triangle","low":2570,"high":51400,"dutyPercent":50},
            "timing":{"periodMs":1000,"phaseDegrees":0,"spreadDegrees":0,"reverseOrder":false}}
    })).unwrap()).unwrap()
}
fn raw(d: &Document) -> Value {
    serde_json::from_slice(&d.encode().unwrap()).unwrap()
}

#[test]
fn same_template_binds_different_native_slots_without_carrying_project_identity() {
    let (mut d, scene, ids) = setup();
    let before = raw(&d);
    let file = template();
    let review = d.review_effect_template(&file, &scene, &ids).unwrap();
    assert_eq!(raw(&d), before); // inspection/cancellation has no mutation
    assert_eq!(review.view().effect.fixture_ids, ids);
    assert_eq!(review.view().usage.effect_channels, 2);
    d.apply_effect_template(review).unwrap();
    let c = d.compile_scene(&scene).unwrap();
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    for (time, expected) in [(0, 2570u16), (500, 51400), (1000, 2570)] {
        p.advance(time).unwrap();
        let out = c.output.render(p.values()).unwrap();
        assert_eq!(u16::from(out.slots[10]), expected >> 8);
        assert_eq!(
            u16::from_be_bytes([out.slots[103], out.slots[101]]),
            expected
        );
        assert_eq!(out.slots[12], 48);
        assert_eq!(out.slots[102], 48);
    }
    let after = raw(&d);
    assert_eq!(
        after["lighting"]["scenes"][0]["assignments"],
        before["lighting"]["scenes"][0]["assignments"]
    );
    assert_eq!(
        after["lighting"]["profiles"],
        before["lighting"]["profiles"]
    );
    assert_eq!(after["lighting"]["patches"], before["lighting"]["patches"]);
    assert_eq!(d, Document::decode(&d.encode().unwrap()).unwrap());
    let e = d.view().scenes[0].effects[0].id.clone();
    let exported = d.effect_template_file(&scene, &e).unwrap();
    assert_ne!(exported.template().template_id, file.template().template_id);
    let exported_text = String::from_utf8(exported.encode().unwrap()).unwrap();
    for id in ids.iter().chain([&scene]) {
        assert!(!exported_text.contains(id));
    }
    assert!(!exported_text.contains("fixtureIds"));
    assert!(!exported_text.contains("address"));
    assert_eq!(
        serde_json::to_value(&file.template().definition).unwrap(),
        serde_json::to_value(&exported.template().definition).unwrap()
    );
}
#[test]
fn unsaved_changes_invalidate_review_and_conflicting_or_missing_targets_are_atomic() {
    let (mut d, scene, ids) = setup();
    let file = template();
    for selected in [
        vec![],
        vec![ids[0].clone(), ids[0].clone()],
        vec!["missing".into()],
        vec![ids[0].clone(); 513],
    ] {
        let before = d.clone();
        assert!(d.review_effect_template(&file, &scene, &selected).is_err());
        assert_eq!(d, before);
    }
    let review = d.review_effect_template(&file, &scene, &ids).unwrap();
    let revision = raw(&d)["project"]["revisionId"].clone();
    edit(
        &mut d,
        json!({"op":"renameScene","id":scene,"name":"改变了但没保存"}),
    );
    assert_eq!(raw(&d)["project"]["revisionId"], revision);
    let before = d.clone();
    assert!(
        d.apply_effect_template(review)
            .unwrap_err()
            .contains("重新检查")
    );
    assert_eq!(d, before);
    let review = d.review_effect_template(&file, &scene, &ids).unwrap();
    d.apply_effect_template(review).unwrap();
    let before = d.clone();
    assert!(
        d.review_effect_template(&file, &scene, &ids)
            .err()
            .unwrap()
            .contains("同一灯具属性")
    );
    assert_eq!(d, before);
    let mut no_dim = raw(&setup().0);
    for p in no_dim["lighting"]["profiles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .skip(2)
    {
        p["channels"]
            .as_array_mut()
            .unwrap()
            .retain(|c| c["attribute"] != "dimmer");
        p["attributes"]
            .as_array_mut()
            .unwrap()
            .retain(|a| a["key"] != "dimmer");
    }
    no_dim["lighting"]["scenes"][0]["assignments"]
        .as_array_mut()
        .unwrap()
        .retain(|a| a["target"]["attribute"] != "dimmer");
    let d = Document::decode(&serde_json::to_vec(&no_dim).unwrap()).unwrap();
    let v = d.view();
    let err = d
        .review_effect_template(&file, &v.scenes[0].id, &[v.fixtures[0].id.clone()])
        .err()
        .unwrap();
    assert!(err.contains("8 位帕灯") && err.contains("亮度模板"));
}
#[test]
fn imported_origin_is_immutable_history_not_a_live_template_dependency() {
    let (mut d, scene, ids) = setup();
    let file = template();
    let review = d.review_effect_template(&file, &scene, &ids).unwrap();
    d.apply_effect_template(review).unwrap();
    let origin = raw(&d)["lighting"]["scenes"][0]["effects"][0]["templateSource"].clone();
    let before = d.clone();
    let mut detached = serde_json::to_value(&d.view().scenes[0].effects[0]).unwrap();
    detached.as_object_mut().unwrap().remove("templateSource");
    assert!(
        d.edit(
            serde_json::from_value(
                json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":detached}})
            )
            .unwrap()
        )
        .unwrap_err()
        .contains("来源快照")
    );
    assert_eq!(d, before);

    let mut effect = serde_json::to_value(&d.view().scenes[0].effects[0]).unwrap();
    effect["periodMs"] = json!(2000);
    edit(
        &mut d,
        json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":effect}}),
    );
    assert_eq!(
        raw(&d)["lighting"]["scenes"][0]["effects"][0]["templateSource"],
        origin
    );
    edit(
        &mut d,
        json!({"op":"duplicateScene","id":scene,"name":"复制"}),
    );
    let v = d.view();
    assert_ne!(v.scenes[0].effects[0].id, v.scenes[1].effects[0].id);
    assert_eq!(
        raw(&d)["lighting"]["scenes"][1]["effects"][0]["templateSource"],
        origin
    );
    assert_eq!(Document::decode(&d.encode().unwrap()).unwrap(), d);
    let before = raw(&d);
    for mutate in [0, 1, 2] {
        let mut invalid = before.clone();
        if mutate == 0 {
            invalid["lighting"]["scenes"][0]["effects"][0]["templateSource"]["template"]["definition"]
                ["timing"]["periodMs"] = json!(3000);
        } else if mutate == 1 {
            invalid["requires"]
                .as_array_mut()
                .unwrap()
                .retain(|r| r["key"] != "lighting.effects.template-source");
        } else {
            invalid["lighting"]["scenes"][0]["effects"][0]["templateSource"]["path"] =
                json!("/private/template");
        }
        assert!(Document::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
    }
}
#[test]
fn strict_portable_codec_rejects_smuggling_and_digest_ignores_key_order_and_spacing() {
    let file = template();
    let original: Value = serde_json::from_slice(&file.encode().unwrap()).unwrap();
    let compact = serde_json::to_vec(&original).unwrap();
    assert_eq!(
        file.source().sha256,
        EffectTemplateFile::decode(&compact)
            .unwrap()
            .source()
            .sha256
    );
    let cases = [
        ("/formatVersion", json!(2)),
        ("/templateId", json!("not-id")),
        ("/definition/name", json!("   ")),
        ("/definition/recipe/kind", json!("laser-program")),
        ("/definition/recipe/low", json!(-1)),
        ("/definition/recipe/high", json!(65536)),
        ("/definition/recipe/dutyPercent", json!(100)),
        ("/definition/timing/periodMs", json!(99)),
        ("/definition/timing/spreadDegrees", json!(361)),
        ("/definition/timing/phaseDegrees", json!(360)),
    ];
    for (path, value) in cases {
        let mut invalid = original.clone();
        *invalid.pointer_mut(path).unwrap() = value;
        assert!(
            EffectTemplateFile::decode(&serde_json::to_vec(&invalid).unwrap()).is_err(),
            "{path}"
        );
    }
    let mut injected = original.clone();
    injected["definition"]["fixtureIds"] = json!(["some fixture"]);
    assert!(EffectTemplateFile::decode(&serde_json::to_vec(&injected).unwrap()).is_err());
    let text = String::from_utf8(compact).unwrap().replacen(
        "\"formatVersion\":1",
        "\"formatVersion\":2,\"formatVersion\":1",
        1,
    );
    assert!(
        EffectTemplateFile::decode(text.as_bytes())
            .unwrap_err()
            .contains("重复")
    );
    let mut oversized = file.encode().unwrap();
    oversized.resize(MAX_EFFECT_TEMPLATE_BYTES + 1, b' ');
    assert!(
        EffectTemplateFile::decode(&oversized)
            .unwrap_err()
            .contains("16 KiB")
    );
}
#[test]
fn ordered_phase_and_disabled_export_preserve_semantics_without_baking_fixture_order() {
    let (mut document, scene, mut ids) = setup();
    ids.reverse();
    let mut raw_template: Value = serde_json::from_slice(&template().encode().unwrap()).unwrap();
    raw_template["definition"]["recipe"]["waveform"] = json!("pulse");
    raw_template["definition"]["recipe"]["low"] = json!(0);
    raw_template["definition"]["recipe"]["high"] = json!(65535);
    raw_template["definition"]["recipe"]["dutyPercent"] = json!(25);
    raw_template["definition"]["timing"]["spreadDegrees"] = json!(360);
    let file = EffectTemplateFile::decode(&serde_json::to_vec(&raw_template).unwrap()).unwrap();
    let review = document
        .review_effect_template(&file, &scene, &ids)
        .unwrap();
    document.apply_effect_template(review).unwrap();
    let compiled = document.compile_scene(&scene).unwrap();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    let out = compiled.output.render(player.values()).unwrap();
    assert_eq!(out.slots[103], 255);
    assert_eq!(out.slots[10], 0);
    player.advance(500).unwrap();
    let out = compiled.output.render(player.values()).unwrap();
    assert_eq!(out.slots[103], 0);
    assert_eq!(out.slots[10], 255);
    let mut effect = serde_json::to_value(&document.view().scenes[0].effects[0]).unwrap();
    effect["enabled"] = json!(false);
    edit(
        &mut document,
        json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":effect}}),
    );
    let file = document
        .effect_template_file(&scene, effect["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        serde_json::to_value(&file.template().definition).unwrap(),
        raw_template["definition"]
    );
    effect["channels"][0]["attribute"] = json!("red");
    edit(
        &mut document,
        json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":effect}}),
    );
    assert!(
        document
            .effect_template_file(&scene, effect["id"].as_str().unwrap())
            .is_err()
    );
}

#[test]
fn scene_effect_capacity_is_checked_without_modifying_the_project() {
    let (mut d, scene, ids) = setup();
    let file = template();
    let candidate = d.review_effect_template(&file, &scene, &ids).unwrap();
    let mut e = serde_json::to_value(&candidate.view().effect).unwrap();
    e["enabled"] = json!(false);
    for _ in 0..32 {
        e["id"] = json!(uuid::Uuid::new_v4().to_string());
        edit(
            &mut d,
            json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":e}}),
        );
    }
    let before = d.clone();
    assert!(d.review_effect_template(&file, &scene, &ids).is_err());
    assert_eq!(d, before);
}

#[test]
fn same_template_revision_cannot_mean_different_content_even_with_valid_checksums() {
    let (mut d, scene, ids) = setup();
    let file = template();
    let review = d
        .review_effect_template(&file, &scene, &[ids[0].clone()])
        .unwrap();
    d.apply_effect_template(review).unwrap();
    let mut altered: Value = serde_json::from_slice(&file.encode().unwrap()).unwrap();
    altered["definition"]["timing"]["periodMs"] = json!(3000);
    let conflicting = EffectTemplateFile::decode(&serde_json::to_vec(&altered).unwrap()).unwrap();
    let before = d.clone();
    assert!(
        d.review_effect_template(&conflicting, &scene, &[ids[1].clone()])
            .err()
            .unwrap()
            .contains("不同内容")
    );
    assert_eq!(d, before);
    altered["revision"] = json!(uuid::Uuid::new_v4().to_string());
    let revised = EffectTemplateFile::decode(&serde_json::to_vec(&altered).unwrap()).unwrap();
    let review = d
        .review_effect_template(&revised, &scene, &[ids[1].clone()])
        .unwrap();
    d.apply_effect_template(review).unwrap();
    assert_eq!(d.view().scenes[0].effects.len(), 2);
}
