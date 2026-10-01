use super::*;
use serde_json::json;
use stagemaster_project::SceneEffect;
fn document() -> Document {
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    value["entryPoints"] = json!([]);
    Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap()
}
fn effect(doc: &Document, period: u32) -> SceneEffect {
    serde_json::from_value(json!({"id":"29999999-0000-4000-8000-000000000001","name":"草稿呼吸","enabled":true,
      "fixtureIds":[doc.view().fixtures[0].id],"periodMs":period,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,
      "channels":[{"attribute":"dimmer","low":0,"high":65535}]})).unwrap()
}
fn snapshot(p: &mut Preview) -> Snapshot {
    p.snapshot(1, p.now(), stagemaster_playback::OutputMaster::default())
        .unwrap()
}
#[test]
fn draft_updates_preserve_paused_time_and_match_authoritative_sampling() {
    let doc = document();
    let source = doc.encode().unwrap();
    let scene = doc.view().scenes[0].id.clone();
    let mut p = Preview {
        origin: Instant::now()
            .checked_sub(std::time::Duration::from_secs(10))
            .unwrap(),
        ..Preview::default()
    };
    p.begin_draft(&doc, 1, 0, &scene, effect(&doc, 1000), false)
        .unwrap();
    // Deterministic intermediate point, without sleeping or a second host clock.
    let loaded = p.loaded.as_mut().unwrap();
    loaded.player = Player::new(loaded.player.plan().clone(), 0);
    loaded.player.execute(0, 0).unwrap();
    loaded.player.pause(250).unwrap();
    let epoch = p.epoch;
    p.update_draft(&doc, 1, epoch, 1, effect(&doc, 2000), false)
        .unwrap();
    let frame = snapshot(&mut p).loaded.unwrap();
    assert_eq!(frame.status, "paused");
    assert_eq!(frame.elapsed_ms, 250);
    let compiled = draft::compile(&doc, &scene, effect(&doc, 2000), false).unwrap();
    let mut oracle = Player::new(compiled.plan, 0);
    oracle.execute(0, 0).unwrap();
    oracle.advance(250).unwrap();
    let expected = compiled.output.render(oracle.values()).unwrap();
    assert_eq!(
        serde_json::to_value(frame.output).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert_eq!(doc.encode().unwrap(), source);
    p.end_draft(epoch).unwrap();
    let frame = snapshot(&mut p).loaded.unwrap();
    assert!(frame.draft_effect_id.is_none());
    assert_eq!(frame.elapsed_ms, 250);
    assert_eq!(frame.status, "paused");
    let base = doc.compile_scene(&scene).unwrap();
    let mut oracle = Player::new(base.plan, 0);
    oracle.execute(0, 0).unwrap();
    oracle.advance(250).unwrap();
    assert_eq!(
        serde_json::to_value(frame.output).unwrap(),
        serde_json::to_value(base.output.render(oracle.values()).unwrap()).unwrap()
    );
}
#[test]
fn draft_failures_are_atomic_and_old_owners_cannot_change_new_playback() {
    let doc = document();
    let scene = doc.view().scenes[0].id.clone();
    let mut p = Preview::default();
    p.begin_draft(&doc, 1, 0, &scene, effect(&doc, 1000), false)
        .unwrap();
    let epoch = p.epoch;
    p.control(1, epoch, 1, Command::Pause, p.now()).unwrap();
    let before = serde_json::to_value(snapshot(&mut p)).unwrap();
    let mut other = effect(&doc, 1000);
    other.id = "29999999-0000-4000-8000-000000000002".into();
    assert!(p.update_draft(&doc, 1, epoch, 1, other, false).is_err());
    assert!(
        p.update_draft(&doc, 2, epoch, 1, effect(&doc, 1000), false)
            .is_err()
    );
    assert!(
        p.update_draft(&doc, 1, epoch, 1, effect(&doc, 0), false)
            .is_err()
    );
    assert_eq!(serde_json::to_value(snapshot(&mut p)).unwrap(), before);
    p.update_draft(&doc, 1, epoch, 2, effect(&doc, 2000), false)
        .unwrap();
    assert!(
        p.update_draft(&doc, 1, epoch, 1, effect(&doc, 3000), false)
            .is_err()
    );
    p.load_scene(&doc, 1, &scene).unwrap();
    let next = p.epoch;
    p.end_draft(epoch).unwrap();
    assert_eq!(p.epoch, next);
    assert!(
        p.update_draft(&doc, 1, epoch, 3, effect(&doc, 1000), false)
            .is_err()
    );
    assert!(snapshot(&mut p).loaded.unwrap().draft_effect_id.is_none());
}
#[test]
fn stopping_draft_then_editing_keeps_default_output_and_end_does_not_restart() {
    let doc = document();
    let scene = doc.view().scenes[0].id.clone();
    let mut p = Preview::default();
    p.begin_draft(&doc, 1, 0, &scene, effect(&doc, 1000), false)
        .unwrap();
    let epoch = p.epoch;
    p.control(1, epoch, 1, Command::Stop, p.now()).unwrap();
    p.update_draft(&doc, 1, epoch, 1, effect(&doc, 2000), true)
        .unwrap();
    assert_eq!(snapshot(&mut p).loaded.unwrap().status, "idle");
    p.end_draft(epoch).unwrap();
    assert_eq!(snapshot(&mut p).loaded.unwrap().status, "idle");
}

impl Preview {
    fn begin_draft(
        &mut self,
        doc: &Document,
        version: u64,
        epoch: u32,
        scene: &str,
        effect: SceneEffect,
        illuminate: bool,
    ) -> Result<(), String> {
        self.guard_epoch(epoch)?;
        let prepared = Preparation {
            document: doc.clone(),
            generation: 0,
            version,
            epoch,
            scene_id: scene.into(),
            effect,
            illuminate,
            serial: None,
        }
        .compile()?;
        self.install_draft(prepared, version).map(|_| ())
    }
    fn update_draft(
        &mut self,
        doc: &Document,
        version: u64,
        epoch: u32,
        serial: u32,
        effect: SceneEffect,
        illuminate: bool,
    ) -> Result<(), String> {
        let scene_id = self.draft_target(version, epoch, serial, &effect.id)?;
        let prepared = Preparation {
            document: doc.clone(),
            generation: 0,
            version,
            epoch,
            scene_id,
            effect,
            illuminate,
            serial: Some(serial),
        }
        .compile()?;
        self.install_draft(prepared, version).map(|_| ())
    }
}

#[test]
fn pause_during_compilation_wins_and_ending_rejects_the_prepared_candidate() {
    let doc = document();
    let scene = doc.view().scenes[0].id.clone();
    let mut p = Preview::default();
    p.begin_draft(&doc, 1, 0, &scene, effect(&doc, 1000), false)
        .unwrap();
    let epoch = p.epoch;
    let prepare = |serial| {
        Preparation {
            document: doc.clone(),
            generation: 0,
            version: 1,
            epoch,
            scene_id: scene.clone(),
            effect: effect(&doc, 2000),
            illuminate: false,
            serial: Some(serial),
        }
        .compile()
        .unwrap()
    };
    let candidate = prepare(1);
    p.control(1, epoch, 1, Command::Pause, p.now()).unwrap();
    let elapsed = snapshot(&mut p).loaded.unwrap().elapsed_ms;
    p.install_draft(candidate, 1).unwrap();
    let updated = snapshot(&mut p).loaded.unwrap();
    assert_eq!(updated.status, "paused");
    assert_eq!(updated.elapsed_ms, elapsed);
    let candidate = prepare(2);
    p.end_draft(epoch).unwrap();
    let restored = serde_json::to_value(snapshot(&mut p)).unwrap();
    assert!(p.install_draft(candidate, 1).is_err());
    assert_eq!(serde_json::to_value(snapshot(&mut p)).unwrap(), restored);
}
