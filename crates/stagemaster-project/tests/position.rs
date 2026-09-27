use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{Document, EditCommand, PositionAxis};
use stagemaster_spatial::{Installation, positioning::JointAngles};
fn edit(d: &mut Document, c: Value) -> Result<(), String> {
    d.edit(serde_json::from_value::<EditCommand>(c).unwrap())
}
fn definition() -> Value {
    json!({"name":"两轴灯","manufacturer":"测试","model":"正交","mode":"五通道","footprint":5,"channels":[{"attribute":"dimmer","coarse":1,"fine":null,"defaultValue":65535},{"attribute":"pan","coarse":4,"fine":2,"defaultValue":32768},{"attribute":"tilt","coarse":5,"fine":3,"defaultValue":32768}],"positioning":{"kind":"intersectingOrthogonal","pan":{"minDegrees":"-270","maxDegrees":"270","reversed":true},"tilt":{"minDegrees":"-135","maxDegrees":"135","reversed":false}}})
}
fn setup() -> Document {
    let mut d = Document::new("指向").unwrap();
    edit(
        &mut d,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition()}}),
    )
    .unwrap();
    let v = d.view();
    let pid = &v.profiles.last().unwrap().id;
    for address in [1, 6] {
        edit(&mut d,json!({"op":"addFixture","name":format!("灯 {address}"),"profileId":pid,"domainId":v.domains[0].id,"universe":1,"address":address})).unwrap();
    }
    edit(&mut d, json!({"op":"addScene","name":"对焦"})).unwrap();
    edit(&mut d,json!({"op":"stage","command":{"op":"putSpace","id":null,"name":"厅","outlineMeters":[["0","0"],["8","0"],["8","6"],["0","6"]],"floorElevationMeters":"0","clearHeightMeters":"6"}})).unwrap();
    let v = d.view();
    for (i, f) in v.fixtures.iter().enumerate() {
        edit(&mut d,json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":f.id,"spaceId":v.stage.spaces[0].id,"positionMeters":{"x":if i==0 {"1"}else{"7"},"y":"2","z":"5"},"rotationDegreesXYZ":{"x":"0","y":"0","z":"30"}}}})).unwrap();
    }
    d
}
fn position(d: &mut Document, mut c: Value) -> Result<(), String> {
    let v = d.view();
    c["sceneId"] = json!(v.scenes[0].id);
    c["fixtureIds"] = json!(v.fixtures.iter().map(|f| &f.id).collect::<Vec<_>>());
    edit(d, json!({"op":"position","command":c}))
}
#[test]
fn physical_mapping_reversal_quantization_and_endpoints() {
    let a = PositionAxis {
        min_degrees: "-270".into(),
        max_degrees: "270".into(),
        reversed: true,
    };
    for fine in [false, true] {
        for degree in [-270.0, -90.0, 0.0, 90.0, 270.0] {
            let raw = a.encode(degree, fine).unwrap();
            let actual = a.decode(raw, fine).unwrap();
            assert!(
                (actual - degree).abs() <= 540.0 / if fine { 65535.0 } else { 255.0 } / 2.0 + 1e-9
            );
        }
    }
    assert_eq!(a.encode(-270.0, true).unwrap(), 65535);
    assert_eq!(a.encode(270.0, true).unwrap(), 0);
    assert!(a.encode(270.01, true).is_err());
    assert!(a.encode(f64::NAN, true).is_err());
    // Eight-bit preview must use the same high byte as the existing encoder.
    assert!((a.decode(0x80ff, false).unwrap() - a.decode(0x8000, false).unwrap()).abs() < 1e-10);
    let fractional = PositionAxis {
        min_degrees: "-1.1".into(),
        max_degrees: "3.2".into(),
        reversed: false,
    };
    assert!(fractional.decode(65535, true).unwrap() <= 3.2);
}
#[test]
fn common_target_uses_each_installation_and_zero_then_encodes_real_coarse_fine() {
    let mut d = setup();
    let f = d.view().fixtures[0].id.clone();
    edit(&mut d,json!({"op":"position","command":{"op":"calibrate","fixtureId":f,"correction":{"panDegrees":"3","tiltDegrees":"-2"}}})).unwrap();
    position(
        &mut d,
        json!({"op":"aim","targetMeters":{"x":"4","y":"3","z":"1"},"branch":null}),
    )
    .unwrap();
    let v = d.view();
    let plan = d.compile_scene(&v.scenes[0].id).unwrap();
    let mut player = Player::new(plan.plan, 0);
    player.execute(0, 0).unwrap();
    let output = plan.output.render(player.values()).unwrap();
    let mut pans = Vec::new();
    for f in &v.fixtures {
        let p = f.positioning.as_ref().unwrap();
        let value = |k: &str| {
            u16::try_from(
                v.scenes[0]
                    .values
                    .iter()
                    .find(|v| v.fixture_id == f.id && v.attribute == k)
                    .unwrap()
                    .value
                    .unwrap(),
            )
            .unwrap()
        };
        let raw = value("pan");
        pans.push(raw);
        let addr = usize::try_from(f.address.unwrap() - 1).unwrap();
        assert_eq!(output.slots[addr + 3], raw.to_be_bytes()[0]);
        assert_eq!(output.slots[addr + 1], raw.to_be_bytes()[1]);
        let placement = v
            .stage
            .placements
            .iter()
            .find(|p| p.fixture_id == f.id)
            .unwrap();
        let install = Installation {
            position_meters: placement.position_meters.numbers(100_000.0).unwrap(),
            rotation_degrees_xyz: placement.rotation_degrees_xyz.numbers(3600.0).unwrap(),
        };
        let ray = p
            .head(f.zero_correction.as_ref())
            .unwrap()
            .ray(
                install,
                JointAngles {
                    pan_degrees: p.pan.decode(raw, true).unwrap(),
                    tilt_degrees: p.tilt.decode(value("tilt"), true).unwrap(),
                },
            )
            .unwrap();
        let scale = (1.0 - ray.origin_meters[2]) / ray.direction[2];
        assert!((ray.origin_meters[0] + scale * ray.direction[0] - 4.0).abs() < 0.002);
        assert!((ray.origin_meters[1] + scale * ray.direction[1] - 3.0).abs() < 0.002);
    }
    assert_ne!(pans[0], pans[1]);
    assert_eq!(d, Document::decode(&d.encode().unwrap()).unwrap());
}
#[test]
fn aim_and_axes_errors_are_atomic_and_home_preserves_light_attributes() {
    let mut d = setup();
    let before = d.clone();
    for c in [
        json!({"op":"axes","panDegrees":"271"}),
        json!({"op":"axes"}),
        json!({"op":"aim","targetMeters":{"x":"7","y":"2","z":"5"}}),
        json!({"op":"aim","targetMeters":{"x":"4","y":"2","z":"0"},"branch":"unknown"}),
    ] {
        assert!(position(&mut d, c).is_err());
        assert_eq!(d, before);
    }
    position(&mut d, json!({"op":"axes","panDegrees":"90"})).unwrap();
    let unchanged = |doc: &Document| {
        doc.view().scenes[0]
            .values
            .iter()
            .filter(|v| v.attribute != "pan")
            .map(|v| (v.fixture_id.clone(), v.attribute.clone(), v.value))
            .collect::<Vec<_>>()
    };
    assert_eq!(unchanged(&d), unchanged(&before));
    position(&mut d, json!({"op":"home"})).unwrap();
    let v = d.view();
    assert!(
        v.scenes[0]
            .values
            .iter()
            .filter(|v| v.attribute == "pan" || v.attribute == "tilt")
            .all(|v| v.value == Some(32768))
    );
    assert!(
        v.scenes[0]
            .values
            .iter()
            .filter(|v| v.attribute == "dimmer")
            .all(|v| v.value == Some(65535))
    );
    let mut narrow = definition();
    narrow["positioning"]["tilt"]["minDegrees"] = json!("10");
    narrow["positioning"]["tilt"]["maxDegrees"] = json!("20");
    // Tampered persisted model is valid but makes the chosen floor point unreachable.
    let mut root: Value = serde_json::from_slice(&d.encode().unwrap()).unwrap();
    root["lighting"]["profiles"][2]["positioning"] = narrow["positioning"].clone();
    d = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let before = d.clone();
    let err = position(
        &mut d,
        json!({"op":"aim","targetMeters":{"x":"1","y":"2","z":"0"}}),
    )
    .unwrap_err();
    assert!(err.contains("机械行程"));
    assert_eq!(d, before);
}
#[test]
fn capability_model_calibration_and_exchange_protect_existing_programming() {
    let mut d = setup();
    let original: Value = serde_json::from_slice(&d.encode().unwrap()).unwrap();
    for path in [
        "/lighting/profiles/2/positioning/pan/maxDegrees",
        "/lighting/fixtures/0/zeroCorrection/panDegrees",
    ] {
        let mut root = original.clone();
        root["lighting"]["fixtures"][0]["zeroCorrection"] =
            json!({"panDegrees":"0","tiltDegrees":"0"});
        *root.pointer_mut(path).unwrap() = json!("99999");
        assert!(Document::decode(&serde_json::to_vec(&root).unwrap()).is_err());
    }
    let mut root = original;
    root["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["key"] != "lighting.positioning");
    assert!(
        Document::decode(&serde_json::to_vec(&root).unwrap())
            .unwrap_err()
            .contains("能力")
    );
    let mut def = definition();
    def["positioning"]["pan"]["reversed"] = json!(false);
    edit(
        &mut d,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":def}}),
    )
    .unwrap();
    let v = d.view();
    let before = d.clone();
    let error=edit(&mut d,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":[v.fixtures[0].id],"profileId":v.profiles.last().unwrap().id}})).unwrap_err();
    assert!(error.contains("运动模型"));
    assert_eq!(d, before);
    let old = Document::new("固定灯").unwrap();
    assert!(Document::decode(&old.encode().unwrap()).is_ok());
}

#[test]
fn portable_package_preserves_calibrated_common_target_and_nonadjacent_fine_slots() {
    let mut d = setup();
    let id = d.view().fixtures[0].id.clone();
    edit(&mut d,json!({"op":"position","command":{"op":"calibrate","fixtureId":id,"correction":{"panDegrees":"3","tiltDegrees":"-2"}}})).unwrap();
    position(
        &mut d,
        json!({"op":"aim","targetMeters":{"x":"4","y":"3","z":"1"},"branch":null}),
    )
    .unwrap();
    let id = d.view().scenes[0].id.clone();
    let compiled = d.compile_scene(&id).unwrap();
    let built = d
        .build_package(&[stagemaster_project::PackageSelection::Scene { id }])
        .unwrap();
    let archive = stagemaster_package::Archive::open(built.bytes.as_slice()).unwrap();
    let program = archive.load(built.bytes.as_slice(), 0).unwrap();
    assert_eq!(compiled.plan, program.plan);
    let mut player = Player::new(program.plan, 0);
    player.execute(0, 0).unwrap();
    let mut slots = [0; 512];
    program.output.render(player.values(), &mut slots).unwrap();
    assert_eq!(
        compiled.output.render(player.values()).unwrap().slots,
        slots
    );
    assert_eq!(archive.entries()[0].usage.attributes, 6);
}
