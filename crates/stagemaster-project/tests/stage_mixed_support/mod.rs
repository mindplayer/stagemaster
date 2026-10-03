use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
pub fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Stage {
        command: serde_json::from_value(command).unwrap(),
    })
}
pub fn root(doc: &Document) -> Value {
    serde_json::from_slice(&doc.encode().unwrap()).unwrap()
}
pub fn setup() -> (Document, Vec<Value>, Vec<String>) {
    let mut doc = Document::new("混合场地").unwrap();
    let view = doc.view();
    for i in 0..3 {
        doc.edit(EditCommand::AddFixture {
            name: format!("灯{i}"),
            profile_id: view.profiles[0].id.clone(),
            domain_id: view.domains[0].id.clone(),
            universe: 1,
            address: 1 + i * 4,
        })
        .unwrap();
    }
    let ids = doc
        .view()
        .fixtures
        .iter()
        .map(|f| f.id.clone())
        .collect::<Vec<_>>();
    for (i, id) in ids.iter().enumerate() {
        edit(&mut doc, json!({"op":"putPlacement","placement":{"fixtureId":id,"spaceId":null,"positionMeters":{"x":i.to_string(),"y":"2.125123","z":(4+i).to_string()},"rotationDegreesXYZ":{"x":"180","y":"7","z":"35"}}})).unwrap();
    }
    let seating: Value = serde_json::from_str(include_str!(
        "../../../../tools/test-data/seating-layout.json"
    ))
    .unwrap();
    for shape in [
        json!({"kind":"rig","rigKind":"truss","spaceId":null,"positionMeters":{"x":"1","y":"2.123456","z":"6"},"yawDegrees":"15","lengthMeters":"3","widthMeters":"0.3","heightMeters":"0.3"}),
        json!({"kind":"platform","spaceId":null,"outlineMeters":[["0","0"],["3","0"],["3","2"],["0","2"]],"baseElevationMeters":"0.5","heightMeters":"0.3"}),
        seating["shape"].clone(),
    ] {
        edit(
            &mut doc,
            json!({"op":"putConstruction","id":null,"name":"构件","shape":shape}),
        )
        .unwrap();
    }
    let constructions = doc.view().stage.constructions;
    edit(&mut doc,json!({"op":"attachFixtures","constructionId":constructions[0].id,"fixtureIds":ids[..2],"layout":null})).unwrap();
    let mut targets = constructions
        .iter()
        .map(|c| json!({"kind":"construction","targetId":c.id}))
        .collect::<Vec<_>>();
    targets.push(json!({"kind":"placement","targetId":ids[0]}));
    (doc, targets, ids)
}
pub fn translate(doc: &mut Document, targets: &[Value], xyz: [&str; 3]) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"translateObjects","targets":targets,"deltaMeters":{"x":xyz[0],"y":xyz[1],"z":xyz[2]}}),
    )
}
