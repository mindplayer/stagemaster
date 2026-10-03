//! Explicit, reproducible stress input. Uses the real package encoder and admission rules.
#[path = "capacity/corpus.rs"]
mod corpus;
#[path = "capacity/programs.rs"]
mod programs;
use programs::Shape;
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let output = root.join("data/MEMORY-002");
    std::fs::create_dir_all(&output)?;
    let mut programs = Vec::new();
    for shape in Shape::ALL {
        let (scale, program) = corpus::frontier(shape, 4);
        println!("{}：边界参数 {scale}；下一参数被原准入拒绝", shape.name());
        programs.push(program);
    }
    let (bytes, archive) = corpus::archive(&programs, 4)?;
    let path = output.join("capacity.smpkg");
    std::fs::write(&path, &bytes)?;
    println!(
        "{}：{} 字节／4 节目，目录预算 {}",
        path.display(),
        bytes.len(),
        archive.catalog_resident_bytes()
    );
    for (shape, entry) in Shape::ALL.iter().zip(archive.entries()) {
        println!("{}：{:?}", shape.name(), entry.usage);
    }
    let (scale, program) = corpus::frontier(Shape::Keyframes, 64);
    let (bytes, archive) = corpus::archive(&[program], 64)?;
    std::fs::write(output.join("catalog.smpkg"), &bytes)?;
    println!(
        "满目录：{} 字节／64 节目，关键帧参数 {scale}，单节目 {:?}",
        bytes.len(),
        archive.entries()[0].usage
    );
    Ok(())
}
