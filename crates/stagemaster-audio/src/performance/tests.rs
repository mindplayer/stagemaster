use super::{LoopExitIntent, PerformanceAudio, decode, stream};
use stagemaster_playback::{LoopPlays, LoopRegion, LoopSchedule};
use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard, atomic::AtomicBool},
    time::{Duration, Instant},
};

mod lifecycle;

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    let lock = TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    await_workers(0);
    lock
}

fn await_workers(count: usize) {
    let start = Instant::now();
    while stream::worker_count() != count {
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "decoder worker did not settle"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn audio_file(rate: u32, channels: u16, frames: u32) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("performance.wav");
    let size = frames * u32::from(channels) * 2;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + size).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(channels.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * u32::from(channels) * 2).to_le_bytes());
    bytes.extend((channels * 2).to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(size.to_le_bytes());
    for frame in 0..frames {
        for channel in 0..channels {
            bytes.extend(sample_value(frame, channel).to_le_bytes());
        }
    }
    std::fs::write(&path, bytes).unwrap();
    (dir, path)
}

fn sample_value(frame: u32, channel: u16) -> i16 {
    let value = i16::try_from(frame % 16_000 + 1).unwrap();
    if channel == 0 { value } else { -value }
}

fn samples(frames: impl IntoIterator<Item = u32>, channels: u16) -> Vec<f32> {
    frames
        .into_iter()
        .flat_map(|frame| {
            (0..channels).map(move |channel| f32::from(sample_value(frame, channel)) / 32_768.0)
        })
        .collect()
}

fn schedule(duration: u64, ranges: &[(u64, u64, LoopPlays)]) -> LoopSchedule {
    LoopSchedule::new(
        duration,
        ranges
            .iter()
            .map(|&(start, end, plays)| LoopRegion { start, end, plays })
            .collect(),
    )
    .unwrap()
}

#[test]
fn stereo_samples_repeat_multiple_regions_with_gaps_and_count_one_streaming() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 200);
    let plan = schedule(
        8,
        &[
            (1, 3, LoopPlays::Count(3)),
            (3, 5, LoopPlays::Count(2)),
            (6, 7, LoopPlays::Count(1)),
        ],
    );
    let audio = PerformanceAudio::prepare(path, 2, &plan, &AtomicBool::new(false)).unwrap();
    assert_eq!(audio.cached_bytes(), 32 * 2 * 4);
    let (source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    assert_eq!(control.snapshot().unwrap().position.tick, 0);
    let expected = (0..8)
        .chain((0..3).flat_map(|_| 8..24))
        .chain((0..2).flat_map(|_| 24..40))
        .chain(40..64)
        .map(|frame| frame + 16);
    assert_eq!(source.collect::<Vec<_>>(), samples(expected, 2));
    let status = control.snapshot().unwrap();
    assert!(status.position.ended);
    assert_eq!(status.position.tick, 64);
    assert!(status.problem.is_none());
}

#[test]
fn exit_and_cancel_are_applied_at_frames_and_pause_preserves_local_pass() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 80);
    let audio = PerformanceAudio::prepare(
        path,
        0,
        &schedule(8, &[(1, 3, LoopPlays::UntilExit)]),
        &AtomicBool::new(false),
    )
    .unwrap();
    let (mut source, control) = audio.source(1, &AtomicBool::new(false)).unwrap();
    assert_eq!(source.next(), Some(samples([8], 2)[0]));
    assert_eq!(control.snapshot().unwrap().position.tick, 8);
    assert_eq!(source.next(), Some(samples([8], 2)[1]));
    assert_eq!(control.snapshot().unwrap().position.tick, 9);
    control.request_exit(0, 1, true).unwrap();
    control.request_exit(0, 1, true).unwrap();
    control.request_exit(0, 1, false).unwrap();
    assert_eq!(
        control.snapshot().unwrap().pending_exit,
        Some(LoopExitIntent {
            region: 0,
            pass: 1,
            requested: false
        })
    );
    assert_eq!(
        source.by_ref().take(30).collect::<Vec<_>>(),
        samples(9..24, 2)
    );
    let paused = control.snapshot().unwrap().position;
    assert_eq!(
        (paused.tick, paused.pass, paused.exit_requested),
        (8, Some(2), false)
    );
    assert_eq!(control.snapshot().unwrap().position, paused);
    assert!(control.request_exit(0, 1, true).is_err());
    control.request_exit(0, 2, true).unwrap();
    assert_eq!(source.by_ref().take(2).collect::<Vec<_>>(), samples([8], 2));
    assert!(control.snapshot().unwrap().position.exit_requested);
    assert_eq!(source.collect::<Vec<_>>(), samples(9..64, 2));
    assert!(control.snapshot().unwrap().position.ended);
}

