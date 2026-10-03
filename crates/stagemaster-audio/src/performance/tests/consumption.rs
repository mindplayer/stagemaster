use super::{AtomicBool, LoopPlays, PerformanceAudio, audio_file, schedule, serial};
use std::time::Instant;

#[test]
fn only_complete_decoded_audio_frames_refresh_consumption_not_reads_or_loop_intent() {
    let _guard = serial();
    let (_dir, path) = audio_file(8000, 2, 80);
    let audio = PerformanceAudio::prepare(
        path,
        0,
        &schedule(10, &[(0, 2, LoopPlays::UntilExit)]),
        &AtomicBool::new(false),
    )
    .unwrap();
    let (mut source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    assert!(control.snapshot().unwrap().consumption.is_none());
    let before = Instant::now();
    source.next().unwrap();
    assert!(control.snapshot().unwrap().consumption.is_none());
    source.next().unwrap();
    let first = control.snapshot().unwrap();
    let stamp = first.consumption.unwrap();
    assert_eq!(stamp.frames, 1);
    assert_eq!(stamp.sample_rate, 8000);
    assert_eq!(stamp.instance, control.instance());
    assert!(stamp.at >= before && stamp.at <= Instant::now());
    assert_eq!(first.position.tick, 1);
    for _ in 0..10 {
        assert_eq!(control.snapshot().unwrap().consumption, Some(stamp));
    }
    control.request_exit(0, 1, true).unwrap();
    assert_eq!(control.snapshot().unwrap().consumption, Some(stamp));
    source.next().unwrap(); // The consumer applies loop metadata before the next full stereo frame.
    let pending = control.snapshot().unwrap();
    assert!(pending.position.exit_requested);
    assert_eq!(pending.position.tick, 1);
    assert_eq!(pending.consumption, Some(stamp));
    source.next().unwrap();
    let second = control.snapshot().unwrap().consumption.unwrap();
    assert_eq!(second.frames, 2);
    assert!(second.at >= stamp.at);
    control.cancel();
    assert_eq!(source.next(), None);
    let cancelled = control.snapshot().unwrap();
    assert!(cancelled.problem.is_some());
    assert_eq!(cancelled.consumption, Some(second));
}

#[test]
fn material_loops_move_backwards_but_consumption_and_instance_do_not() {
    let _guard = serial();
    let (_dir, path) = audio_file(8000, 2, 80);
    let audio = PerformanceAudio::prepare(
        path,
        0,
        &schedule(10, &[(0, 2, LoopPlays::Count(3))]),
        &AtomicBool::new(false),
    )
    .unwrap();
    let (mut source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    let instance = control.instance();
    for frame in 1..=112 {
        assert!(source.next().is_some());
        assert!(source.next().is_some());
        let snapshot = control.snapshot().unwrap();
        let stamp = snapshot.consumption.unwrap();
        assert_eq!(stamp.instance, instance);
        assert_eq!(stamp.frames, frame);
        assert_eq!(
            snapshot.position.tick,
            if frame < 48 { frame % 16 } else { frame - 32 }
        );
    }
    let ended = control.snapshot().unwrap();
    assert!(ended.position.ended);
    for _ in 0..5 {
        assert_eq!(source.next(), None);
    }
    assert_eq!(control.snapshot().unwrap().consumption, ended.consumption);
    drop(source);
    let (_, replacement) = audio.source(0, &AtomicBool::new(false)).unwrap();
    assert_ne!(replacement.instance(), instance);
    assert!(replacement.snapshot().unwrap().consumption.is_none());
}
