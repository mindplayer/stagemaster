use super::*;
use serde_json::json;
use stagemaster_playback::OutputMaster;
fn document() -> Document {
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    value["entryPoints"] = json!([]);
    value["lighting"]["sequences"][0]["repeat"] = json!("loop");
    value["lighting"]["sequences"][0]["steps"][0]["delay"]["ticks"] = json!("200");
    Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap()
}
fn frame(p: &mut Preview, now: u64) -> LoadedView {
    p.snapshot(1, now, OutputMaster::default())
        .unwrap()
        .loaded
        .unwrap()
}
#[test]
fn continuous_rate_scales_delay_fade_wait_and_loops_independent_of_renderer() {
    let doc = document();
    let id = doc.view().sequences[0].id.clone();
    let mut p = Preview::default();
    p.load(&doc, 1, &id).unwrap();
    let start = p.now();
    p.control(1, p.epoch, 1, Command::Next, start).unwrap();
    let compiled = doc.compile_sequence(&id).unwrap();
    let mut oracle = Player::new(compiled.plan, 0);
    oracle.execute(0, 0).unwrap();
    p.control(1, p.epoch, 2, Command::SetRate { percent: 50 }, start + 100)
        .unwrap();
    for (real, logical) in [(200, 150), (300, 200), (800, 450), (1500, 800)] {
        let f = frame(&mut p, start + real);
        oracle.advance(logical).unwrap();
        assert_eq!(f.elapsed_ms, oracle.elapsed_ms());
        assert_eq!(
            serde_json::to_value(f.output).unwrap(),
            serde_json::to_value(compiled.output.render(oracle.values()).unwrap()).unwrap()
        );
    }
    p.control(
        1,
        p.epoch,
        3,
        Command::SetRate { percent: 200 },
        start + 1500,
    )
    .unwrap();
    // No intervening render polling; 4000 real ms produces 8000 logical ms.
    let f = frame(&mut p, start + 5500);
    oracle.advance(8800).unwrap();
    assert_eq!(
        f.step_id.as_ref(),
        Some(&p.loaded.as_ref().unwrap().steps[oracle.index().unwrap()].id)
    );
    assert_eq!(f.elapsed_ms, oracle.elapsed_ms());
    assert_eq!(f.rate_percent, 200);
    let render = p
        .render_output(1, start + 5500, OutputMaster::default())
        .unwrap()
        .output
        .unwrap();
    assert_eq!(
        serde_json::to_value(render).unwrap(),
        serde_json::to_value(f.output).unwrap()
    );
}
#[test]
fn changing_rate_while_paused_does_not_resume_and_loading_resets_it() {
    let doc = document();
    let scene = doc.view().scenes[0].id.clone();
    let mut p = Preview::default();
    p.load_scene(&doc, 1, &scene).unwrap();
    let start = p.now();
    p.control(1, p.epoch, 1, Command::Next, start).unwrap();
    p.control(1, p.epoch, 2, Command::Pause, start + 150)
        .unwrap();
    p.control(1, p.epoch, 3, Command::SetRate { percent: 25 }, start + 200)
        .unwrap();
    let f = frame(&mut p, start + 8000);
    assert_eq!(f.status, "paused");
    assert_eq!(f.elapsed_ms, 150);
    p.control(1, p.epoch, 4, Command::Resume, start + 8000)
        .unwrap();
    assert_eq!(frame(&mut p, start + 8200).elapsed_ms, 200);
    p.control(1, p.epoch, 5, Command::Stop, start + 8200)
        .unwrap();
    assert_eq!(frame(&mut p, start + 8300).rate_percent, 25);
    p.load_scene(&doc, 1, &scene).unwrap();
    let now = p.now();
    assert_eq!(frame(&mut p, now).rate_percent, 100);
}
#[test]
fn rate_commands_reject_invalid_stale_replayed_requests_without_state_changes() {
    let doc = document();
    let mut p = Preview::default();
    p.load_scene(&doc, 1, &doc.view().scenes[0].id).unwrap();
    let start = p.now();
    let epoch = p.epoch;
    p.control(1, epoch, 1, Command::Next, start).unwrap();
    for (version, request_epoch, serial, percent, now) in [
        (1, epoch, 2, 24, start + 100),
        (1, epoch, 2, 401, start + 100),
        (2, epoch, 2, 50, start + 100),
        (1, epoch + 1, 2, 50, start + 100),
        (1, epoch, 1, 50, start + 100),
    ] {
        let before = serde_json::to_value(frame(&mut p, start)).unwrap();
        assert!(
            p.control(
                version,
                request_epoch,
                serial,
                Command::SetRate { percent },
                now
            )
            .is_err()
        );
        assert_eq!(serde_json::to_value(frame(&mut p, start)).unwrap(), before);
        assert_eq!(p.last_serial, 1);
    }
    for value in [
        json!({"kind":"setRate","percent":50.5}),
        json!({"kind":"setRate","percent":-1}),
        json!({"kind":"setRate","percent":50,"extra":true}),
    ] {
        assert!(serde_json::from_value::<Command>(value).is_err());
    }
    p.clear();
    assert!(
        p.control(1, p.epoch, 1, Command::SetRate { percent: 50 }, p.now())
            .is_err()
    );
}

#[test]
fn effects_change_rate_continuously_and_keep_monitor_and_render_identical() {
    let mut doc = document();
    let view = doc.view();
    let scene = view.scenes[0].id.clone();
    doc.edit(
        serde_json::from_value(
            json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{
        "id":"29999999-0000-4000-8000-000000000001","name":"变速测试","enabled":true,
        "fixtureIds":[view.fixtures[0].id],"periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,
        "reverse":false,"waveform":"triangle","dutyPercent":50,
        "channels":[{"attribute":"dimmer","low":0,"high":65535}]}}}),
        )
        .unwrap(),
    )
    .unwrap();
    let before = doc.encode().unwrap();
    let mut p = Preview::default();
    p.load_scene(&doc, 1, &scene).unwrap();
    let start = p.now();
    p.control(1, p.epoch, 1, Command::Next, start).unwrap();
    let compiled = doc.compile_scene(&scene).unwrap();
    let mut oracle = Player::new(compiled.plan, 0);
    oracle.execute(0, 0).unwrap();
    p.control(1, p.epoch, 2, Command::SetRate { percent: 50 }, start + 250)
        .unwrap();
    for (real, logical) in [(250, 250), (500, 375), (750, 500), (1250, 750)] {
        let rendered = p
            .render_output(1, start + real, OutputMaster::default())
            .unwrap()
            .output
            .unwrap();
        let f = frame(&mut p, start + real);
        assert_eq!(f.elapsed_ms, logical);
        oracle.advance(logical).unwrap();
        let expected =
            serde_json::to_value(compiled.output.render(oracle.values()).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(rendered).unwrap(), expected);
        assert_eq!(serde_json::to_value(f.output).unwrap(), expected);
    }
    assert_eq!(doc.encode().unwrap(), before);
}
