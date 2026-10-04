// Use the same public edit fixture as mixed-movement tests; no renderer-only scene model.
#[path = "../../stagemaster-project/tests/stage_mixed_support/mod.rs"]
mod support;
use serde_json::json;

#[test]
fn projected_owners_preserve_business_identity_and_attached_lights() {
    let (mut doc, targets, ids) = support::setup();
    let before = support::root(&doc);
    let scene = stagemaster_previs::scene(&doc).unwrap();
    assert_eq!(scene.meshes.len(), 3);
    for (mesh, target) in scene.meshes.iter().zip(&targets) {
        assert_eq!(mesh.construction_id, target["targetId"]);
        assert!(mesh.movable);
    }
    assert_eq!(scene.meshes[0].attached_fixture_ids, ids[..2]);
    assert!(
        scene.meshes[1..]
            .iter()
            .all(|m| m.attached_fixture_ids.is_empty())
    );
    assert_eq!(support::root(&doc), before);
    support::translate(&mut doc, &targets, ["1.25", "0", "0"]).unwrap();
    let moved = stagemaster_previs::scene(&doc).unwrap();
    for (a, b) in scene.meshes.iter().zip(&moved.meshes) {
        assert_eq!(a.construction_id, b.construction_id);
        assert_eq!(a.attached_fixture_ids, b.attached_fixture_ids);
        for (p, q) in a
            .triangles
            .iter()
            .flatten()
            .zip(b.triangles.iter().flatten())
        {
            assert!((q[0] - p[0] - 1.25).abs() < 1e-9);
        }
    }
    for i in 0..2 {
        assert!(
            (moved.fixtures[i].origin_meters[0] - scene.fixtures[i].origin_meters[0] - 1.25).abs()
                < 1e-9
        );
    }
    assert_eq!(
        moved.fixtures[2].origin_meters.map(f64::to_bits),
        scene.fixtures[2].origin_meters.map(f64::to_bits)
    );
}

#[test]
fn enclosure_floor_and_shell_share_an_immovable_owner() {
    let mut doc = stagemaster_project::Document::new("围护归属").unwrap();
    support::edit(&mut doc, json!({"op":"putSpace","id":null,"name":"房间","outlineMeters":[["0","0"],["8","0"],["8","6"],["0","6"]],"floorElevationMeters":"0","clearHeightMeters":"7"})).unwrap();
    let space_id = doc.view().stage.spaces[0].id.clone();
    support::edit(&mut doc, json!({"op":"putConstruction","id":null,"name":"围护","shape":{"kind":"enclosure","spaceId":space_id,"wallThicknessMeters":"0.1","floorThicknessMeters":"0.1","ceilingThicknessMeters":"0.1"}})).unwrap();
    let owner = doc.view().stage.constructions[0].id.clone();
    let before = doc.encode().unwrap();
    let projected = stagemaster_previs::scene(&doc).unwrap();
    assert_eq!(projected.meshes.len(), 2);
    assert_ne!(projected.meshes[0].id, projected.meshes[1].id);
    for mesh in projected.meshes {
        assert_eq!(mesh.construction_id, owner);
        assert!(!mesh.movable);
        assert!(mesh.attached_fixture_ids.is_empty());
        let wire = serde_json::to_value(mesh).unwrap();
        assert_eq!(wire["constructionId"], owner);
        assert_eq!(wire["movable"], false);
        assert_eq!(wire["attachedFixtureIds"], json!([]));
    }
    assert_eq!(doc.encode().unwrap(), before);
}
