use super::*;
use crate::OutputBinding;

fn bound() -> (Transport, MixerSource, OutputBinding) {
    let (mixer, source) = rodio::mixer::mixer(2.try_into().unwrap(), 8_000.try_into().unwrap());
    let binding = OutputBinding::new(mixer);
    (Transport::with_output(binding.clone()), source, binding)
}
fn load_plain(transport: &mut Transport, path: std::path::PathBuf) {
    let prepared = transport
        .load_performance_request(path, 100, 900, None)
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_load(prepared).unwrap();
}

#[test]
fn ordinary_music_primes_silent_health_then_confirms_play_and_pause_on_real_frames() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 8_000);
    let (mut transport, mut mixer, _) = bound();
    load_plain(&mut transport, path);
    assert!(
        transport
            .performance_observation()
            .unwrap()
            .unwrap()
            .snapshot
            .render
            .is_none()
    );
    transport.prime_performance().unwrap();
    assert!(!transport.position().playing);
    let instance = transport
        .performance_observation()
        .unwrap()
        .unwrap()
        .instance;
    for _ in 0..50 * 8 * 2 {
        assert_eq!(mixer.next(), Some(0.0));
    }
    let ready = transport.performance_observation().unwrap().unwrap();
    assert!(ready.snapshot.consumption.is_none());
    assert_eq!(ready.snapshot.position.tick, 0);
    assert_eq!(ready.snapshot.render.unwrap().applied, ready.request);
    transport.prime_performance().unwrap(); // Already primed remains the same instance.
    assert_eq!(
        transport
            .performance_observation()
            .unwrap()
            .unwrap()
            .instance,
        instance
    );
    transport.play().unwrap();
    let requested = transport.performance_observation().unwrap().unwrap();
    assert_ne!(
        requested.snapshot.render.unwrap().applied,
        requested.request
    );
    assert!(transport.prime_performance().is_err()); // Cannot silently turn an active play into pause.
    let samples: Vec<_> = mixer.by_ref().take(20 * 8 * 2).collect();
    assert!(samples.iter().any(|sample| *sample != 0.0));
    let played = transport.performance_observation().unwrap().unwrap();
    assert_eq!(played.snapshot.render.unwrap().applied, played.request);
    assert!(played.snapshot.consumption.unwrap().frames > 0);
    transport.pause();
    consume(&mut mixer, 10);
    let paused = transport.performance_observation().unwrap().unwrap();
    consume(&mut mixer, 80);
    let healthy = transport.performance_observation().unwrap().unwrap();
    assert_eq!(paused.snapshot.position, healthy.snapshot.position);
    assert_eq!(paused.snapshot.consumption, healthy.snapshot.consumption);
    assert_eq!(healthy.snapshot.render.unwrap().applied, healthy.request);
    assert!(healthy.snapshot.render.unwrap().sequence > paused.snapshot.render.unwrap().sequence);
}

#[test]
fn paused_seek_is_primed_before_play_and_linear_end_restarts_with_fresh_identity() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 8_000);
    let (mut transport, mut mixer, _) = bound();
    load_plain(&mut transport, path);
    transport.prime_performance().unwrap();
    consume(&mut mixer, 20);
    let old = transport
        .performance_observation()
        .unwrap()
        .unwrap()
        .instance;
    let request = transport.seek_preparation(750, false).unwrap().unwrap();
    transport
        .apply_seek(request.prepare(&AtomicBool::new(false)).unwrap())
        .unwrap();
    let ready = transport.performance_observation().unwrap().unwrap();
    assert_ne!(ready.instance, old);
    assert!(ready.snapshot.render.is_none());
    transport.prime_performance().unwrap();
    consume(&mut mixer, 20);
    let primed = transport.performance_observation().unwrap().unwrap();
    assert_eq!(primed.snapshot.position.tick, 750 * 8);
    assert!(primed.snapshot.consumption.is_none());
    transport.play().unwrap();
    consume(&mut mixer, 150);
    let ended = transport.performance_observation().unwrap().unwrap();
    assert_eq!(ended.snapshot.position.tick, 800 * 8);
    assert!(ended.snapshot.position.ended);
    assert!(ended.snapshot.problem.is_none());
    assert!(!transport.position().playing);
    assert!(transport.prime_performance().is_err());
    consume(&mut mixer, 50);
    assert_eq!(
        transport
            .performance_observation()
            .unwrap()
            .unwrap()
            .snapshot
            .render,
        ended.snapshot.render
    );
    let restart = transport.seek_preparation(0, false).unwrap().unwrap();
    transport
        .apply_seek(restart.prepare(&AtomicBool::new(false)).unwrap())
        .unwrap();
    transport.prime_performance().unwrap();
    consume(&mut mixer, 10);
    let fresh = transport.performance_observation().unwrap().unwrap();
    assert_ne!(fresh.instance, ended.instance);
    assert_eq!(fresh.snapshot.position.tick, 0);
    assert!(fresh.snapshot.consumption.is_none());
}

