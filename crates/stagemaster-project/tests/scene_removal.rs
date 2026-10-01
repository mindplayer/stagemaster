use serde_json::json;
use stagemaster_project::{Document, EditCommand};

fn edit(doc: &mut Document, value: serde_json::Value) -> Result<(), String> {
    doc.edit(serde_json::from_value::<EditCommand>(value).unwrap())
}

#[test]
fn mixed_scene_removal_preserves_entire_document_when_any_reference_remains() {
    let mut doc = Document::new("删除验收").unwrap();
    let view = doc.view();
    doc.edit(EditCommand::AddFixture {
        name: "验收灯具".into(),
        profile_id: view.profiles[1].id.clone(),
        domain_id: view.domains[0].id.clone(),
        universe: 1,
        address: 1,
    })
    .unwrap();
    for name in ["被使用", "未使用甲", "未使用乙"] {
        edit(&mut doc, json!({"op":"addScene","name":name})).unwrap();
    }
    let ids: Vec<_> = doc.view().scenes.iter().map(|s| s.id.clone()).collect();
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"演出","sceneId":ids[0]}}),
    )
    .unwrap();
    let original = doc.clone();
    assert!(
        edit(
            &mut doc,
            json!({"op":"batch","commands":[
                {"op":"removeScene","id":ids[1]},
                {"op":"removeScene","id":ids[0]}
            ]})
        )
        .is_err()
    );
    assert_eq!(doc, original);
    edit(
        &mut doc,
        json!({"op":"batch","commands":[
            {"op":"removeScene","id":ids[2]},
            {"op":"removeScene","id":ids[1]}
        ]}),
    )
    .unwrap();
    assert_eq!(doc.view().scenes.len(), 1);
    assert_eq!(doc.view().scenes[0].id, ids[0]);
    assert_eq!(
        serde_json::to_value(doc.view().sequences).unwrap(),
        serde_json::to_value(original.view().sequences).unwrap()
    );
}
