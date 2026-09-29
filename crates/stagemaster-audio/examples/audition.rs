//! Explicit native-output acceptance; not run by cargo test. Pass a short known test sound.
use stagemaster_audio::{Transport, analyze};
use std::{path::PathBuf, sync::atomic::AtomicBool, thread, time::Duration};
fn main() -> Result<(), String> {
    let path = PathBuf::from(std::env::args().nth(1).ok_or("请指定验收音频")?);
    let wave = analyze(&path, &AtomicBool::new(false))?;
    if wave.duration_ms < 4000 {
        return Err("验收文件至少 4 秒".into());
    }
    println!(
        "waveform duration={}ms buckets={} bytes={}",
        wave.duration_ms,
        wave.peaks.len(),
        wave.peaks.len() * 4
    );
    let mut transport = Transport::default();
    transport.load(path, 500, wave.duration_ms - 500)?;
    transport.set_volume(15)?;
    transport.play()?;
    thread::sleep(Duration::from_millis(600));
    let first = transport.position();
    if !first.playing || first.position_ms < 300 || first.problem.is_some() {
        return Err("本机音频消费时钟未推进".into());
    }
    transport.pause();
    let paused = transport.position().position_ms;
    thread::sleep(Duration::from_millis(220));
    assert_eq!(transport.position().position_ms, paused);
    transport.seek(1200)?;
    assert_eq!(transport.position().position_ms, 1200);
    transport.play()?;
    thread::sleep(Duration::from_millis(250));
    assert!(transport.position().position_ms >= 1300);
    let duration = transport.position().duration_ms;
    transport.seek(duration - 250)?;
    thread::sleep(Duration::from_millis(550));
    assert!(!transport.position().playing);
    assert_eq!(transport.position().position_ms, duration);
    transport.play()?;
    thread::sleep(Duration::from_millis(180));
    assert!(transport.position().position_ms < 1000);
    transport.stop();
    assert_eq!(transport.position().position_ms, 0);
    assert!(!transport.position().playing);
    println!(
        "PASS native output: pause/seek/resume/trim/end/replay/stop; first position={}ms (software cursor, not DAC latency)",
        first.position_ms
    );
    Ok(())
}
