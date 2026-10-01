use serde_json::{Value, json};
use stagemaster_project::{ConstructionShape, Document, EditCommand};
fn vector() -> Value {
    serde_json::from_str(include_str!("../../../tools/test-data/seating-layout.json")).unwrap()
}
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Stage {
        command: serde_json::from_value(command).unwrap(),
    })
}
fn put(doc: &mut Document, shape: &Value) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"putConstruction","id":null,"name":"观众座区","shape":shape}),
    )
}
#[test]
fn world_geometry_matches_independent_rotated_aisle_vector() {
    let v = vector();
    let ConstructionShape::Seating(s) = serde_json::from_value(v["shape"].clone()).unwrap() else {
        panic!()
    };
    let layout = s.layout().unwrap();
    let expected: Vec<[f64; 2]> = serde_json::from_value(v["centers"].clone()).unwrap();
    assert_eq!(layout.centers.len(), expected.len());
    for (a, b) in layout
        .centers
        .iter()
        .flatten()
        .zip(expected.iter().flatten())
    {
        assert!((a - b).abs() < 1e-10);
    }
    let outline: Vec<[f64; 2]> = serde_json::from_value(v["outline"].clone()).unwrap();
    for (a, b) in layout
        .outline
        .iter()
        .flatten()
        .zip(outline.iter().flatten())
    {
        assert!((a - b).abs() < 1e-10);
    }
    // Independent distance between column 2 and 3 minus chair width is the clear aisle.
    assert!(
        ((layout.centers[1][1] - layout.centers[2][1]).abs() - layout.seat_width - 2.0).abs()
            < 1e-10
    );
}
#[test]
fn invalid_shapes_fail_without_partial_document_or_capability_changes() {
    let doc = Document::new("原子座区").unwrap();
    let bad = [
        ("rows", json!(0)),
        ("rows", json!(65)),
        ("rows", json!(64)),
        ("columns", json!(0)),
        ("seatWidthMeters", json!("0.2")),
        ("seatDepthMeters", json!("1.3")),
        ("columnSpacingMeters", json!("0.4")),
        ("rowSpacingMeters", json!("0.4")),
        ("yawDegrees", json!("NaN")),
        ("positionMeters", json!({"x":"100000","y":"0","z":"0"})),
        ("positionMeters", json!({"x":"0","y":"0","z":"99999.9"})),
        ("aisle", json!({"afterColumn":0,"widthMeters":"2"})),
        ("aisle", json!({"afterColumn":4,"widthMeters":"2"})),
        ("aisle", json!({"afterColumn":2,"widthMeters":"0.3"})),
        ("spaceId", json!("a0000000-0000-4000-8000-000000000001")),
    ];
    for (key, value) in bad {
        let mut next = doc.clone();
        let mut s = vector()["shape"].clone();
        if key == "rows" && value == 64 {
            s["columns"] = json!(9);
        }
        s[key] = value;
        assert!(put(&mut next, &s).is_err(), "accepted {key}");
        assert_eq!(next, doc);
    }
}
#[test]
fn aggregate_budget_is_checked_before_geometry_and_copy_is_atomic() {
    let mut doc = Document::new("有界座区").unwrap();
    let mut s = vector()["shape"].clone();
    s["rows"] = json!(64);
    s["columns"] = json!(8);
    put(&mut doc, &s).unwrap();
    let id = doc.view().stage.constructions[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"duplicateConstruction","id":id,"name":"第二组"}),
    )
    .unwrap();
    let before = doc.clone();
    assert!(
        edit(
            &mut doc,
            json!({"op":"duplicateConstruction","id":id,"name":"第三组"})
        )
        .unwrap_err()
        .contains("1024")
    );
    assert_eq!(doc, before);
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
}
#[test]
fn capability_schema_and_section_lock_are_preserved_on_roundtrip() {
    let mut doc = Document::new("独立身份").unwrap();
    put(&mut doc, &vector()["shape"]).unwrap();
    let id = doc.view().stage.constructions[0].id.clone();
    let encoded = doc.encode().unwrap();
    let root: Value = serde_json::from_slice(&encoded).unwrap();
    assert!(
        root["requires"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["key"] == "stage.seating")
    );
    for mode in 0..4 {
        let mut broken = root.clone();
        match mode {
            0 => broken["requires"]
                .as_array_mut()
                .unwrap()
                .retain(|c| c["key"] != "stage.seating"),
            1 => broken["stage"]["constructions"][0]["shape"]["rows"] = json!(1.5),
            2 => broken["stage"]["constructions"][0]["shape"]["extra"] = json!(true),
            _ => {
                broken["stage"]["constructions"][0]["shape"]
                    .as_object_mut()
                    .unwrap()
                    .remove("aisle");
            }
        }
        assert!(Document::decode(&serde_json::to_vec(&broken).unwrap()).is_err());
    }
    edit(&mut doc,json!({"op":"setEditLocks","targets":[{"kind":"construction","targetId":id}],"locked":true})).unwrap();
    let locked = doc.clone();
    assert!(edit(&mut doc, json!({"op":"removeConstruction","id":id})).is_err());
    assert_eq!(doc, locked);
    edit(
        &mut doc,
        json!({"op":"duplicateConstruction","id":id,"name":"可编辑副本"}),
    )
    .unwrap();
    assert_ne!(doc.view().stage.constructions[1].id, id);
    assert_eq!(doc.view().stage.edit_locks.len(), 1);
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
}
