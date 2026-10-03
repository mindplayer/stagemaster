use super::{AtomicBool, LoopPlays, PerformanceAudio, audio_file, await_workers, schedule, serial};

#[test]
fn cancelled_source_stops_at_a_frame_boundary_and_old_control_is_rejected() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 80);
    let audio = PerformanceAudio::prepare(
        path,
        0,
        &schedule(10, &[(0, 10, LoopPlays::UntilExit)]),
        &AtomicBool::new(false),
    )
    .unwrap();
    let (mut source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    source.next().unwrap();
    control.cancel();
    assert!(source.check_ready().is_err());
    source.next().unwrap(); // Complete the other channel, never a partial stereo frame.
    assert_eq!(control.snapshot().unwrap().position.tick, 1);
    assert_eq!(source.next(), None);
    assert!(
        control
            .snapshot()
            .unwrap()
            .problem
            .unwrap()
            .contains("取消")
    );
    assert!(control.request_exit(0, 1, true).is_err());
    drop(source);
    assert!(control.snapshot().unwrap().stopped);
    let (mut replacement, new_control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    assert!(replacement.next().is_some());
    assert!(new_control.snapshot().unwrap().problem.is_none());
}

#[test]
fn decoder_worker_budget_and_drop_bound_cancelled_preparations() {
    let _guard = serial();
    // More than the whole prefetch queue; each idle consumer leaves its producer blocked.
    let (_dir, path) = audio_file(8_000, 1, 160_000);
    let audio = PerformanceAudio::prepare(path, 0, &schedule(20_000, &[]), &AtomicBool::new(false))
        .unwrap();
    let (first, _) = audio.source(0, &AtomicBool::new(false)).unwrap();
    let (second, _) = audio.source(0, &AtomicBool::new(false)).unwrap();
    await_workers(2);
    let error = audio.source(0, &AtomicBool::new(false)).err().unwrap();
    assert!(error.contains("收尾"));
    drop(first);
    await_workers(1);
    drop(second);
    await_workers(0);
    let (source, _) = audio.source(19_000, &AtomicBool::new(false)).unwrap();
    assert_eq!(source.count(), 8_000);
}

#[test]
fn short_cached_input_missing_file_and_unsupported_channels_fail_before_playback() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 1, 80);
    let plan = schedule(20, &[(0, 20, LoopPlays::Count(2))]);
    assert!(PerformanceAudio::prepare(path.clone(), 0, &plan, &AtomicBool::new(false)).is_err());
    std::fs::remove_file(&path).unwrap();
    assert!(PerformanceAudio::prepare(path, 0, &plan, &AtomicBool::new(false)).is_err());
    let (_dir, path) = audio_file(8_000, 3, 80);
    assert!(PerformanceAudio::prepare(path, 0, &plan, &AtomicBool::new(false)).is_err());
}

#[test]
fn decoder_failure_after_warmup_never_advances_the_consumer_with_silence() {
    let _guard = serial();
    // One more block than the entire prefetch queue. The producer cannot discover
    // the short file until the prepared source actually consumes its first block.
    for playing in [true, false] {
        let (_dir, path) = audio_file(8_000, 1, 129 * 1_024);
        let audio =
            PerformanceAudio::prepare(path, 0, &schedule(20_000, &[]), &AtomicBool::new(false))
                .unwrap();
        let (mut source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
        source.check_ready().unwrap();
        // Receiving the first block opens one queue slot. Stop after its first frame
        // so the consumer cannot race the worker's failure before issuing pause.
        let consumed = source.by_ref().take(1).count();
        assert_eq!(consumed, 1);
        control.request_playback(playing).unwrap();
        let before = control.snapshot().unwrap();
        await_workers(0);
        assert!(source.check_ready().is_err());
        assert_eq!(source.next(), None);
        let status = control.snapshot().unwrap();
        assert!(status.problem.unwrap().contains("解码"));
        assert_eq!(status.position.tick, consumed as u64);
        assert!(!status.position.ended);
        assert_eq!(status.consumption, before.consumption);
        assert_eq!(status.render, before.render);
        assert_eq!(source.next(), None);
    }
}

#[test]
fn a_boundary_late_exit_cannot_silently_target_the_following_pass_of_the_same_region() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 80);
    let audio = PerformanceAudio::prepare(
        path,
        0,
        &schedule(10, &[(0, 1, LoopPlays::UntilExit)]),
        &AtomicBool::new(false),
    )
    .unwrap();
    let (mut source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    assert_eq!(source.by_ref().take(15).count(), 15);
    control.request_exit(0, 1, true).unwrap();
    assert_eq!(control.snapshot().unwrap().pending_exit.unwrap().pass, 1);
    source.next().unwrap();
    assert_eq!(control.snapshot().unwrap().position.pass, Some(2));
    source.next().unwrap();
    let status = control.snapshot().unwrap();
    assert!(status.control_problem.is_some());
    assert!(!status.position.exit_requested);
    assert_eq!(status.position.pass, Some(2));
}
