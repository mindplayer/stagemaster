use serde_json::{Value, json};
use stagemaster_project::{ConstructionShape, Document, EditCommand};
fn vector() -> Value {
    serde_json::from_str(include_str!(
        "../../../tools/test-data/seating-arc-layout.json"
    ))
    .unwrap()
}
fn layout(shape: Value) -> Result<stagemaster_project::SeatingLayout, String> {
    let ConstructionShape::Seating(s) = serde_json::from_value(shape).unwrap() else {
        panic!()
    };
    s.layout()
}
fn put(doc: &mut Document, shape: &Value) -> Result<(), String> {
    doc.edit(serde_json::from_value::<EditCommand>(json!({"op":"stage","command":{"op":"putConstruction","id":null,"name":"弧排","shape":shape}})).unwrap())
}
#[test]
fn six_seats_match_independent_thirty_degree_triangles_and_point_toward_focus() {
    let v = vector();
    let actual = layout(v["shape"].clone()).unwrap();
    let expected: Vec<[f64; 2]> = serde_json::from_value(v["centers"].clone()).unwrap();
    let outline: Vec<[f64; 2]> = serde_json::from_value(v["outline"].clone()).unwrap();
    for (a, b) in actual
        .centers
        .iter()
        .chain(actual.outline.iter())
        .flatten()
        .zip(expected.iter().chain(outline.iter()).flatten())
    {
        assert!((a - b).abs() < 1e-9);
    }
    let focus = actual.focus.unwrap();
    for (i, (center, angle)) in actual
        .centers
        .iter()
        .zip(&actual.seat_yaws_radians)
        .enumerate()
    {
        let direction = [focus[0] - center[0], focus[1] - center[1]];
        let radius = direction[0].hypot(direction[1]);
        assert!((radius - if i < 3 { 4.0 } else { 5.0 }).abs() < 1e-9);
        assert!((direction[0] / radius + angle.sin()).abs() < 1e-9);
        assert!((direction[1] / radius - angle.cos()).abs() < 1e-9);
        assert!((angle.to_degrees() - v["anglesDegrees"][i].as_f64().unwrap()).abs() < 1e-9);
    }
}
#[test]
fn inner_aisle_corners_respect_clear_width_and_widen_in_the_rear() {
    let mut s = vector()["shape"].clone();
    s["columns"] = json!(4);
    s["columnSpacingMeters"] = json!("0.6");
    s["aisle"] = json!({"afterColumn":2,"widthMeters":"1.2"});
    let a = layout(s).unwrap();
    let corner = |i: usize, x: f64, y: f64| {
        let (sin, cos) = a.seat_yaws_radians[i].sin_cos();
        [
            a.centers[i][0] + x * cos - y * sin,
            a.centers[i][1] + x * sin + y * cos,
        ]
    };
    let distance = |i, j| {
        let left = corner(i, 0.25, 0.25);
        let right = corner(j, -0.25, 0.25);
        (left[0] - right[0]).hypot(left[1] - right[1])
    };
    assert!((distance(1, 2) - 1.2).abs() < 1e-9);
    assert!(distance(5, 6) > 1.2);
}
#[test]
fn malformed_geometry_and_missing_capability_never_mutate_a_project() {
    let original = Document::new("弧排校验").unwrap();
    for (field, value) in [
        ("arc", json!({"radiusMeters":"0.9"})),
        ("arc", json!({"radiusMeters":"10001"})),
        ("arc", json!({"radiusMeters":"1"})),
        ("columnSpacingMeters", json!("0.5")),
        ("rowSpacingMeters", json!("0.49")),
        ("positionMeters", json!({"x":"100000","y":"20","z":"0.5"})),
    ] {
        let mut d = original.clone();
        let mut s = vector()["shape"].clone();
        s[field] = value;
        assert!(put(&mut d, &s).is_err(), "accepted {field}");
        assert_eq!(d, original);
    }
    let mut doc = original;
    put(&mut doc, &vector()["shape"]).unwrap();
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
    let root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    for mode in 0..3 {
        let mut bad = root.clone();
        match mode {
            0 => bad["requires"]
                .as_array_mut()
                .unwrap()
                .retain(|c| c["key"] != "stage.seating.arc"),
            1 => bad["stage"]["constructions"][0]["shape"]["arc"]["extra"] = json!(true),
            _ => bad["stage"]["constructions"][0]["shape"]["arc"] = json!({}),
        }
        assert!(Document::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}
#[test]
fn arc_cloning_keeps_parameters_but_not_edit_lock_and_budget_is_bounded() {
    let mut doc = Document::new("弧排复制").unwrap();
    let mut s = vector()["shape"].clone();
    s["rows"] = json!(8);
    s["columns"] = json!(64);
    s["columnSpacingMeters"] = json!("0.6");
    s["arc"] = json!({"radiusMeters":"30"});
    put(&mut doc, &s).unwrap();
    let id = doc.view().stage.constructions[0].id.clone();
    let edit = |doc: &mut Document, c: Value| {
        doc.edit(serde_json::from_value::<EditCommand>(json!({"op":"stage","command":c})).unwrap())
    };
    edit(&mut doc,json!({"op":"setEditLocks","targets":[{"kind":"construction","targetId":id}],"locked":true})).unwrap();
    edit(
        &mut doc,
        json!({"op":"duplicateConstruction","id":id,"name":"副本"}),
    )
    .unwrap();
    assert_eq!(doc.view().stage.constructions.len(), 2);
    assert_eq!(doc.view().stage.edit_locks.len(), 1);
    let before = doc.clone();
    assert!(
        edit(
            &mut doc,
            json!({"op":"duplicateConstruction","id":id,"name":"超量"})
        )
        .unwrap_err()
        .contains("1024")
    );
    assert_eq!(doc, before);
}
