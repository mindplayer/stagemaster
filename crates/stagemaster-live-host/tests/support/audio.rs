use stagemaster_audio::{PerformanceAudio, PerformanceControl, PerformanceSource};
use stagemaster_playback::LoopSchedule;
use std::sync::atomic::AtomicBool;

pub fn audio() -> (tempfile::TempDir, PerformanceSource, PerformanceControl) {
    let (dir, audio) = prepared_audio();
    let (source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
    (dir, source, control)
}
pub fn prepared_audio() -> (tempfile::TempDir, PerformanceAudio) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("consumed.wav");
    let mut wav = Vec::new();
    wav.extend(b"RIFF");
    wav.extend(32_036_u32.to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16_u32.to_le_bytes());
    wav.extend(1_u16.to_le_bytes());
    wav.extend(2_u16.to_le_bytes());
    wav.extend(8000_u32.to_le_bytes());
    wav.extend(32_000_u32.to_le_bytes());
    wav.extend(4_u16.to_le_bytes());
    wav.extend(16_u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend(32_000_u32.to_le_bytes());
    for _ in 0..16_000 {
        wav.extend(4000_i16.to_le_bytes());
    }
    std::fs::write(&path, wav).unwrap();
    let audio = PerformanceAudio::prepare(
        path,
        0,
        &LoopSchedule::new(1000, vec![]).unwrap(),
        &AtomicBool::new(false),
    )
    .unwrap();
    (dir, audio)
}
pub fn consume(source: &mut PerformanceSource, millis: usize) {
    for _ in 0..millis * 8 * 2 {
        source.next().unwrap();
    }
}
