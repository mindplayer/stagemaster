use serde_json::{Value, json};
use stagemaster_previs::{editing_lights, playback_lights, scene};
use stagemaster_project::{Document, EditCommand, ValueMode};
fn edit(doc: &mut Document, value: Value) {
    doc.edit(serde_json::from_value(value).unwrap()).unwrap();
}
fn setup() -> (Document, String, String) {
    let mut doc = Document::new("投影核对").unwrap();
    let view = doc.view();
    doc.edit(EditCommand::AddFixture {
        name: "彩色灯".into(),
        profile_id: view.profiles[1].id.clone(),
        domain_id: view.domains[0].id.clone(),
        universe: 1,
        address: 1,
    })
    .unwrap();
    doc.edit(EditCommand::AddFixture {
        name: "尚未布置".into(),
        profile_id: view.profiles[0].id.clone(),
        domain_id: view.domains[0].id.clone(),
        universe: 1,
        address: 5,
    })
    .unwrap();
    let fixture = doc.view().fixtures[0].id.clone();
    doc.edit(EditCommand::AddScene {
        name: "红色".into(),
    })
    .unwrap();
    let scene = doc.view().scenes[0].id.clone();
    for (attr, value) in [("dimmer", 32768), ("red", 65535), ("green", 0), ("blue", 0)] {
        doc.edit(EditCommand::SetSceneValue {
            scene_id: scene.clone(),
            fixture_id: fixture.clone(),
            attribute: attr.into(),
            mode: ValueMode::Literal,
            value,
        })
        .unwrap();
    }
    edit(
        &mut doc,
        json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":fixture,"spaceId":null,"positionMeters":{"x":"2","y":"3","z":"4"},"rotationDegreesXYZ":{"x":"90","y":"0","z":"0"}}}}),
    );
    (doc, fixture, scene)
}
fn volume(triangles: &[[[f64; 3]; 3]]) -> f64 {
    triangles
        .iter()
        .map(|[a, b, c]| {
            (a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0
        })
        .sum()
}
#[test]
fn empty_project_and_unplaced_fixtures_do_not_invent_geometry() {
    let blank = scene(&Document::new("空白").unwrap()).unwrap();
    assert!(blank.meshes.is_empty());
    assert!(blank.fixtures.is_empty());
    let (doc, fixture, _) = setup();
    let before = doc.encode().unwrap();
    let projected = scene(&doc).unwrap();
    assert_eq!(projected.fixtures.len(), 1);
    assert_eq!(projected.fixtures[0].id, fixture);
    assert!(projected.fixtures[0].direction[0].abs() < 1e-12);
    assert!((projected.fixtures[0].direction[1] - 1.0).abs() < 1e-12);
    assert!(projected.fixtures[0].direction[2].abs() < 1e-12);
    assert_eq!(projected.fixtures[0].optics, "generic-illustrative");
    assert_eq!(before, doc.encode().unwrap());
}
#[test]
fn concave_extrusion_preserves_volume_in_both_outline_orders() {
    let (mut doc, _, _) = setup();
    let mut points = vec![
        ["0", "0"],
        ["8", "0"],
        ["8", "3"],
        ["3", "3"],
        ["3", "7"],
        ["0", "7"],
    ];
    for _ in 0..2 {
        edit(
            &mut doc,
            json!({"op":"stage","command":{"op":"putConstruction","id":null,"name":"凹台","shape":{"kind":"platform","spaceId":null,"outlineMeters":points,"baseElevationMeters":"2","heightMeters":"0.5"}}}),
        );
        points.reverse();
    }
    let projected = scene(&doc).unwrap();
    assert_eq!(projected.meshes.len(), 2);
    for mesh in projected.meshes {
        assert!((volume(&mesh.triangles) - 18.0).abs() < 1e-10);
        assert!(
            mesh.triangles
                .iter()
                .flatten()
                .all(|p| (2.0..=2.5).contains(&p[2]))
        );
    }
}
#[test]
fn small_valid_wall_edges_do_not_use_floor_area_threshold() {
    let (mut doc, _, _) = setup();
    edit(
        &mut doc,
        json!({"op":"stage","command":{"op":"putSpace","id":null,"name":"微边","outlineMeters":[["0","0"],["0.002","0"],["8","0"],["8","6"],["0","6"]],"floorElevationMeters":"1.2","clearHeightMeters":"4.5"}}),
    );
    let id = doc.view().stage.spaces[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"stage","command":{"op":"putConstruction","id":null,"name":"围护","shape":{"kind":"enclosure","spaceId":id,"wallThicknessMeters":"0.001","floorThicknessMeters":"0.1","ceilingThicknessMeters":"0.2"}}}),
    );
    let projected = scene(&doc).unwrap();
    assert_eq!(projected.meshes.len(), 2);
    assert_eq!(projected.meshes[0].view_role, "solid");
    assert_eq!(projected.meshes[1].view_role, "enclosureShell");
    assert_ne!(projected.meshes[0].id, projected.meshes[1].id);
    let triangles: Vec<_> = projected
        .meshes
        .iter()
        .flat_map(|m| m.triangles.iter().copied())
        .collect();
    assert!(
        triangles
            .iter()
            .flatten()
            .all(|p| p.iter().all(|v| v.is_finite()))
    );
    assert!((volume(&triangles) - (48.0 * 0.3 + 28.0 * 0.001 * 4.5)).abs() < 1e-8);
}
#[test]
fn isolated_scene_and_playback_use_identical_committed_light_values() {
    let (mut doc, fixture, id) = setup();
    let editing = editing_lights(&doc, Some(&id)).unwrap();
    assert_eq!(editing.len(), 1);
    assert_eq!(editing[0].fixture_id, fixture);
    assert!((editing[0].intensity - 32768.0 / 65535.0).abs() < 1e-12);
    assert!((editing[0].color[0] - 1.0).abs() < 1e-12);
    assert!(editing[0].color[1].abs() < 1e-12);
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"演出","sceneId":id}}),
    );
    let compiled = doc.compile_sequence(&doc.view().sequences[0].id).unwrap();
    let output = compiled
        .output
        .render(&compiled.plan.steps()[0].target)
        .unwrap();
    let playback = playback_lights(&doc, &output);
    assert!((playback[0].intensity - editing[0].intensity).abs() < 1e-12);
    for i in 0..3 {
        assert!((playback[0].color[i] - editing[0].color[i]).abs() < 1e-12);
    }
    assert!(editing_lights(&doc, None).unwrap()[0].intensity.abs() < 1e-12);
    assert!(editing_lights(&doc, Some("missing")).is_err());
}
#[test]
fn unsupported_normalized_attributes_do_not_masquerade_as_generic_lights() {
    let (doc, _, _) = setup();
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let profile = &mut root["lighting"]["profiles"][1];
    profile["attributes"][1]["key"] = json!("pan");
    profile["channels"][1]["attribute"] = json!("pan");
    for value in root["lighting"]["scenes"][0]["assignments"]
        .as_array_mut()
        .unwrap()
    {
        if value["target"]["attribute"] == "red" {
            value["target"]["attribute"] = json!("pan");
        }
    }
    let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    assert!(scene(&doc).is_err());
}
#[test]
fn mesh_budget_rejects_oversize_preview_without_mutating_editable_project() {
    let (mut doc, _, _) = setup();
    let points = (0..128)
        .map(|i| {
            let angle = f64::from(i) * std::f64::consts::TAU / 128.0;
            [angle.cos(), angle.sin()].map(|coordinate| {
                let rounded = (coordinate * 10_000_000.0).round() / 1_000_000.0;
                if rounded == 0.0 {
                    "0".to_owned()
                } else {
                    rounded.to_string()
                }
            })
        })
        .collect::<Vec<_>>();
    let commands=(0..202).map(|i|json!({"op":"stage","command":{"op":"putConstruction","id":null,"name":format!("台 {i}"),"shape":{"kind":"platform","spaceId":null,"outlineMeters":points,"baseElevationMeters":"0","heightMeters":"1"}}})).collect::<Vec<_>>();
    edit(&mut doc, json!({"op":"batch","commands":commands}));
    let before = doc.encode().unwrap();
    assert!(scene(&doc).is_err());
    assert_eq!(doc.encode().unwrap(), before);
}