#[test]
fn a_control_arriving_between_stereo_channels_cannot_exit_the_next_region() {
    let _guard = serial();
    let (_dir, path) = audio_file(8_000, 2, 80);
    let plan = schedule(
        8,
        &[(0, 1, LoopPlays::Count(1)), (1, 3, LoopPlays::UntilExit)],
    );
    let audio = PerformanceAudio::prepare(path, 0, &plan, &AtomicBool::new(false)).unwrap();
    let (mut source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    let _ = source.by_ref().take(15).count();
    control.request_exit(0, 1, true).unwrap();
    source.next().unwrap();
    assert_eq!(control.snapshot().unwrap().position.region, Some(1));
    source.next().unwrap();
    let status = control.snapshot().unwrap();
    assert!(status.control_problem.is_some());
    assert!(!status.position.exit_requested);
    assert!(control.request_exit(0, 1, true).is_err());
    assert!(status.problem.is_none());
}

#[test]
fn source_recreation_resets_pass_and_trim_quantization_has_no_boundary_gap() {
    let _guard = serial();
    let (_dir, path) = audio_file(44_100, 1, 2_000);
    let plan = schedule(
        20,
        &[(1, 10, LoopPlays::Count(2)), (10, 12, LoopPlays::Count(2))],
    );
    let audio = PerformanceAudio::prepare(path, 1, &plan, &AtomicBool::new(false)).unwrap();
    // floor(10 * 44.1) - floor(1 * 44.1) = 397, independently of source trim.
    assert_eq!(audio.cached_bytes(), (397 + 88) * 4);
    let (source, control) = audio.source(2, &AtomicBool::new(false)).unwrap();
    assert_eq!(control.snapshot().unwrap().position.pass, Some(1));
    let expected = (88..441)
        .chain(44..441)
        .chain(441..529)
        .chain(441..529)
        .chain(529..882)
        .map(|frame| frame + 44);
    assert_eq!(source.collect::<Vec<_>>(), samples(expected, 1));
    let (mut source, _) = audio.source(20, &AtomicBool::new(false)).unwrap();
    assert_eq!(source.next(), None);
    assert!(audio.source(21, &AtomicBool::new(false)).is_err());
    assert!(audio.source(0, &AtomicBool::new(true)).is_err());
}

#[test]
fn cache_budget_counts_all_repeats_but_not_a_single_pass_or_infinite_duration() {
    let _guard = serial();
    let (_dir, path) = audio_file(48_000, 2, 480);
    let plan = schedule(3_600_000, &[(0, 3_600_000, LoopPlays::Count(1))]);
    let audio = PerformanceAudio::prepare(path.clone(), 0, &plan, &AtomicBool::new(false)).unwrap();
    assert_eq!(audio.cached_bytes(), 0);
    assert!(audio.source(0, &AtomicBool::new(false)).is_err());
    let plan = schedule(
        300_000,
        &[
            (0, 100_000, LoopPlays::UntilExit),
            (100_000, 200_000, LoopPlays::Count(2)),
        ],
    );
    let error = PerformanceAudio::prepare(path.clone(), 0, &plan, &AtomicBool::new(false))
        .err()
        .unwrap();
    assert!(error.contains("64 MiB"));
    let plan = schedule(10, &[(0, 10, LoopPlays::UntilExit)]);
    let audio = PerformanceAudio::prepare(path.clone(), 0, &plan, &AtomicBool::new(false)).unwrap();
    assert_eq!(audio.cached_bytes(), 480 * 2 * 4);
    assert!(PerformanceAudio::prepare(path, 0, &plan, &AtomicBool::new(true)).is_err());
    assert!(
        decode::checkpoint(
            &AtomicBool::new(false),
            Instant::now()
                .checked_sub(Duration::from_secs(121))
                .unwrap()
        )
        .is_err()
    );
}
