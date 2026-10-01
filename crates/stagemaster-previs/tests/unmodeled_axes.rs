use serde_json::json;
use stagemaster_project::{Document, EditCommand};

#[test]
fn a_placed_unmodeled_head_is_named_in_preview_error_instead_of_drawn_as_fixed() {
    let mut doc = Document::new("未定义几何").unwrap();
    let commands = [
        json!({"op":"fixture","command":{"op":"saveProfile","definition":{"name":"未知角度","manufacturer":"自定义","model":"待测","mode":"5CH","footprint":5,"channels":[{"attribute":"dimmer","coarse":5,"fine":null,"defaultValue":0},{"attribute":"pan","coarse":1,"fine":2,"defaultValue":32768},{"attribute":"tilt","coarse":3,"fine":4,"defaultValue":32768}]}}}),
    ];
    for c in commands {
        doc.edit(serde_json::from_value(c).unwrap()).unwrap();
    }
    let v = doc.view();
    doc.edit(EditCommand::AddFixture {
        name: "大厅左侧摇头".into(),
        profile_id: v.profiles.last().unwrap().id.clone(),
        domain_id: v.domains[0].id.clone(),
        universe: 1,
        address: 1,
    })
    .unwrap();
    assert!(stagemaster_previs::scene(&doc).unwrap().fixtures.is_empty());
    let f = doc.view().fixtures[0].id.clone();
    doc.edit(serde_json::from_value(json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":f,"spaceId":null,"positionMeters":{"x":"2","y":"3","z":"4"},"rotationDegreesXYZ":{"x":"0","y":"0","z":"0"}}}})).unwrap()).unwrap();
    let before = doc.encode().unwrap();
    let err = stagemaster_previs::scene(&doc).err().unwrap();
    assert!(err.contains("大厅左侧摇头") && err.contains("轴行程"));
    assert_eq!(doc.encode().unwrap(), before);
}
