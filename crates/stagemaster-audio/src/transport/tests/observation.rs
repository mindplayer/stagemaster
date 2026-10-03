use super::*;

#[test]
fn native_player_keeps_consumption_identity_through_pause_and_replaces_it_after_stop() {
    let _guard = serial();
    let (mut transport, _dir) = loaded();
    let ready = transport.performance_observation().unwrap().unwrap();
    assert!(!ready.requested_playing);
    assert!(ready.snapshot.consumption.is_none());
    let (output, mut mixer) = Output::virtual_device();
    transport.output = Some(output);
    transport.play().unwrap();
    consume(&mut mixer, 80);
    let first = transport.performance_observation().unwrap().unwrap();
    assert_eq!(first.instance, ready.instance);
    assert!(first.requested_playing);
    let first_consumed = first.snapshot.consumption.unwrap();
    assert!(first_consumed.frames > 0);
    transport.pause();
    consume(&mut mixer, 10); // Settle rodio's periodic pause handling, not a DAC timing claim.
    let paused = transport.performance_observation().unwrap().unwrap();
    assert!(!paused.requested_playing);
    consume(&mut mixer, 100);
    for _ in 0..10 {
        let cached = transport.performance_observation().unwrap().unwrap();
        assert_eq!(cached.snapshot.consumption, paused.snapshot.consumption);
        assert_eq!(cached.instance, first.instance);
    }
    transport.play().unwrap();
    consume(&mut mixer, 20);
    let resumed = transport.performance_observation().unwrap().unwrap();
    assert_eq!(resumed.instance, first.instance);
    assert!(
        resumed.snapshot.consumption.unwrap().frames > paused.snapshot.consumption.unwrap().frames
    );
    transport.stop();
    assert!(transport.performance_observation().unwrap().is_none());
    let replacement = transport
        .play_preparation()
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_seek(replacement).unwrap();
    let new = transport.performance_observation().unwrap().unwrap();
    assert_ne!(new.instance, first.instance);
    assert!(new.snapshot.consumption.is_none());
    assert_eq!(
        transport.position().performance.unwrap().instance,
        Some(new.instance.to_string())
    );
}

#[test]
fn execution_observation_reports_busy_while_ui_can_keep_its_explicitly_pending_cache() {
    let _guard = serial();
    let (mut transport, _dir) = loaded();
    let (output, mut mixer) = Output::virtual_device();
    transport.output = Some(output);
    transport.play().unwrap();
    consume(&mut mixer, 20);
    let first = transport.position();
    transport
        .performance
        .as_ref()
        .unwrap()
        .voice
        .as_ref()
        .unwrap()
        .control
        .block_snapshot_for_test();
    assert!(transport.performance_observation().is_err());
    let cached = transport.position();
    assert_eq!(cached.position_ms, first.position_ms);
    assert!(cached.performance.unwrap().snapshot_pending);
}
