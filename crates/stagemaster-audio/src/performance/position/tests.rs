use super::*;
use std::sync::Arc;

const PLAY: PlaybackRequest = PlaybackRequest {
    revision: 0,
    playing: true,
};
const PAUSE: PlaybackRequest = PlaybackRequest {
    revision: 1,
    playing: false,
};

fn value(tick: u64) -> LoopPosition {
    LoopPosition {
        tick,
        repeated_ticks: tick * 2,
        region: Some(usize::try_from(tick % 128).unwrap()),
        pass: Some(tick + 1),
        exit_requested: tick.is_multiple_of(2),
        ended: false,
    }
}

#[test]
fn concurrent_readers_never_observe_a_torn_position_and_final_state_is_available() {
    let position = Arc::new(PublishedPosition::new(value(0), 48_000).unwrap());
    let writer = position.clone();
    let thread = std::thread::spawn(move || {
        for tick in 1..=100_000 {
            writer.publish(value(tick));
        }
    });
    for _ in 0..100_000 {
        if let Ok(read) = position.read() {
            assert_eq!(read, value(read.tick));
        }
    }
    thread.join().unwrap();
    assert_eq!(position.read().unwrap(), value(100_000));
}

#[test]
fn concurrent_consumption_keeps_time_frame_count_and_loop_position_together() {
    let position = Arc::new(PublishedPosition::new(value(0), 48_000).unwrap());
    let writer = position.clone();
    let started = Instant::now();
    let thread = std::thread::spawn(move || {
        for frame in 1..=100_000 {
            writer.frame(value(frame), PLAY).unwrap();
        }
    });
    let mut previous = None::<Consumption>;
    for _ in 0..100_000 {
        if let Ok(read) = position.observe()
            && let Some(stamp) = read.consumption
        {
            assert_eq!(read.position, value(stamp.frames));
            assert_eq!(stamp.instance, position.instance());
            assert_eq!(stamp.sample_rate, 48_000);
            assert!(stamp.at >= started && stamp.at <= Instant::now());
            let render = read.render.unwrap();
            assert_eq!(
                (render.instance, render.sample_rate, render.at),
                (stamp.instance, stamp.sample_rate, stamp.at)
            );
            assert_eq!(render.sequence, stamp.frames);
            assert_eq!(render.applied, PLAY);
            if let Some(old) = previous {
                assert!(stamp.frames >= old.frames && stamp.at >= old.at);
            }
            previous = Some(stamp);
        }
    }
    thread.join().unwrap();
    let read = position.observe().unwrap();
    assert_eq!(read.position, value(100_000));
    assert_eq!(read.consumption.unwrap().frames, 100_000);
}

#[test]
fn initial_busy_metadata_and_exhausted_consumption_cannot_invent_fresh_frames() {
    let position = PublishedPosition::new(value(0), 48_000).unwrap();
    assert!(PublishedPosition::new(value(0), 0).is_err());
    let initial = position.observe().unwrap();
    assert!(initial.consumption.is_none() && initial.render.is_none());
    position.frame(value(1), PLAY).unwrap();
    let before = position.observe().unwrap();
    let mut metadata = value(1);
    metadata.exit_requested = !metadata.exit_requested;
    position.publish(metadata);
    assert_eq!(
        position.observe().unwrap(),
        Publication {
            position: metadata,
            ..before
        }
    );
    position.frames.store(u64::MAX, Ordering::Relaxed);
    let exhausted = position.observe().unwrap();
    assert!(position.frame(value(2), PLAY).is_err());
    assert_eq!(position.observe().unwrap(), exhausted);
    position.block_for_test();
    assert!(position.observe().is_err());
}

#[test]
fn concurrent_pause_health_never_pairs_a_new_position_with_an_old_applied_request() {
    let position = Arc::new(PublishedPosition::new(value(0), 48_000).unwrap());
    let writer = position.clone();
    let thread = std::thread::spawn(move || {
        for seq in 1..=100_000_u64 {
            let playing = !seq.is_multiple_of(2);
            writer
                .frame(
                    value(seq.div_ceil(2)),
                    PlaybackRequest {
                        revision: seq,
                        playing,
                    },
                )
                .unwrap();
        }
    });
    for _ in 0..100_000 {
        if let Ok(read) = position.observe()
            && let Some(render) = read.render
        {
            let consumed = read.consumption.unwrap();
            assert_eq!(render.applied.revision, render.sequence);
            assert_eq!(render.applied.playing, !render.sequence.is_multiple_of(2));
            assert_eq!(consumed.frames, render.sequence.div_ceil(2));
            assert_eq!(read.position, value(consumed.frames));
            assert!(render.at >= consumed.at);
            if render.applied.playing {
                assert_eq!(render.at, consumed.at);
            }
        }
    }
    thread.join().unwrap();
    let last = position.observe().unwrap();
    assert_eq!(last.render.unwrap().sequence, 100_000);
    assert_eq!(last.consumption.unwrap().frames, 50_000);
}

#[test]
fn paused_health_has_no_consumption_and_render_exhaustion_preserves_last_snapshot() {
    let position = PublishedPosition::new(value(0), 8_000).unwrap();
    position.frame(value(0), PAUSE).unwrap();
    let paused = position.observe().unwrap();
    assert!(paused.consumption.is_none());
    assert_eq!(paused.render.unwrap().applied, PAUSE);
    position.render_sequence.store(u64::MAX, Ordering::Relaxed);
    let exhausted = position.observe().unwrap();
    assert!(position.frame(value(0), PAUSE).is_err());
    assert!(position.frame(value(1), PLAY).is_err());
    assert_eq!(position.observe().unwrap(), exhausted);
}