#[test]
fn explicit_output_survives_clear_reload_and_failure_never_falls_back_to_system_audio() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 8_000);
    let (mut transport, mut mixer, binding) = bound();
    for _ in 0..2 {
        load_plain(&mut transport, path.clone());
        transport.prime_performance().unwrap();
        consume(&mut mixer, 20);
        assert!(
            transport
                .performance_observation()
                .unwrap()
                .unwrap()
                .snapshot
                .render
                .is_some()
        );
        transport.clear();
    }
    load_plain(&mut transport, path.clone());
    binding.report_failure();
    assert!(
        transport
            .prime_performance()
            .unwrap_err()
            .contains("音频输出已中断")
    );
    assert!(transport.play().is_err());
    assert!(transport.performance_observation().is_err());
    assert!(transport.position().problem.is_some());
    transport.clear();
    load_plain(&mut transport, path);
    assert!(transport.prime_performance().is_err());
    assert!(transport.performance_observation().is_err());
    assert!(transport.position().problem.is_some());
    assert!(!transport.position().playing);
}

#[test]
fn bad_preparation_cannot_replace_a_primed_voice_or_open_an_output() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 8_000);
    let (mut transport, mut mixer, _) = bound();
    load_plain(&mut transport, path.clone());
    let stale = transport
        .seek_preparation(200, false)
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.prime_performance().unwrap();
    consume(&mut mixer, 10);
    let first = transport.performance_observation().unwrap().unwrap();
    assert!(transport.apply_seek(stale).is_err());
    for (start, end) in [(900, 100), (100, 100), (0, crate::MAX_DURATION_MS + 1)] {
        assert!(
            transport
                .load_performance_request(path.clone(), start, end, None)
                .is_err()
        );
    }
    assert!(
        transport
            .load_performance_request(path.clone(), 0, 800, Some(schedule()))
            .is_err()
    );
    assert!(
        transport
            .load_performance_request(path, 0, 1000, None)
            .unwrap()
            .prepare(&AtomicBool::new(true))
            .is_err()
    );
    let after = transport.performance_observation().unwrap().unwrap();
    assert_eq!(after.instance, first.instance);
    assert_eq!(after.snapshot.render, first.snapshot.render);
    assert!(after.snapshot.consumption.is_none());
    let mut ordinary = Transport::default();
    ordinary.load("unused.wav".into(), 0, 1000).unwrap();
    assert!(ordinary.prime_performance().is_err());
}

#[test]
fn failed_bound_device_invalidates_observation_and_rejects_resume_after_stop() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 8_000);
    let (mut transport, mut mixer, binding) = bound();
    load_plain(&mut transport, path);
    transport.prime_performance().unwrap();
    consume(&mut mixer, 20);
    let initial = transport.performance_observation().unwrap().unwrap();
    binding.report_failure();
    assert!(transport.performance_observation().is_err());
    assert!(transport.position().problem.is_some());
    assert!(transport.prime_performance().is_err());
    assert!(transport.play().is_err());
    assert_eq!(
        transport.position().performance.unwrap().instance,
        Some(initial.instance.to_string())
    );
    transport.stop();
    let replacement = transport
        .seek_preparation(0, false)
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_seek(replacement).unwrap();
    assert!(transport.prime_performance().is_err());
    assert!(transport.performance_observation().is_err());
    assert!(!transport.position().playing);
}
