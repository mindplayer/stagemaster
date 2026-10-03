use super::{AtomicBool, LoopPlays, PerformanceAudio, audio_file, samples, schedule, serial};

#[test]
fn stereo_pause_and_resume_apply_between_frames_without_losing_pcm_or_restamping_reads() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 80);
    let audio =
        PerformanceAudio::prepare(path, 0, &schedule(10, &[]), &AtomicBool::new(false)).unwrap();
    let (mut source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    let original = control.requested_playback();
    let initial_pause = control.request_playback(false).unwrap();
    assert!(control.snapshot().unwrap().render.is_none());
    assert_eq!(source.next(), Some(0.0));
    let requested = control.request_playback(true).unwrap(); // between paused left/right
    assert!(control.snapshot().unwrap().render.is_none());
    assert_eq!(source.next(), Some(0.0));
    let ready = control.snapshot().unwrap();
    assert!(ready.consumption.is_none());
    assert_eq!(ready.render.unwrap().applied, initial_pause);
    assert_eq!(ready.position.tick, 0);
    assert_eq!(source.next(), Some(samples([0], 2)[0]));
    let paused = control.request_playback(false).unwrap(); // between playing left/right
    assert_eq!(control.snapshot().unwrap().render, ready.render);
    assert_eq!(source.next(), Some(samples([0], 2)[1]));
    let first = control.snapshot().unwrap();
    assert_eq!(first.render.unwrap().applied, requested);
    assert_eq!(first.consumption.unwrap().frames, 1);
    assert_eq!(first.position.tick, 1);
    for _ in 0..32 {
        assert_eq!(source.next(), Some(0.0));
        assert_eq!(source.next(), Some(0.0));
        let current = control.snapshot().unwrap();
        assert_eq!(current.position, first.position);
        assert_eq!(current.consumption, first.consumption);
        assert_eq!(current.render.unwrap().applied, paused);
    }
    let health = control.snapshot().unwrap().render.unwrap();
    assert_eq!(health.sequence, 34);
    assert!(health.at >= first.render.unwrap().at);
    assert_eq!(health.instance, first.consumption.unwrap().instance);
    for _ in 0..10 {
        assert_eq!(control.snapshot().unwrap().render, Some(health));
    }
    // Latest intent wins before a new frame; none of the requests creates an observation.
    control.request_playback(true).unwrap();
    control.request_playback(false).unwrap();
    let resumed = control.request_playback(true).unwrap();
    assert!(resumed.revision > original.revision);
    assert_eq!(control.snapshot().unwrap().render, Some(health));
    assert_eq!(source.by_ref().take(2).collect::<Vec<_>>(), samples([1], 2));
    assert_eq!(control.snapshot().unwrap().render.unwrap().applied, resumed);
    assert_eq!(source.by_ref().collect::<Vec<_>>(), samples(2..80, 2));
    let ended = control.snapshot().unwrap();
    assert!(ended.position.ended);
    assert_eq!(ended.consumption.unwrap().frames, 80);
    assert_eq!(source.next(), None);
    assert_eq!(control.snapshot().unwrap().render, ended.render);
}

#[test]
fn paused_loop_accepts_exit_metadata_but_cancel_and_drop_cannot_publish_health() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 1, 80);
    let plan = schedule(10, &[(0, 2, LoopPlays::UntilExit)]);
    let audio = PerformanceAudio::prepare(path, 0, &plan, &AtomicBool::new(false)).unwrap();
    let (mut source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        source.by_ref().take(17).collect::<Vec<_>>(),
        samples((0..16).chain([0]), 1)
    );
    control.request_playback(false).unwrap();
    assert_eq!(source.next(), Some(0.0));
    let paused = control.snapshot().unwrap();
    assert_eq!(paused.position.pass, Some(2));
    control.request_exit(0, 2, true).unwrap();
    assert_eq!(control.snapshot().unwrap().render, paused.render);
    assert_eq!(source.next(), Some(0.0));
    let exited = control.snapshot().unwrap();
    assert!(exited.position.exit_requested);
    assert_eq!(exited.position.tick, paused.position.tick);
    assert_eq!(exited.position.pass, paused.position.pass);
    assert_eq!(exited.consumption, paused.consumption);
    control.cancel();
    assert!(control.request_playback(true).is_err());
    for _ in 0..3 {
        assert_eq!(source.next(), None);
    }
    assert!(control.snapshot().unwrap().problem.is_some());
    assert_eq!(control.snapshot().unwrap().render, exited.render);
    drop(source);
    assert!(control.snapshot().unwrap().stopped);
    assert_eq!(control.snapshot().unwrap().render, exited.render);
}
