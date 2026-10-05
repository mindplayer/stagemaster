//! Offline software acceptance corpus. Never connects, installs or sends output to hardware.
#[path = "../../../tools/package-acceptance/bounded_corpus.rs"]
mod corpus;
#[path = "../../../tools/package-acceptance/reader_metrics.rs"]
mod metrics;
use stagemaster_package::Archive;
use std::{error::Error, fmt::Write as _, path::Path, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let dir = root.join("data/MEMORY-004");
    std::fs::create_dir_all(&dir)?;
    let mut report = String::from(
        "# 主机软件证据；不代表实板期限、堆／栈或输出波形。\n名称\t字节\t读请求\t最大读缓冲\t主机耗时微秒\t实际拒绝\n",
    );
    for case in corpus::cases() {
        let reader = metrics::Reader::new(&case.bytes);
        let start = Instant::now();
        let error = Archive::open(&reader).unwrap_err();
        let micros = start.elapsed().as_micros();
        assert_eq!(error, case.expected, "{}", case.name);
        reader.assert_complete_hash();
        let reads = reader.reads.borrow();
        let maximum = reads.iter().map(|r| r.1).max().unwrap_or(0);
        writeln!(
            report,
            "{}\t{}\t{}\t{}\t{}\t{}",
            case.name,
            case.bytes.len(),
            reads.len(),
            maximum,
            micros,
            error
        )?;
        std::fs::write(dir.join(format!("{}.smpkg", case.name)), case.bytes)?;
    }
    let bytes = corpus::valid_64();
    let archive = Archive::open(bytes.as_slice())?;
    for index in 0..archive.entries().len() {
        archive.load(bytes.as_slice(), index)?;
    }
    writeln!(report, "合法对照\t{}\t64 个节目均独立加载通过", bytes.len())?;
    std::fs::write(dir.join("valid-64.smpkg"), bytes)?;
    std::fs::write(dir.join("software-report.tsv"), &report)?;
    print!("{report}");
    Ok(())
}
