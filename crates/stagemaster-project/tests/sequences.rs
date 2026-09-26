use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{Document, EditCommand};
fn root() -> Value {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    root
}
fn decode(root: &Value) -> Document {
    Document::decode(&serde_json::to_vec(root).unwrap()).unwrap()
}
fn apply(doc: &mut Document, cmd: Value) -> Result<(), String> {
    doc.edit(EditCommand::Sequence {
        command: serde_json::from_value(cmd).unwrap(),
    })
}

#[test]
fn scene_reference_deletion_number_aliases_and_invalid_times_are_atomic() {
    let mut doc = decode(&root());
    let view = doc.view();
    let seq = &view.sequences[0];
    let first = &seq.steps[0];
    let last = &seq.steps[1];
    let before = doc.clone();
    assert!(
        doc.edit(serde_json::from_value(json!({"op":"removeScene","id":first.scene_id})).unwrap())
            .unwrap_err()
            .contains("入场节目")
    );
    assert_eq!(doc, before);
    for (number, delay) in [("1.0", 0), ("3", 86_400_001)] {
        assert!(apply(&mut doc,json!({"kind":"updateStep","id":seq.id,"stepId":last.id,"name":"不能成功","number":number,"sceneId":last.scene_id,"delayMs":delay,"fadeMs":1000,"waitMs":null})).is_err());
        assert_eq!(doc, before);
    }
    assert!(
        apply(
            &mut doc,
            json!({"kind":"moveStep","id":seq.id,"stepId":first.id,"index":9})
        )
        .is_err()
    );
    assert_eq!(doc, before);
}
#[test]
fn moving_and_copying_preserve_references_and_create_distinct_identities() {
    let mut doc = decode(&root());
    let view = doc.view();
    let seq = &view.sequences[0];
    let first = &seq.steps[0];
    apply(
        &mut doc,
        json!({"kind":"moveStep","id":seq.id,"stepId":first.id,"index":1}),
    )
    .unwrap();
    assert_eq!(doc.view().sequences[0].steps[1].id, first.id);
    assert_eq!(doc.view().sequences[0].steps[1].number, "1");
    apply(
        &mut doc,
        json!({"kind":"duplicateStep","id":seq.id,"stepId":first.id}),
    )
    .unwrap();
    let copy = &doc.view().sequences[0].steps[2];
    assert_ne!(copy.id, first.id);
    assert_eq!(copy.scene_id, first.scene_id);
    assert_eq!(copy.fade_ms, first.fade_ms);
    assert_eq!(copy.number, "3");
    apply(
        &mut doc,
        json!({"kind":"duplicate","id":seq.id,"name":"第二份"}),
    )
    .unwrap();
    let view = doc.view();
    assert_ne!(view.sequences[0].id, view.sequences[1].id);
    for (a, b) in view.sequences[0].steps.iter().zip(&view.sequences[1].steps) {
        assert_ne!(a.id, b.id);
        assert_eq!(a.scene_id, b.scene_id);
    }
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
#[test]
fn creation_insertion_and_last_step_removal_protect_a_usable_list() {
    let mut doc = decode(&root());
    let scene = doc.view().scenes[0].id.clone();
    apply(
        &mut doc,
        json!({"kind":"add","name":"新列表","sceneId":scene}),
    )
    .unwrap();
    let view = doc.view();
    let seq = &view.sequences[1];
    let step = &seq.steps[0];
    assert!(
        apply(
            &mut doc,
            json!({"kind":"removeStep","id":seq.id,"stepId":step.id})
        )
        .unwrap_err()
        .contains("至少")
    );
    apply(
        &mut doc,
        json!({"kind":"insertStep","id":seq.id,"sceneId":scene,"afterId":step.id}),
    )
    .unwrap();
    apply(
        &mut doc,
        json!({"kind":"removeStep","id":seq.id,"stepId":step.id}),
    )
    .unwrap();
    assert_eq!(doc.view().sequences[1].steps.len(), 1);
    apply(&mut doc, json!({"kind":"remove","id":seq.id})).unwrap();
    assert_eq!(doc.view().sequences.len(), 1);
}
#[test]
fn timebases_require_exact_milliseconds_and_valid_integer_limits() {
    let mut input = root();
    input["lighting"]["sequences"][0]["steps"][0]["fade"] =
        json!({"ticks":"72000","ticksPerSecond":"48000"});
    assert_eq!(decode(&input).view().sequences[0].steps[0].fade_ms, 1500);
    for time in [
        json!({"ticks":"1","ticksPerSecond":"48000"}),
        json!({"ticks":"9223372036854775808","ticksPerSecond":"1000"}),
        json!({"ticks":"0","ticksPerSecond":"1000000001"}),
    ] {
        input["lighting"]["sequences"][0]["steps"][0]["fade"] = time;
        assert!(Document::decode(&serde_json::to_vec(&input).unwrap()).is_err());
    }
}
#[test]
fn compiler_resolves_inheritance_release_isolation_and_presets_with_independent_targets() {
    let mut input = root();
    let target = input["lighting"]["scenes"][0]["assignments"][0]["target"].clone();
    input["lighting"]["scenes"][0]["assignments"] = json!([{"target":target,"operation":"set","source":{"kind":"literal","value":{"kind":"normalized","value":50000}}}]);
    input["lighting"]["profiles"][0]["attributes"][0]["default"]["value"] = json!(1000);
    let red = json!({"fixtureId":input["lighting"]["fixtures"][0]["id"],"attribute":"red"});
    input["lighting"]["scenes"][1]["assignments"] = json!([{"target":red,"operation":"set","source":{"kind":"literal","value":{"kind":"normalized","value":10000}}}]);
    let doc = decode(&input);
    let id = &doc.view().sequences[0].id;
    let compiled = doc.compile_sequence(id).unwrap();
    assert_eq!(compiled.plan.steps()[0].target[0], 50000);
    assert_eq!(compiled.plan.steps()[1].target[0], 50000);
    input["lighting"]["sequences"][0]["tracking"] = json!("isolated");
    assert_eq!(
        decode(&input).compile_sequence(id).unwrap().plan.steps()[1].target[0],
        1000
    );
    input["lighting"]["sequences"][0]["tracking"] = json!("inherited");
    input["lighting"]["scenes"][1]["assignments"] =
        json!([{"target":target,"operation":"release"}]);
    assert_eq!(
        decode(&input).compile_sequence(id).unwrap().plan.steps()[1].target[0],
        1000
    );
    let doc = decode(&root());
    let compiled = doc.compile_sequence(&doc.view().sequences[0].id).unwrap();
    assert_eq!(compiled.plan.steps()[0].target[3], 65535); // Blue resolved from the stored preset.
}
#[test]
fn encoded_frames_match_hand_values_and_save_reopen_preserves_playback() {
    let mut input = root();
    input["lighting"]["scenes"][0]["assignments"][0]["source"] =
        json!({"kind":"literal","value":{"kind":"normalized","value":65535}});
    let doc = decode(&input);
    let id = doc.view().sequences[0].id.clone();
    let compiled = doc.compile_sequence(&id).unwrap();
    let reopened = Document::decode(&doc.next_revision().encode().unwrap())
        .unwrap()
        .compile_sequence(&id)
        .unwrap();
    assert_eq!(compiled.plan, reopened.plan);
    let mut a = Player::new(compiled.plan, 0);
    let mut b = Player::new(reopened.plan, 0);
    a.execute(0, 0).unwrap();
    b.execute(0, 0).unwrap();
    for time in [0, 750, 1500, 6500, 7000, 7500] {
        a.advance(time).unwrap();
        b.advance(time).unwrap();
        let frame = compiled.output.render(a.values()).unwrap();
        assert_eq!(frame.slots.len(), 512);
        assert_eq!(
            frame.slots,
            reopened.output.render(b.values()).unwrap().slots
        );
        if time == 750 {
            assert_eq!(frame.slots[0], 128);
            assert_eq!(frame.slots[3], 128);
            assert_eq!(frame.slots[511], 0);
        }
    }
}
#[test]
fn compiler_rejects_unpatched_projects_without_affecting_editability() {
    let mut input = root();
    input["lighting"]["patches"] = json!([]);
    let doc = decode(&input);
    assert!(
        doc.compile_sequence(&doc.view().sequences[0].id)
            .unwrap_err_message()
            .contains("配适")
    );
}
trait ErrorMessage {
    fn unwrap_err_message(self) -> String;
}
impl<T> ErrorMessage for Result<T, String> {
    fn unwrap_err_message(self) -> String {
        match self {
            Ok(_) => panic!("expected rejection"),
            Err(e) => e,
        }
    }
}
#[test]
fn fully_automatic_zero_loop_is_rejected_but_manual_loop_is_valid() {
    let mut input = root();
    let seq = &mut input["lighting"]["sequences"][0];
    seq["repeat"] = json!("loop");
    for step in seq["steps"].as_array_mut().unwrap() {
        for key in ["delay", "fade"] {
            step[key]["ticks"] = json!("0");
        }
        step["advance"]["wait"]["ticks"] = json!("0");
    }
    assert!(Document::decode(&serde_json::to_vec(&input).unwrap()).is_err());
    input["lighting"]["sequences"][0]["steps"][0]["advance"] = json!({"kind":"manual"});
    decode(&input);
}

#[test]
fn compiler_rejects_two_output_lines_but_editor_preserves_them() {
    let mut doc = decode(&root());
    let view = doc.view();
    doc.edit(serde_json::from_value(json!({"op":"addFixture","name":"第二路","profileId":view.profiles[0].id,"domainId":view.domains[0].id,"universe":2,"address":1})).unwrap()).unwrap();
    assert!(
        doc.compile_sequence(&view.sequences[0].id)
            .unwrap_err_message()
            .contains("同一线路")
    );
    assert_eq!(doc.view().fixtures.len(), 2);
}

#[test]
fn sixteen_bit_mapping_preserves_coarse_fine_order() {
    let mut input = root();
    let p = &mut input["lighting"]["profiles"][0];
    p["footprint"] = json!(5);
    p["channels"][0]["encoding"] = json!("u16-be");
    p["channels"][0]["offsets"] = json!([0, 4]);
    input["lighting"]["scenes"][0]["assignments"][0]["source"]["value"]["value"] = json!(4660);
    let doc = decode(&input);
    let compiled = doc.compile_sequence(&doc.view().sequences[0].id).unwrap();
    let output = compiled
        .output
        .render(&compiled.plan.steps()[0].target)
        .unwrap();
    assert_eq!(output.slots[0], 0x12);
    assert_eq!(output.slots[4], 0x34);
}
