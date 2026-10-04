use serde_json::{Value, json};
use stagemaster_project::{
    Document, EditCommand, FixtureEdit, MAX_PROFILE_FILE_BYTES, ProfileFile,
};

fn fixture() -> Document {
    let mut d = Document::new("模式往返").unwrap();
    let definition = json!({"name":"中文摇头 · 16 位","manufacturer":"测试厂商","model":"旧款","mode":"十通道","footprint":10,
      "channels":[
        {"attribute":"dimmer","coarse":2,"fine":1,"defaultValue":4660},
        {"attribute":"pan","coarse":4,"fine":3,"defaultValue":32767},
        {"attribute":"tilt","coarse":6,"fine":5,"defaultValue":12345},
        {"attribute":"shutter","coarse":7,"fine":8,"defaultValue":{"functionKey":"strobe","position":23456},"functions":[
          {"key":"open","name":"开光","mode":"slot","dmxFrom":0,"dmxTo":511,"dmxDefault":257},
          {"key":"strobe","name":"受控频闪","mode":"range","dmxFrom":1024,"dmxTo":65535,"dmxDefault":32768}]}],
      "positioning":{"kind":"intersectingOrthogonal","pan":{"minDegrees":"-270","maxDegrees":"270","reversed":true},"tilt":{"minDegrees":"-135","maxDegrees":"135","reversed":false}}});
    d.edit(serde_json::from_value(json!({"op":"fixture","command":{"op":"saveProfile","id":null,"definition":definition}})).unwrap()).unwrap();
    let p = d.view();
    d.edit(EditCommand::AddFixture {
        name: "原有灯".into(),
        profile_id: p.profiles.last().unwrap().id.clone(),
        domain_id: p.domains[0].id.clone(),
        universe: 1,
        address: 1,
    })
    .unwrap();
    d.edit(EditCommand::AddScene {
        name: "默认场景".into(),
    })
    .unwrap();
    d
}
fn file(d: &Document) -> ProfileFile {
    d.profile_file(&d.view().profiles.last().unwrap().id)
        .unwrap()
}
fn value(d: &Document) -> Value {
    serde_json::from_slice(&file(d).encode().unwrap()).unwrap()
}
#[test]
fn full_mode_roundtrip_keeps_coarse_fine_defaults_functions_and_mechanics() {
    let source = fixture();
    let before = source.encode().unwrap();
    let f = file(&source);
    let decoded = ProfileFile::decode(&f.encode().unwrap()).unwrap();
    assert_eq!(f.encode().unwrap(), decoded.encode().unwrap());
    assert_eq!(decoded.definition().channels[0].coarse, 2);
    assert_eq!(decoded.definition().channels[0].fine, Some(1));
    let mut target = Document::new("新工程").unwrap();
    target
        .edit(EditCommand::Fixture {
            command: FixtureEdit::SaveProfile {
                id: None,
                definition: Box::new(decoded.definition().clone()),
            },
        })
        .unwrap();
    let imported = file(&target);
    assert_ne!(imported.source().profile_id, f.source().profile_id);
    assert_ne!(imported.source().revision, f.source().revision);
    assert_eq!(
        serde_json::to_value(imported.definition()).unwrap(),
        serde_json::to_value(f.definition()).unwrap()
    );
    assert!(target.view().fixtures.is_empty());
    assert!(target.view().scenes.is_empty());
    assert_eq!(before, source.encode().unwrap());
}
#[test]
fn importing_same_named_mode_leaves_original_fixtures_and_output_unchanged() {
    let mut d = fixture();
    let original = d.view();
    let scene = &original.scenes[0].id;
    let before = d.compile_scene(scene).unwrap();
    let file = file(&d);
    d.edit(EditCommand::Fixture {
        command: FixtureEdit::SaveProfile {
            id: None,
            definition: Box::new(file.definition().clone()),
        },
    })
    .unwrap();
    let view = d.view();
    assert_eq!(view.fixtures[0].profile_id, original.fixtures[0].profile_id);
    assert_eq!(view.profiles.len(), original.profiles.len() + 1);
    assert_eq!(
        view.profiles.last().unwrap().name,
        original.profiles.last().unwrap().name
    );
    assert_eq!(view.profiles[2].revision, original.profiles[2].revision);
    assert_eq!(before.plan, d.compile_scene(scene).unwrap().plan);
}
#[test]
fn unknown_versions_fields_bad_channel_functions_and_budgets_are_rejected() {
    let source = fixture();
    let original = value(&source);
    let cases = [
        ("/formatVersion", json!(2)),
        ("/format", json!("gdtf")),
        ("/definition/channels/0/coarse", json!(7)),
        ("/definition/channels/3/functions/1/dmxFrom", json!(400)),
        ("/definition/positioning/pan/maxDegrees", json!("-300")),
        ("/source/profileId", json!("invalid")),
    ];
    for (pointer, invalid) in cases {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = invalid;
        assert!(
            ProfileFile::decode(&serde_json::to_vec(&changed).unwrap()).is_err(),
            "{pointer}"
        );
    }
    for pointer in [
        "",
        "/source",
        "/definition",
        "/definition/channels/0",
        "/definition/channels/3/functions/0",
    ] {
        let mut changed = original.clone();
        changed.pointer_mut(pointer).unwrap()["unexpected"] = json!(true);
        assert!(
            ProfileFile::decode(&serde_json::to_vec(&changed).unwrap()).is_err(),
            "{pointer}"
        );
    }
    assert!(ProfileFile::decode(&vec![b' '; MAX_PROFILE_FILE_BYTES + 1]).is_err());
    assert!(ProfileFile::decode(b"{broken").is_err());
    assert!(ProfileFile::decode(&source.encode().unwrap()).is_err());
}
#[test]
fn export_refuses_lossy_legacy_metadata_instead_of_dropping_it() {
    let source = Document::new("原档案").unwrap();
    assert!(source.profile_file(&source.view().profiles[0].id).is_ok());
    let mut raw: Value = serde_json::from_slice(&source.encode().unwrap()).unwrap();
    raw["lighting"]["profiles"][0]["name"] = json!(" 有空格的原名 ");
    let legacy = Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert!(
        legacy
            .profile_file(&legacy.view().profiles[0].id)
            .err()
            .unwrap()
            .contains("完整保留")
    );
}
