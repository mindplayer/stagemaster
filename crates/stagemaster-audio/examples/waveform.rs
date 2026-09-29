//! Export the bounded native envelope for read-only rendering acceptance.
use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    sync::atomic::AtomicBool,
};
fn main() -> Result<(), String> {
    let path = PathBuf::from(env::args_os().nth(1).ok_or("需要音乐文件路径")?);
    let wave = stagemaster_audio::analyze(&path, &AtomicBool::new(false))?;
    writeln!(
        io::stdout().lock(),
        "{{\"durationMs\":{},\"bucketMs\":{},\"channels\":{:?}}}",
        wave.duration_ms,
        wave.bucket_ms,
        wave.channels
    )
    .map_err(|e| e.to_string())
}
