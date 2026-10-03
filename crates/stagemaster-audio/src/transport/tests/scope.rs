use super::*;
use crate::{LoopRange, OutputBinding, OutputScope};
use std::path::Path;

fn bound(scope: &OutputScope) -> (Transport, MixerSource, OutputBinding) {
    let (mixer, source) = rodio::mixer::mixer(2.try_into().unwrap(), 8_000.try_into().unwrap());
    let binding = OutputBinding::new(mixer);
    let mut transport = Transport::with_output(binding.clone());
    transport.set_output_scope(scope.clone()).unwrap();
    (transport, source, binding)
}

fn scope(root: &Path) -> OutputScope {
    OutputScope::new(root.join("output")).unwrap()
}

#[test]
fn audition_keeps_ownership_after_play_returns_until_explicit_stop_or_drop() {
    let (dir, path) = audio_file(8_000, 2, 8_000);
    let scope = scope(dir.path());
    let (mut first, mut mixer, _) = bound(&scope);
    let (mut second, _, _) = bound(&scope);
    first.load(path.clone(), 0, 1000).unwrap();
    second.load(path.clone(), 0, 1000).unwrap();
    first.seek(10).unwrap();
    drop(scope.reserve().unwrap()); // Metadata/paused cursor do not claim the route.
    first.play().unwrap();
    assert!(mixer.by_ref().take(320).any(|sample| sample != 0.0));
    assert!(second.play().unwrap_err().contains("占用"));
    first.pause();
    first.seek(200).unwrap();
    assert!(scope.reserve().is_err());
    let prepared = first
        .loop_request(Some(LoopRange {
            start_ms: 100,
            end_ms: 500,
        }))
        .unwrap()
        .prepare()
        .unwrap();
    first.apply_loop(prepared).unwrap();
    first.play().unwrap();
    consume(&mut mixer, 600);
    assert!(scope.reserve().is_err());
    first.pause();
    let stale = first.loop_request(None).unwrap().prepare().unwrap();
    first.stop();
    assert!(first.apply_loop(stale).is_err());
    second.play().unwrap();
    assert!(first.play().is_err());
    second.clear();
    first.play().unwrap(); // Output binding and scope survive explicit stop.
    assert!(first.set_output_scope(scope.clone()).is_err());
    drop(first);
    second.load(path, 0, 1000).unwrap();
    second.play().unwrap(); // The same scope also survives clear/load.
    assert!(scope.reserve().is_err());
    drop(second);
    assert!(scope.reserve().is_ok());
}

#[test]
fn prepared_voice_owns_silent_pause_and_rejects_old_work_after_stop() {
    let _guard = serial();
    let (dir, path) = audio_file(8_000, 2, 8_000);
    let scope = scope(dir.path());
    let (mut transport, mut mixer, _) = bound(&scope);
    let ready = transport
        .load_performance_request(path, 0, 1000, Some(schedule()))
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_load(ready).unwrap();
    drop(scope.reserve().unwrap());
    transport.prime_performance().unwrap();
    consume(&mut mixer, 20);
    assert!(scope.reserve().is_err());
    transport.play().unwrap();
    consume(&mut mixer, 650);
    transport.pause();
    consume(&mut mixer, 20);
    assert!(scope.reserve().is_err());
    let prepared = transport
        .seek_preparation(200, false)
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_seek(prepared).unwrap();
    assert!(scope.reserve().is_err());
    let stale = transport
        .seek_preparation(300, true)
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.stop();
    let rival = scope.reserve().unwrap();
    assert!(transport.apply_seek(stale).is_err());
    let prepared = transport
        .play_preparation()
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    assert!(transport.apply_seek(prepared).unwrap_err().contains("占用"));
    drop(rival);
    let prepared = transport
        .play_preparation()
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_seek(prepared).unwrap();
    assert!(scope.reserve().is_err());
    transport.clear();
    assert!(scope.reserve().is_ok());
}

#[test]
fn natural_end_and_output_failure_keep_ownership_but_failed_first_mount_does_not() {
    let (dir, path) = audio_file(8_000, 2, 800);
    let scope = scope(dir.path());
    let (mut transport, mut mixer, binding) = bound(&scope);
    transport.load(path.clone(), 0, 100).unwrap();
    transport.play().unwrap();
    consume(&mut mixer, 150);
    assert!(!transport.position().playing);
    assert!(scope.reserve().is_err());
    binding.report_failure();
    assert!(transport.position().problem.is_some());
    assert!(scope.reserve().is_err());
    transport.stop();
    assert!(scope.reserve().is_ok());
    assert!(transport.play().is_err());
    assert!(scope.reserve().is_ok());
    transport
        .load(dir.path().join("missing.wav"), 0, 100)
        .unwrap();
    assert!(transport.play().is_err());
    assert!(scope.reserve().is_ok());
}
