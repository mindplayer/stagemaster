use serde_json::{Value, json};
use stagemaster_project::Document;
#[test]
fn chairs_have_positive_volume_and_correct_world_bounds_in_one_section_mesh() {
    let mut doc = Document::new("座区网格").unwrap();
    let vector: Value =
        serde_json::from_str(include_str!("../../../tools/test-data/seating-layout.json")).unwrap();
    doc.edit(serde_json::from_value(json!({"op":"stage","command":{"op":"putConstruction","id":null,"name":"观众","shape":vector["shape"]}})).unwrap()).unwrap();
    let before = doc.encode().unwrap();
    let scene = stagemaster_previs::scene(&doc).unwrap();
    assert_eq!(scene.meshes.len(), 1);
    let mesh = &scene.meshes[0];
    assert_eq!(mesh.triangles.len(), 8 * 72);
    assert_eq!(mesh.id, doc.view().stage.constructions[0].id);
    let points: Vec<_> = mesh.triangles.iter().flatten().collect();
    let bounds = |axis| {
        (
            points.iter().map(|p| p[axis]).fold(f64::INFINITY, f64::min),
            points
                .iter()
                .map(|p| p[axis])
                .fold(f64::NEG_INFINITY, f64::max),
        )
    };
    for (axis, expected) in [(0, (9.25, 10.75)), (1, (17.5, 22.5)), (2, (0.5, 1.35))] {
        let (min, max) = bounds(axis);
        assert!((min - expected.0).abs() < 1e-10);
        assert!((max - expected.1).abs() < 1e-10);
    }
    let volume: f64 = mesh
        .triangles
        .iter()
        .map(|[a, b, c]| {
            (a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0
        })
        .sum();
    assert!((volume - 0.19568).abs() < 1e-9);
    assert_eq!(doc.encode().unwrap(), before);
    assert!(scene.fixtures.is_empty());
}

#[test]
fn curved_chair_mesh_centers_and_backs_match_world_focus_without_extra_objects() {
    let mut doc = Document::new("弧排网格").unwrap();
    let v: Value = serde_json::from_str(include_str!(
        "../../../tools/test-data/seating-arc-layout.json"
    ))
    .unwrap();
    doc.edit(serde_json::from_value(json!({"op":"stage","command":{"op":"putConstruction","id":null,"name":"弧排","shape":v["shape"]}})).unwrap()).unwrap();
    let scene = stagemaster_previs::scene(&doc).unwrap();
    assert_eq!(scene.meshes.len(), 1);
    let mesh = &scene.meshes[0];
    assert_eq!(mesh.triangles.len(), 6 * 72);
    let expected: Vec<[f64; 2]> = serde_json::from_value(v["centers"].clone()).unwrap();
    let focus: [f64; 2] = serde_json::from_value(v["focus"].clone()).unwrap();
    let center = |triangles: &[stagemaster_previs::Triangle]| {
        [0, 1].map(|axis| {
            let values: Vec<f64> = triangles.iter().flatten().map(|p| p[axis]).collect();
            values
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min)
                .midpoint(values.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        })
    };
    for (chair, expected) in mesh.triangles.chunks_exact(72).zip(expected) {
        let seat = center(&chair[..12]);
        let back = center(&chair[12..24]);
        for axis in 0..2 {
            assert!((seat[axis] - expected[axis]).abs() < 1e-9);
        }
        let toward = [focus[0] - seat[0], focus[1] - seat[1]];
        let offset = [back[0] - seat[0], back[1] - seat[1]];
        assert!(toward[0] * offset[0] + toward[1] * offset[1] < 0.0);
        assert!((toward[0] * offset[1] - toward[1] * offset[0]).abs() < 1e-9);
    }
}
