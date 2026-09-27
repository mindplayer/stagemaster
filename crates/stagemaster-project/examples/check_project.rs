//! Read-only CLI for reproducible engineering checks; never opens a device.
use stagemaster_project::{Document, MAX_BYTES};
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).ok_or("请指定工程 JSON 路径")?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let document = Document::decode(&bytes)?;
    let started = std::time::Instant::now();
    let report = document.check();
    eprintln!(
        "检查 {} 个节目，用时 {:?}",
        report.programs.len(),
        started.elapsed()
    );
    serde_json::to_writer_pretty(std::io::stdout().lock(), &report)?;
    Ok(())
}
