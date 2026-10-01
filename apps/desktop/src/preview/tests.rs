use super::*;
fn document() -> Document {
    let mut input: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    input["entryPoints"] = serde_json::json!([]);
    Document::decode(&serde_json::to_vec(&input).unwrap()).unwrap()
}
#[test]
fn scene_effect_renderer_and_monitor_share_clock_serial_and_epoch() {
    let mut doc = document();
    let view = doc.view();
    let scene = &view.scenes[0].id;
    let fixture = &view.fixtures[0].id;
    doc.edit(
        serde_json::from_value(
            serde_json::json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{
        "id":"29999999-0000-4000-8000-000000000001","name":"测试呼吸","enabled":true,
        "fixtureIds":[fixture],"periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,
        "reverse":false,"waveform":"triangle","dutyPercent":50,
        "channels":[{"attribute":"dimmer","low":0,"high":65535}]}}}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut p = Preview::default();
    p.load_scene(&doc, 1, scene).unwrap();
    let now = p.now();
    let epoch = p.epoch;
    p.control(
        1,
        epoch,
        1,
        Command::Execute {
            step_id: scene.clone(),
        },
        now,
    )
    .unwrap();
    let rendered = serde_json::to_value(
        p.render_output(1, now + 250, stagemaster_playback::OutputMaster::default())
            .unwrap()
            .output
            .unwrap(),
    )
    .unwrap();
    let monitored = p
        .snapshot(1, now + 250, stagemaster_playback::OutputMaster::default())
        .unwrap();
    assert_eq!(monitored.control_serial, 1);
    let loaded = monitored.loaded.unwrap();
    assert_eq!(loaded.scene_id.as_ref(), Some(scene));
    assert!(loaded.sequence_id.is_empty());
    assert_eq!(rendered, serde_json::to_value(loaded.output).unwrap());
    p.control(1, epoch, 2, Command::Pause, now + 250).unwrap();
    p.control(1, epoch, 3, Command::Resume, now + 10_000)
        .unwrap();
    let frame = p
        .snapshot(
            1,
            now + 10_250,
            stagemaster_playback::OutputMaster::default(),
        )
        .unwrap();
    assert_eq!(frame.loaded.unwrap().elapsed_ms, 500);
    p.load(&doc, 1, &view.sequences[0].id).unwrap();
    assert!(p.control(1, epoch, 4, Command::Stop, p.now()).is_err());
    let frame = p
        .snapshot(1, p.now(), stagemaster_playback::OutputMaster::default())
        .unwrap();
    assert_eq!(frame.control_serial, 0);
    assert!(frame.loaded.unwrap().scene_id.is_none());
}
#[test]
fn renderer_polling_or_disconnect_cannot_own_the_show_clock() {
    let doc = document();
    let id = doc.view().sequences[0].id.clone();
    let mut observed = Preview::default();
    let mut disconnected = Preview::default();
    observed.load(&doc, 1, &id).unwrap();
    disconnected.load(&doc, 1, &id).unwrap();
    let now = observed.now().max(disconnected.now());
    observed
        .control(1, observed.epoch, 1, Command::Next, now)
        .unwrap();
    disconnected
        .control(1, disconnected.epoch, 1, Command::Next, now)
        .unwrap();
    for tick in [10, 100, 500, 1100] {
        observed
            .render_output(1, now + tick, stagemaster_playback::OutputMaster::default())
            .unwrap();
    }
    let a = serde_json::to_value(
        observed
            .snapshot(1, now + 1500, stagemaster_playback::OutputMaster::default())
            .unwrap(),
    )
    .unwrap();
    let b = serde_json::to_value(
        disconnected
            .snapshot(1, now + 1500, stagemaster_playback::OutputMaster::default())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(a, b);
    // A render-version failure also cannot reset, stop or advance the player independently.
    assert!(
        observed
            .render_output(2, now + 2000, stagemaster_playback::OutputMaster::default())
            .unwrap()
            .output
            .is_none()
    );
    let a = serde_json::to_value(
        observed
            .snapshot(1, now + 2500, stagemaster_playback::OutputMaster::default())
            .unwrap(),
    )
    .unwrap();
    let b = serde_json::to_value(
        disconnected
            .snapshot(1, now + 2500, stagemaster_playback::OutputMaster::default())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(a, b);
}
#[test]
fn stale_plans_reject_new_execution_but_allow_pause_and_release() {
    let doc = document();
    let id = doc.view().sequences[0].id.clone();
    let mut p = Preview::default();
    p.load(&doc, 3, &id).unwrap();
    let now = p.now();
    let step = p.loaded.as_ref().unwrap().steps[0].id.clone();
    p.control(
        3,
        p.epoch,
        1,
        Command::Execute {
            step_id: step.clone(),
        },
        now,
    )
    .unwrap();
    assert!(
        p.control(4, p.epoch, 2, Command::Execute { step_id: step }, now + 500)
            .is_err()
    );
    p.control(4, p.epoch, 3, Command::Pause, now + 750).unwrap();
    let snapshot = p
        .snapshot(4, now + 1000, stagemaster_playback::OutputMaster::default())
        .unwrap()
        .loaded
        .unwrap();
    assert!(snapshot.stale);
    assert_eq!(snapshot.status, "paused");
    assert_eq!(snapshot.elapsed_ms, 750);
    assert!(
        p.control(4, p.epoch, 4, Command::Resume, now + 1000)
            .is_err()
    );
    p.control(4, p.epoch, 5, Command::Stop, now + 1000).unwrap();
    assert_eq!(
        p.snapshot(4, now + 1000, stagemaster_playback::OutputMaster::default())
            .unwrap()
            .loaded
            .unwrap()
            .output
            .slots[0],
        0
    );
}
#[test]
fn repeated_and_wrong_epoch_controls_are_rejected_and_failed_load_preserves_plan() {
    let doc = document();
    let id = doc.view().sequences[0].id.clone();
    let mut p = Preview::default();
    p.load(&doc, 1, &id).unwrap();
    let now = p.now();
    let epoch = p.epoch;
    p.control(1, epoch, 1, Command::Next, now).unwrap();
    assert!(p.control(1, epoch, 1, Command::Next, now).is_err());
    assert_eq!(p.loaded.as_ref().unwrap().player.index(), Some(0));
    assert!(p.load(&doc, 1, "missing").is_err());
    assert_eq!(p.epoch, epoch);
    assert_eq!(p.loaded.as_ref().unwrap().player.index(), Some(0));
    p.load(&doc, 1, &id).unwrap();
    assert!(p.control(1, epoch, 2, Command::Next, p.now()).is_err());
    p.clear();
    assert!(
        p.snapshot(1, p.now(), stagemaster_playback::OutputMaster::default())
            .unwrap()
            .loaded
            .is_none()
    );
}

#[test]
fn output_master_never_changes_sequence_transport_or_underlying_values() {
    let doc = document();
    let id = doc.view().sequences[0].id.clone();
    let mut p = Preview::default();
    p.load(&doc, 1, &id).unwrap();
    let now = p.now();
    p.control(1, p.epoch, 1, Command::Next, now).unwrap();
    let mut control = stagemaster_playback::OutputMaster::default();
    control.set_percent(50).unwrap();
    control.set_blackout(true);
    let dark = p.snapshot(1, now + 500, control).unwrap().loaded.unwrap();
    assert_eq!(dark.status, "running");
    assert_eq!(dark.elapsed_ms, 500);
    for f in dark.output.fixtures {
        for a in f.attributes {
            if a.key == "dimmer" {
                assert_eq!(a.value, 0);
            }
        }
    }
    control.set_blackout(false);
    let reduced = p.snapshot(1, now + 750, control).unwrap().loaded.unwrap();
    assert_eq!(reduced.elapsed_ms, 750);
    let full = p
        .snapshot(1, now + 750, stagemaster_playback::OutputMaster::default())
        .unwrap()
        .loaded
        .unwrap();
    for (a, b) in reduced
        .output
        .fixtures
        .iter()
        .flat_map(|f| &f.attributes)
        .zip(full.output.fixtures.iter().flat_map(|f| &f.attributes))
    {
        assert_eq!(
            a.value,
            if a.key == "dimmer" {
                control.scale(b.value)
            } else {
                b.value
            }
        );
    }
    p.control(1, p.epoch, 2, Command::Pause, now + 750).unwrap();
    assert_eq!(
        p.snapshot(1, now + 850, control)
            .unwrap()
            .loaded
            .unwrap()
            .elapsed_ms,
        750
    );
}
