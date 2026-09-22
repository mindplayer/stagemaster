use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand, ValueMode};

fn with_fixture() -> Document {
    let mut doc = Document::new("剧场灯光").unwrap();
    let view = doc.view();
    doc.edit(EditCommand::AddFixture {
        name: "面光 1".into(),
        profile_id: view.profiles[1].id.clone(),
        domain_id: view.domains[0].id.clone(),
        universe: 1,
        address: 1,
    })
    .unwrap();
    doc
}
fn value(doc: &Document) -> Value {
    serde_json::from_slice(&doc.encode().unwrap()).unwrap()
}
fn reject(mut root: Value, pointer: &str, next: Value) {
    *root.pointer_mut(pointer).unwrap() = next;
    assert!(
        Document::decode(&serde_json::to_vec(&root).unwrap()).is_err(),
        "must reject {pointer}"
    );
}
#[test]
fn blank_project_contains_no_fixtures_or_scenes_and_has_unique_identity() {
    let a = Document::new("工程").unwrap();
    let b = Document::new("工程").unwrap();
    assert!(a.view().fixtures.is_empty());
    assert!(a.view().scenes.is_empty());
    assert_ne!(a.view().id, b.view().id);
    assert_eq!(a, Document::decode(&a.encode().unwrap()).unwrap());
}
#[test]
fn strict_reader_rejects_duplicate_keys_precision_depth_encoding_and_size() {
    for bytes in [
        br#"{"id":1,"\u0069d":2}"#.to_vec(),
        vec![0xff],
        vec![b' '; stagemaster_project::MAX_BYTES + 1],
        format!("{}0{}", "[".repeat(66), "]".repeat(66)).into_bytes(),
        b"{\"n\":9007199254740993}".to_vec(),
        b"{} trailing".to_vec(),
        b"\xef\xbb\xbf{}".to_vec(),
    ] {
        assert!(Document::decode(&bytes).is_err());
    }
}
#[test]
fn schema_and_capability_boundary_reject_unknown_fields_versions_and_modules() {
    let root = value(&with_fixture());
    reject(
        root.clone(),
        "/project/parentRevisionIds",
        json!([root["project"]["revisionId"]]),
    );
    for (path, next) in [
        ("/formatVersion", json!("future")),
        ("/semanticsVersion", json!("future")),
        ("/requires/0/version", json!(2)),
        ("/lighting/fixtures/0/id", json!("bad-id")),
        ("/lighting/fixtures/0/name", json!("  ")),
    ] {
        reject(root.clone(), path, next);
    }
    let mut unknown = root.clone();
    unknown["future"] = json!({});
    assert!(Document::decode(&serde_json::to_vec(&unknown).unwrap()).is_err());
    let mut capability = root;
    capability["requires"] = json!([]);
    assert!(Document::decode(&serde_json::to_vec(&capability).unwrap()).is_err());
    assert!(
        Document::decode(include_bytes!(
            "../../../docs/project-format/examples/escape-room.project.json"
        ))
        .is_err()
    );
    assert!(
        Document::decode(include_bytes!(
            "../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .is_err()
    );
}
#[test]
fn patch_collisions_and_end_overflow_are_atomic() {
    let mut doc = with_fixture();
    let before = doc.clone();
    let view = doc.view();
    for address in [1, 4, 510, 512] {
        assert!(
            doc.edit(EditCommand::AddFixture {
                name: "冲突".into(),
                profile_id: view.profiles[1].id.clone(),
                domain_id: view.domains[0].id.clone(),
                universe: 1,
                address
            })
            .is_err()
        );
        assert_eq!(doc, before);
    }
    doc.edit(EditCommand::AddFixture {
        name: "第二路".into(),
        profile_id: view.profiles[1].id.clone(),
        domain_id: view.domains[0].id.clone(),
        universe: 2,
        address: 1,
    })
    .unwrap();
    doc.edit(EditCommand::UpdateFixture {
        id: view.fixtures[0].id.clone(),
        name: "新名称".into(),
        universe: 1,
        address: 509,
    })
    .unwrap();
    assert_eq!(doc.view().fixtures[0].address, Some(509));
}
#[test]
fn identities_profile_mapping_and_target_types_are_checked() {
    let mut doc = with_fixture();
    doc.edit(EditCommand::AddScene {
        name: "开场".into(),
    })
    .unwrap();
    let root = value(&doc);
    for (path, next) in [
        ("/lighting/fixtures/0/id", root["project"]["id"].clone()),
        (
            "/lighting/fixtures/0/profileId",
            root["domains"][0]["id"].clone(),
        ),
        ("/lighting/profiles/1/channels/0/offsets/0", json!(5)),
        ("/lighting/profiles/1/channels/1/offsets/0", json!(0)),
        (
            "/lighting/scenes/0/assignments/0/target/attribute",
            json!("unknown"),
        ),
        (
            "/lighting/scenes/0/assignments/0/source/value/value",
            json!(65536),
        ),
    ] {
        reject(root.clone(), path, next);
    }
    let mut duplicate = root;
    let entry = duplicate["lighting"]["scenes"][0]["assignments"][0].clone();
    duplicate["lighting"]["scenes"][0]["assignments"]
        .as_array_mut()
        .unwrap()
        .push(entry);
    assert!(Document::decode(&serde_json::to_vec(&duplicate).unwrap()).is_err());
}
#[test]
fn scene_changes_reopen_and_preserve_reference_identity() {
    let mut doc = with_fixture();
    doc.edit(EditCommand::AddScene {
        name: "开场".into(),
    })
    .unwrap();
    let view = doc.view();
    let scene_id = view.scenes[0].id.clone();
    let fixture_id = view.fixtures[0].id.clone();
    doc.edit(EditCommand::SetSceneValue {
        scene_id: scene_id.clone(),
        fixture_id: fixture_id.clone(),
        attribute: "red".into(),
        mode: ValueMode::Literal,
        value: 47000,
    })
    .unwrap();
    doc.edit(EditCommand::RenameScene {
        id: scene_id.clone(),
        name: "新开场".into(),
    })
    .unwrap();
    let reopened = Document::decode(&doc.encode().unwrap()).unwrap();
    assert_eq!(reopened, doc);
    assert_eq!(reopened.view().scenes[0].id, scene_id);
    assert_eq!(reopened.view().scenes[0].values[1].value, Some(47000));
    let before = doc.clone();
    assert!(
        doc.edit(EditCommand::RemoveFixture { id: fixture_id })
            .is_err()
    );
    assert_eq!(doc, before);
}
#[test]
fn preset_references_survive_rename_and_revision_without_being_baked() {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["lighting"]["sequences"] = json!([]);
    root["entryPoints"] = json!([]);
    let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let scene_id = doc.view().scenes[0].id.clone();
    doc.edit(EditCommand::RenameScene {
        id: scene_id,
        name: "入场蓝色".into(),
    })
    .unwrap();
    let saved = doc.next_revision();
    let new = value(&saved);
    assert_eq!(
        new["lighting"]["scenes"][0]["assignments"][3]["source"],
        root["lighting"]["scenes"][0]["assignments"][3]["source"]
    );
    assert_eq!(
        new["project"]["parentRevisionIds"][0],
        root["project"]["revisionId"]
    );
    assert_ne!(new["project"]["revisionId"], root["project"]["revisionId"]);
    assert_eq!(saved.view().scenes[0].values[3].value, Some(65535));
    assert!(saved.same_content(&doc));
}
#[test]
fn release_is_distinct_from_zero_and_clearing_last_assignment_is_rejected() {
    let mut doc = with_fixture();
    doc.edit(EditCommand::AddScene {
        name: "场景".into(),
    })
    .unwrap();
    let view = doc.view();
    let command = |mode| EditCommand::SetSceneValue {
        scene_id: view.scenes[0].id.clone(),
        fixture_id: view.fixtures[0].id.clone(),
        attribute: "dimmer".into(),
        mode,
        value: 0,
    };
    doc.edit(command(ValueMode::Release)).unwrap();
    assert_eq!(doc.view().scenes[0].values[0].mode, "release");
    assert_eq!(doc.view().scenes[0].values[0].value, None);
    doc.edit(command(ValueMode::Literal)).unwrap();
    assert_eq!(doc.view().scenes[0].values[0].value, Some(0));
    for key in ["red", "green", "blue"] {
        doc.edit(EditCommand::SetSceneValue {
            scene_id: view.scenes[0].id.clone(),
            fixture_id: view.fixtures[0].id.clone(),
            attribute: key.into(),
            mode: ValueMode::Remove,
            value: 0,
        })
        .unwrap();
    }
    let before = doc.clone();
    assert!(doc.edit(command(ValueMode::Remove)).is_err());
    assert_eq!(doc, before);
}

#[test]
fn first_save_has_no_fictional_parent_revision() {
    let draft = with_fixture();
    let first = draft.next_revision();
    let first_json = value(&first);
    assert_eq!(first_json["project"]["parentRevisionIds"], json!([]));
    let second = first.next_revision();
    assert_eq!(
        value(&second)["project"]["parentRevisionIds"],
        json!([first_json["project"]["revisionId"]])
    );
}

#[test]
fn integer_decimal_spelling_keeps_profile_and_patch_values() {
    let document = with_fixture();
    let input = String::from_utf8(document.encode().unwrap())
        .unwrap()
        .replace("\"footprint\": 4", "\"footprint\": 4.0")
        .replace("\"address\": 1", "\"address\": 1e0");
    let reopened = Document::decode(input.as_bytes()).unwrap();
    assert_eq!(reopened.view().fixtures[0].footprint, 4);
    assert_eq!(reopened.view().fixtures[0].address, Some(1));
    assert!(document.same_content(&reopened));
}
