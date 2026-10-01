#[path = "support/world_line.rs"]
mod support;
use serde_json::json;
use stagemaster_playback::Player;
use stagemaster_project::{Document, PackageSelection};
use stagemaster_spatial::{Installation, positioning::JointAngles};
use support::{
    common::{decode, edit, put, raw},
    effect, setup,
};

#[test]
fn entire_quantized_cycle_points_within_tolerance_and_matches_portable_package() {
    for fine in [true, false] {
        let (mut doc, scene, ids) = setup(fine);
        put(&mut doc, &scene, &effect(&ids, fine)).unwrap();
        let reopened = Document::decode(&doc.encode().unwrap()).unwrap();
        let view = reopened.view();
        let compiled = reopened.compile_scene(&scene).unwrap();
        assert_eq!(compiled.plan.keyframe_count(), 128);
        let package = reopened
            .build_package(&[PackageSelection::Scene { id: scene }])
            .unwrap();
        let archive = stagemaster_package::Archive::open(package.bytes.as_slice()).unwrap();
        let program = archive.load(package.bytes.as_slice(), 0).unwrap();
        assert_eq!(compiled.plan, program.plan);
        let mut player = Player::new(compiled.plan, 0);
        player.execute(0, 0).unwrap();
        for ms in 0_u32..=65536 {
            player.advance(u64::from(ms)).unwrap();
            let output = compiled.output.render(player.values()).unwrap();
            let mut slots = [0; 512];
            program.output.render(player.values(), &mut slots).unwrap();
            assert_eq!(output.slots, slots);
            for (i, fixture) in output.fixtures.iter().enumerate() {
                let model = view.fixtures[i].positioning.as_ref().unwrap();
                let placement = &view.stage.placements[i];
                let numbers = |v: &stagemaster_project::SpatialVector3| {
                    [
                        v.x.parse::<f64>().unwrap(),
                        v.y.parse().unwrap(),
                        v.z.parse().unwrap(),
                    ]
                };
                let install = Installation {
                    position_meters: numbers(&placement.position_meters),
                    rotation_degrees_xyz: numbers(&placement.rotation_degrees_xyz),
                };
                let head = model
                    .head(view.fixtures[i].zero_correction.as_ref())
                    .unwrap();
                let ray = head
                    .ray(
                        install,
                        JointAngles {
                            pan_degrees: model
                                .pan
                                .decode(fixture.attributes[1].value, fine)
                                .unwrap(),
                            tilt_degrees: model
                                .tilt
                                .decode(fixture.attributes[2].value, fine)
                                .unwrap(),
                        },
                    )
                    .unwrap();
                let delay = if i == 0 { 16384 } else { 32768 };
                let phase = f64::from((ms + 65536 - delay) % 65536) / 65536.0;
                let u = (1.0 - (std::f64::consts::TAU * phase).cos()) * 0.5;
                let target = [-0.5 + u, 1.0 + u, 0.5];
                let distance = target
                    .into_iter()
                    .zip(install.position_meters)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt();
                let residual = (0..3)
                    .map(|axis| {
                        (install.position_meters[axis] + distance * ray.direction[axis]
                            - target[axis])
                            .powi(2)
                    })
                    .sum::<f64>()
                    .sqrt();
                assert!(
                    residual <= if fine { 0.1 } else { 0.8 },
                    "{fine} {ms} {i}: {residual}"
                );
                assert_eq!(fixture.attributes[0].value, 12345);
            }
        }
    }
}

#[test]
fn malformed_paths_conflicts_and_missing_capability_do_not_mutate_the_document() {
    let (mut doc, scene, ids) = setup(true);
    let original = doc.encode().unwrap();
    let base = effect(&ids, true);
    for candidate in [
        {
            let mut e = base.clone();
            e["channels"] = json!([{"attribute":"pan"}]);
            e
        },
        {
            let mut e = base.clone();
            e["waveform"] = json!("smooth");
            e
        },
        {
            let mut e = base.clone();
            e["targetPath"]["toMeters"] = e["targetPath"]["fromMeters"].clone();
            e
        },
        {
            let mut e = base.clone();
            e["targetPath"]["maxErrorMeters"] = json!("0");
            e
        },
        {
            let mut e = base.clone();
            e["channels"][0] = json!({"attribute":"pan","low":0,"high":65535});
            e
        },
    ] {
        assert!(put(&mut doc, &scene, &candidate).is_err());
        assert_eq!(doc.encode().unwrap(), original);
    }
    put(&mut doc, &scene, &base).unwrap();
    let current = doc.encode().unwrap();
    let mut duplicate = base.clone();
    duplicate["id"] = json!("49999999-0000-4000-8000-000000000002");
    assert!(
        put(&mut doc, &scene, &duplicate)
            .unwrap_err()
            .contains("同一灯具属性")
    );
    assert_eq!(doc.encode().unwrap(), current);
    let mut root = raw(&doc);
    root["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "lighting.effects.world-line");
    assert!(
        Document::decode(&serde_json::to_vec(&root).unwrap())
            .err()
            .unwrap()
            .contains("缺少工程能力")
    );
}

#[test]
fn compilation_reports_fixture_precision_and_missing_placement_without_silent_fallback() {
    let (mut doc, scene, ids) = setup(false);
    put(&mut doc, &scene, &effect(&ids, true)).unwrap();
    let error = doc.compile_scene(&scene).err().unwrap();
    assert!(
        error.contains("共同直线") && error.contains("灯 1") && error.contains("误差上界"),
        "{error}"
    );
    let mut root = raw(&doc);
    root["stage"]["placements"] = json!([]);
    assert!(
        decode(&root)
            .compile_scene(&scene)
            .err()
            .unwrap()
            .contains("尚未布置灯位")
    );
    // Inactive authoring intent can be retained without occupying runtime resources.
    root["lighting"]["scenes"][0]["effects"][0]["enabled"] = json!(false);
    assert_eq!(
        decode(&root)
            .compile_scene(&scene)
            .unwrap()
            .plan
            .keyframe_count(),
        0
    );
}

#[test]
fn changed_installation_recompiles_world_targets_but_keeps_the_old_plan_immutable() {
    let (mut doc, scene, ids) = setup(true);
    put(&mut doc, &scene, &effect(&ids, true)).unwrap();
    let old = doc.compile_scene(&scene).unwrap().plan;
    let unchanged = old.clone();
    let mut placement = serde_json::to_value(&doc.view().stage.placements[0]).unwrap();
    placement["positionMeters"]["x"] = json!("-1.5");
    edit(
        &mut doc,
        json!({"op":"stage","command":{"op":"putPlacement","placement":placement}}),
    )
    .unwrap();
    let new = doc.compile_scene(&scene).unwrap().plan;
    assert_ne!(old.effects()[0][0], new.effects()[0][0]);
    assert_eq!(old.effects()[0][2], new.effects()[0][2]);
    assert_eq!(old, unchanged);
}
