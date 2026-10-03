use super::{Transport, output::Output};
use crate::performance::tests::{audio_file, serial};
use rodio::mixer::MixerSource;
use stagemaster_playback::{LoopPlays, LoopRegion, LoopSchedule};
use std::sync::atomic::AtomicBool;
mod observation;
mod resident;
mod scope;

fn schedule() -> LoopSchedule {
    LoopSchedule::new(
        1_000,
        vec![
            LoopRegion {
                start: 100,
                end: 300,
                plays: LoopPlays::Count(2),
            },
            LoopRegion {
                start: 400,
                end: 500,
                plays: LoopPlays::UntilExit,
            },
        ],
    )
    .unwrap()
}

fn loaded() -> (Transport, tempfile::TempDir) {
    let (dir, path) = audio_file(8_000, 2, 8_000);
    let mut transport = Transport::default();
    let ready = transport
        .load_request(path, 0, 1_000, Some(schedule()))
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_load(ready).unwrap();
    (transport, dir)
}

fn consume(source: &mut MixerSource, ms: usize) {
    for _ in 0..ms * 8 * 2 {
        source.next().unwrap();
    }
}

#[test]
fn real_player_pause_retains_pass_exit_and_source_then_stop_creates_a_new_run() {
    let _guard = serial();
    let (mut transport, _dir) = loaded();
    let (output, mut mixer) = Output::virtual_device();
    transport.output = Some(output);
    transport.play().unwrap();
    consume(&mut mixer, 850);
    let pos = transport.position();
    let run = pos.performance.unwrap();
    assert_eq!(run.region, Some(1));
    assert_eq!(run.pass.as_deref(), Some("3"));
    let instance = run.instance.unwrap();
    transport.exit_performance(&instance, 1, 3, true).unwrap();
    consume(&mut mixer, 2);
    assert!(transport.position().performance.unwrap().exit_requested);
    transport.pause();
    consume(&mut mixer, 10); // Drain any already buffered Mixer frame and confirm source pause.
    let frozen = transport.position();
    assert!(!frozen.playing);
    consume(&mut mixer, 100);
    let paused = transport.position();
    assert_eq!(paused.position_ms, frozen.position_ms);
    assert_eq!(
        paused.performance.as_ref().unwrap().pass.as_deref(),
        Some("3")
    );
    assert!(paused.performance.unwrap().exit_requested);
    assert!(transport.play_preparation().unwrap().is_none());
    transport.play().unwrap();
    consume(&mut mixer, 80);
    let running = transport.position();
    assert!(running.playing);
    assert!(running.position_ms >= 500);
    assert_eq!(
        running.performance.as_ref().unwrap().instance.as_deref(),
        Some(instance.as_str())
    );
    assert_eq!(running.performance.unwrap().region, None);
    transport.stop();
    assert_eq!(transport.position().position_ms, 0);
    assert!(transport.position().performance.unwrap().instance.is_none());
    assert!(transport.exit_performance(&instance, 1, 3, true).is_err());
    let prepared = transport
        .play_preparation()
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_seek(prepared).unwrap();
    assert_ne!(
        transport
            .position()
            .performance
            .unwrap()
            .instance
            .as_deref(),
        Some(instance.as_str())
    );
    assert!(transport.position().playing);
}

#[test]
fn stale_load_or_seek_cannot_replace_a_newer_cursor_or_another_transport() {
    let _guard = serial();
    let (mut transport, _dir) = loaded();
    let first = transport.position().performance.unwrap().instance;
    let prepared = transport
        .seek_preparation(200, false)
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    let mut other = Transport::default();
    assert!(other.apply_seek(prepared).is_err());
    assert_eq!(transport.position().performance.unwrap().instance, first);
    let prepared = transport
        .seek_preparation(200, false)
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.pause();
    assert!(transport.apply_seek(prepared).is_err());
    assert_eq!(transport.position().position_ms, 0);
    let prepared = transport
        .seek_preparation(200, false)
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.set_volume(35).unwrap(); // Volume is independent from source preparation.
    transport.apply_seek(prepared).unwrap();
    assert_eq!(transport.position().position_ms, 200);
    assert_eq!(transport.position().volume_percent, 35);
    assert_eq!(
        transport.position().performance.unwrap().pass.as_deref(),
        Some("1")
    );
    let load = transport
        .load_request("linear.wav".into(), 0, 1_000, None)
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.stop();
    assert!(transport.apply_load(load).is_err());
    assert!(transport.position().performance.is_some());
}

#[test]
fn failed_or_cancelled_preparation_and_temporary_loops_preserve_the_formal_run() {
    let _guard = serial();
    let (mut transport, _dir) = loaded();
    let original = transport.position().performance.unwrap().instance;
    let (dir, missing) = audio_file(8_000, 2, 8_000);
    std::fs::remove_file(&missing).unwrap();
    let invalid = transport
        .load_request(missing, 0, 1_000, Some(schedule()))
        .unwrap();
    assert!(invalid.prepare(&AtomicBool::new(false)).is_err());
    let cancelled = transport.seek_preparation(200, false).unwrap().unwrap();
    assert!(cancelled.prepare(&AtomicBool::new(true)).is_err());
    assert!(
        transport
            .loop_request(Some(crate::LoopRange {
                start_ms: 100,
                end_ms: 500
            }))
            .is_err()
    );
    assert_eq!(transport.position().performance.unwrap().instance, original);
    assert!(transport.seek(200).is_err());
    assert_eq!(transport.position().position_ms, 0);
    drop(dir);
}

#[test]
fn natural_end_stays_at_the_end_until_play_restarts_from_zero_with_fresh_identity() {
    let _guard = serial();
    let (mut transport, _dir) = loaded();
    let (output, mut mixer) = Output::virtual_device();
    transport.output = Some(output);
    let prepared = transport
        .seek_preparation(700, true)
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_seek(prepared).unwrap();
    let previous = transport.position().performance.unwrap().instance;
    consume(&mut mixer, 400);
    let ended = transport.position();
    assert!(!ended.playing);
    assert_eq!(ended.position_ms, 1_000);
    assert!(ended.performance.unwrap().ended);
    let prepared = transport
        .play_preparation()
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_seek(prepared).unwrap();
    assert_eq!(transport.position().position_ms, 0);
    assert_ne!(transport.position().performance.unwrap().instance, previous);
    assert!(transport.position().playing);
}
