use super::{Session, audio_tests::session};
use crate::audio::Command;
use serde_json::json;
use std::sync::{Mutex, atomic::AtomicBool};

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn add_loop(s: &mut Session) -> String {
    s.edit(
        s.generation,
        serde_json::from_value(json!({"op":"audio","command":{
        "kind":"loopRegions","command":{"kind":"add","name":"等待演员","startMs":0,"endMs":500,
        "plays":{"kind":"untilExit"}}}}))
        .unwrap(),
    )
    .unwrap();
    s.document
        .as_ref()
        .unwrap()
        .audio_timeline()
        .unwrap()
        .loop_regions[0]
        .id
        .clone()
}

fn load(s: &mut Session, dir: &tempfile::TempDir) {
    let intent = s.audio_load_intent(s.generation).unwrap().unwrap();
    let prepared = intent
        .prepare(dir.path().join("rehearsal.wav"), &AtomicBool::new(false))
        .unwrap();
    s.apply_audio_load(s.generation, prepared).unwrap();
}

#[test]
fn formal_load_seek_and_exit_use_real_identity_without_mutating_the_project() {
    let _guard = TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (mut s, dir) = session();
    let region_id = add_loop(&mut s);
    assert!(!s.audio.active());
    load(&mut s, &dir);
    let document = s.document.clone();
    let history = s.undo.len();
    let run = s.audio.position().performance.unwrap();
    let instance = run.instance.unwrap();
    assert_eq!(run.region, Some(0));
    let command = |instance: &str, pass: &str, requested: bool| Command::ExitLoop {
        instance: instance.into(),
        region_id: region_id.clone(),
        pass: pass.into(),
        requested,
    };
    assert!(
        s.audio_request(s.generation, command(&instance, "01", true))
            .is_err()
    );
    assert!(
        s.audio_request(
            s.generation,
            command(&instance, "18446744073709551616", true)
        )
        .is_err()
    );
    let pos = s
        .audio_request(s.generation, command(&instance, "1", true))
        .unwrap();
    assert!(pos.performance.unwrap().pending_exit.unwrap().requested);
    let pos = s
        .audio_request(s.generation, command(&instance, "1", false))
        .unwrap();
    assert!(!pos.performance.unwrap().pending_exit.unwrap().requested);
    let request = s
        .audio_preparation(s.generation, &Command::Seek { position_ms: 250 })
        .unwrap()
        .unwrap();
    let prepared = request.prepare(&AtomicBool::new(false)).unwrap();
    s.apply_audio_preparation(s.generation, prepared).unwrap();
    assert_eq!(s.audio.position().position_ms, 250);
    assert!(
        s.audio_request(s.generation, command(&instance, "1", true))
            .is_err()
    );
    assert_eq!(s.document, document);
    assert_eq!(s.undo.len(), history);
    assert!(!s.audio.position().playing);
}

#[test]
fn name_and_lock_edits_preserve_loaded_run_but_semantic_changes_release_it() {
    let _guard = TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (mut s, dir) = session();
    let id = add_loop(&mut s);
    load(&mut s, &dir);
    let original = s.audio.position().performance.unwrap().instance;
    let mut region = s
        .document
        .as_ref()
        .unwrap()
        .audio_timeline()
        .unwrap()
        .loop_regions[0]
        .clone();
    region.name = "演员就位".into();
    s.edit(
        s.generation,
        serde_json::from_value(json!({"op":"audio","command":{
        "kind":"loopRegions","command":{"kind":"put","region":region}}}))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(s.audio.position().performance.unwrap().instance, original);
    for locked in [true, false] {
        s.edit(s.generation, serde_json::from_value(json!({"op":"audio","command":{
            "kind":"loopRegions","command":{"kind":"edit","ids":[id],"action":{"kind":"locked","locked":locked}}}})).unwrap()).unwrap();
        assert_eq!(s.audio.position().performance.unwrap().instance, original);
    }
    s.edit(s.generation, serde_json::from_value(json!({"op":"audio","command":{
        "kind":"loopRegions","command":{"kind":"edit","ids":[id],"action":{"kind":"plays","plays":{"kind":"count","count":2}}}}})).unwrap()).unwrap();
    assert!(!s.audio.active());
    load(&mut s, &dir);
    assert_ne!(s.audio.position().performance.unwrap().instance, original);
}

#[test]
fn load_intent_precedes_slow_file_work_and_cannot_override_stop_or_new_project() {
    let _guard = TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (mut s, dir) = session();
    add_loop(&mut s);
    load(&mut s, &dir);
    let intent = s.audio_load_intent(s.generation).unwrap().unwrap();
    s.audio_request(s.generation, Command::Stop).unwrap();
    let prepared = intent
        .prepare(dir.path().join("rehearsal.wav"), &AtomicBool::new(false))
        .unwrap();
    assert!(s.apply_audio_load(s.generation, prepared).is_err());
    assert!(s.audio.active());
    assert!(s.audio.position().performance.unwrap().instance.is_none());
    let old = s.generation;
    let intent = s.audio_load_intent(old).unwrap().unwrap();
    let prepared = intent
        .prepare(dir.path().join("rehearsal.wav"), &AtomicBool::new(false))
        .unwrap();
    s.replace(
        stagemaster_project::Document::new("另一个工程").unwrap(),
        None,
    );
    assert!(s.apply_audio_load(old, prepared).is_err());
    assert!(!s.audio.active());
}
