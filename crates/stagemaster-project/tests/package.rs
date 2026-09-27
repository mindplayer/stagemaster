use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use stagemaster_package::{Archive, Kind};
use stagemaster_playback::Player;
use stagemaster_project::{Document, PackageSelection};
fn edit(d: &mut Document, command: Value) {
    d.edit(serde_json::from_value(command).unwrap()).unwrap();
}
fn source() -> Document {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap()
}
fn all(doc: &Document) -> Vec<PackageSelection> {
    let v = doc.view();
    v.scenes
        .into_iter()
        .map(|s| PackageSelection::Scene { id: s.id })
        .chain(
            v.sequences
                .into_iter()
                .map(|s| PackageSelection::Sequence { id: s.id }),
        )
        .collect()
}
fn compare(doc: &Document) {
    let built = doc.build_package(&all(doc)).unwrap();
    let archive = Archive::open(built.bytes.as_slice()).unwrap();
    for (index, entry) in archive.entries().iter().enumerate() {
        let id = uuid::Uuid::from_bytes(entry.id).to_string();
        let compiled = match entry.kind {
            Kind::Scene => doc.compile_scene(&id),
            Kind::Sequence => doc.compile_sequence(&id),
        }
        .unwrap();
        let program = archive.load(built.bytes.as_slice(), index).unwrap();
        assert_eq!(compiled.plan, program.plan);
        let mut before = Player::new(compiled.plan, 0);
        let mut after = Player::new(program.plan, 0);
        before.execute(0, 0).unwrap();
        after.execute(0, 0).unwrap();
        for t in (0..20_000).step_by(17) {
            before.advance(t).unwrap();
            after.advance(t).unwrap();
            assert_eq!(before.values(), after.values());
            assert_eq!(before.index(), after.index());
            assert_eq!(before.status(), after.status());
            let mut slots = [0; 512];
            program.output.render(after.values(), &mut slots).unwrap();
            assert_eq!(
                compiled.output.render(before.values()).unwrap().slots,
                slots
            );
            if t == 1700 {
                before.pause(t).unwrap();
                after.pause(t).unwrap();
            }
            if t == 3400 {
                before.resume(t).unwrap();
                after.resume(t).unwrap();
            }
        }
        before.stop(20_000).unwrap();
        after.stop(20_000).unwrap();
        let mut slots = [0; 512];
        program.output.render(after.values(), &mut slots).unwrap();
        assert_eq!(
            compiled.output.render(before.values()).unwrap().slots,
            slots
        );
    }
}
#[test]
fn self_contained_presets_tracking_timing_and_output_survive_export() {
    let doc = source();
    let before = doc.encode().unwrap();
    compare(&doc);
    assert_eq!(doc.encode().unwrap(), before);
    let mut reversed = all(&doc);
    reversed.reverse();
    assert_eq!(
        doc.build_package(&all(&doc)).unwrap().bytes,
        doc.build_package(&reversed).unwrap().bytes
    );
}
#[test]
fn effect_curves_and_keyframes_use_the_existing_integer_kernel() {
    let mut d = source();
    let v = d.view();
    let fixtures: Vec<_> = v.fixtures.iter().map(|f| &f.id).collect();
    for waveform in ["smooth", "triangle", "pulse", "keyframes"] {
        let mut e = json!({"id":"29999999-0000-4000-8000-000000000001","name":"呼吸","enabled":true,"fixtureIds":fixtures,"periodMs":800,"spreadDegrees":360,"phaseDegrees":45,"reverse":true,"waveform":waveform,"dutyPercent":33,"channels":[{"attribute":"dimmer","low":1024,"high":55000}]});
        if waveform == "keyframes" {
            e["channels"] = json!([{"attribute":"dimmer","keyframes":[{"position":0,"value":0,"transition":"linear"},{"position":5000,"value":65535,"transition":"smooth"}]}]);
        }
        edit(
            &mut d,
            json!({"op":"effect","command":{"kind":"put","sceneId":v.scenes[0].id,"effect":e}}),
        );
        compare(&d);
    }
}
#[test]
fn unsaved_content_has_its_own_identity_without_modifying_revision() {
    let mut doc = source();
    let selection = all(&doc);
    let before = doc.build_package(&selection).unwrap();
    edit(
        &mut doc,
        json!({"op":"setInfo","name":"未保存的新名称","description":"修改说明"}),
    );
    let after = doc.build_package(&selection).unwrap();
    assert_eq!(before.report.revision_id, after.report.revision_id);
    assert_ne!(before.report.source_digest, after.report.source_digest);
    assert_ne!(before.report.package_digest, after.report.package_digest);
}
#[test]
fn selection_and_resource_errors_do_not_return_partial_packages() {
    let doc = source();
    let selections = all(&doc);
    assert!(doc.build_package(&[]).is_err());
    assert!(
        doc.build_package(&[selections[0].clone(), selections[0].clone()])
            .is_err()
    );
    let mut mixed = selections.clone();
    mixed.push(PackageSelection::Scene {
        id: "29999999-0000-4000-8000-000000000099".into(),
    });
    let errors = doc.build_package(&mixed).err().unwrap();
    assert!(errors[0].location.is_some());
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let original = root["lighting"]["sequences"][0]["steps"][0].clone();
    root["lighting"]["sequences"][0]["steps"] = json!(
        (1..=129)
            .map(|i| {
                let mut step = original.clone();
                step["id"] = json!(uuid::Uuid::new_v4().to_string());
                step["number"] = json!(i.to_string());
                step
            })
            .collect::<Vec<_>>()
    );
    let over = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let errors = over.build_package(&all(&over)).err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("128")));
    root["lighting"]["patches"] = json!([]);
    let unpatched = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let errors = unpatched.build_package(&all(&unpatched)).err().unwrap();
    assert!(errors.iter().all(|e| e.location.is_some()));
}

#[test]
fn accepted_compact_projects_can_be_hashed_without_pretty_print_expansion() {
    let mut root: Value = serde_json::from_slice(&source().encode().unwrap()).unwrap();
    let scene = root["lighting"]["scenes"][0].clone();
    for _ in 0..6000 {
        let mut copy = scene.clone();
        copy["id"] = json!(uuid::Uuid::new_v4().to_string());
        root["lighting"]["scenes"]
            .as_array_mut()
            .unwrap()
            .push(copy);
    }
    let bytes = serde_json::to_vec(&root).unwrap();
    assert!(bytes.len() < stagemaster_project::MAX_BYTES);
    assert!(serde_json::to_vec_pretty(&root).unwrap().len() > stagemaster_project::MAX_BYTES);
    let doc = Document::decode(&bytes).unwrap();
    let built = doc
        .build_package(&[PackageSelection::Scene {
            id: scene["id"].as_str().unwrap().into(),
        }])
        .unwrap();
    assert_eq!(built.report.programs.len(), 1);
    let archive = Archive::open(built.bytes.as_slice()).unwrap();
    assert_eq!(
        archive.source().snapshot_digest,
        <[u8; 32]>::from(Sha256::digest(&bytes))
    );
}
