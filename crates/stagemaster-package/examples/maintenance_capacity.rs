//! Generate a larger valid catalogue for maintenance tests, without raising admission limits.
use stagemaster_package::{Archive, Builder, Kind, Mapping, Output, Program, Source, StepLabel};
use stagemaster_playback::{Plan, Step};
use std::{error::Error, path::Path};

fn program(attributes: usize, steps: usize) -> Program {
    Program {
        plan: Plan::new(
            vec![65535; attributes],
            (0..steps)
                .map(|index| Step {
                    target: vec![u16::try_from(65535 - index).unwrap(); attributes],
                    delay_ms: 0,
                    fade_ms: 1000,
                    wait_ms: Some(1000),
                })
                .collect(),
            true,
        )
        .unwrap(),
        output: Output {
            universe: 1,
            mappings: (1..=attributes)
                .map(|index| Mapping {
                    coarse: u16::try_from(index).unwrap(),
                    fine: None,
                })
                .collect(),
        },
        labels: (0..steps)
            .map(|index| StepLabel {
                id: (index as u128 + 1).to_be_bytes(),
                name: "渐变".into(),
                number: (index + 1).to_string(),
            })
            .collect(),
    }
}

fn encode(
    attributes: usize,
    steps: usize,
) -> Result<(Vec<u8>, Archive), stagemaster_package::Error> {
    let plan = program(attributes, steps);
    let mut builder = Builder::new(Source {
        project_id: [0x72; 16],
        revision_id: [0x75; 16],
        snapshot_digest: [0x76; 32],
        project_name: "ESP32 容量验收".into(),
    });
    for index in 0..64_u128 {
        builder.add(Kind::Sequence, (index + 1).to_be_bytes(), "渐变节目", &plan)?;
    }
    builder.finish()
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let mut largest = Vec::new();
    // Distinct mapping/step tradeoffs; this is a representative search, not a proof
    // of the maximum encoded size of every legal semantic shape.
    for attributes in [32, 64, 96, 128, 160, 192, 256, 320, 384, 448, 512] {
        let (mut low, mut high) = (1, 128);
        encode(attributes, low)?;
        while low < high {
            let mid = low + (high - low).div_ceil(2);
            if encode(attributes, mid).is_ok() {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        assert!(low < 128 && encode(attributes, low + 1).is_err());
        let (bytes, archive) = encode(attributes, low)?;
        println!(
            "属性={attributes} 步骤={low} 字节={} 加载预算={}; 下一步骤被原准入拒绝",
            bytes.len(),
            archive.entries()[0].usage.loader_peak_bytes
        );
        if bytes.len() > largest.len() {
            largest = bytes;
        }
    }
    assert!(largest.len() > 799_587);
    let destination = root.join("data/MEMORY-003");
    std::fs::create_dir_all(&destination)?;
    let path = destination.join("large.smpkg");
    std::fs::write(&path, &largest)?;
    let archive = Archive::open(largest.as_slice())?;
    for index in 0..archive.entries().len() {
        archive.load(largest.as_slice(), index)?;
    }
    println!(
        "已独立校验并逐项加载：{}，{} 字节／64 节目",
        path.display(),
        largest.len()
    );
    Ok(())
}
