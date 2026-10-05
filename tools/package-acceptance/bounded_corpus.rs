//! Deterministic, untrusted inputs shared by software acceptance tests and an offline generator.
use sha2::{Digest, Sha256};
use stagemaster_package::{
    Error, MAX_PACKAGE_BYTES, Mapping, Output, Program, StepLabel, encode_program,
};
use stagemaster_playback::{Plan, Step};

pub struct Case {
    pub name: &'static str,
    pub bytes: Vec<u8>,
    pub expected: Error,
}
pub fn reseal(bytes: &mut [u8]) {
    let mut hash = Sha256::new();
    hash.update(&bytes[..32]);
    hash.update(&bytes[64..]);
    bytes[32..64].copy_from_slice(&hash.finalize());
}
fn prefix(count: u64) -> minicbor::Encoder<Vec<u8>> {
    let mut e = minicbor::Encoder::new(Vec::new());
    e.array(6)
        .unwrap()
        .str(stagemaster_package::COMPILER)
        .unwrap()
        .bytes(&[1; 16])
        .unwrap()
        .bytes(&[2; 16])
        .unwrap()
        .bytes(&[3; 32])
        .unwrap()
        .str("畸形包软件验收")
        .unwrap()
        .array(count)
        .unwrap();
    e
}
fn catalog(blocks: &[Vec<u8>]) -> Vec<u8> {
    let mut e = prefix(blocks.len() as u64);
    let mut offset = 0_u32;
    for (i, block) in blocks.iter().enumerate() {
        e.array(6)
            .unwrap()
            .u8(1)
            .unwrap()
            .bytes(&(i as u128 + 1).to_be_bytes())
            .unwrap()
            .str("测试节目")
            .unwrap()
            .u32(offset)
            .unwrap()
            .u32(u32::try_from(block.len()).unwrap())
            .unwrap()
            .bytes(&Sha256::digest(block))
            .unwrap();
        offset += u32::try_from(block.len()).unwrap();
    }
    e.into_writer()
}
fn envelope(catalog: &[u8], body: &[u8], count: u16) -> Vec<u8> {
    let mut bytes = vec![0; 64];
    bytes[..8].copy_from_slice(b"STMPLAY\0");
    bytes[8..16].copy_from_slice(&[1, 0, 64, 0, 1, 0, 1, 0]);
    bytes[16..20].copy_from_slice(&u32::try_from(catalog.len()).unwrap().to_le_bytes());
    bytes[20..24].copy_from_slice(
        &u32::try_from(64 + catalog.len() + body.len())
            .unwrap()
            .to_le_bytes(),
    );
    bytes[24..26].copy_from_slice(&count.to_le_bytes());
    bytes.extend_from_slice(catalog);
    bytes.extend_from_slice(body);
    reseal(&mut bytes);
    bytes
}
pub fn raw_archive(blocks: &[Vec<u8>]) -> Vec<u8> {
    envelope(
        &catalog(blocks),
        &blocks.concat(),
        u16::try_from(blocks.len()).unwrap(),
    )
}
pub fn valid_block() -> Vec<u8> {
    let p = Program {
        plan: Plan::new(
            vec![65535; 320],
            (0..17)
                .map(|i| Step {
                    target: vec![u16::try_from(65535 - i).unwrap(); 320],
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
            mappings: (17..337)
                .map(|coarse| Mapping { coarse, fine: None })
                .collect(),
        },
        labels: (1_u128..=17)
            .map(|i| StepLabel {
                id: i.to_be_bytes(),
                name: "渐变".into(),
                number: i.to_string(),
            })
            .collect(),
    };
    encode_program(&p).unwrap()
}
pub fn valid_64() -> Vec<u8> {
    raw_archive(&vec![valid_block(); 64])
}
pub fn cases() -> Vec<Case> {
    // A 2 MiB file whose authenticated catalogue honestly declares an excessive single block.
    // Both candidate lengths use a four-byte CBOR integer, so the catalogue length is stable.
    let tentative = vec![vec![0; MAX_PACKAGE_BYTES - 64]];
    let length = MAX_PACKAGE_BYTES - 64 - catalog(&tentative).len();
    let oversized = raw_archive(&[vec![0; length]]);
    assert_eq!(oversized.len(), MAX_PACKAGE_BYTES);
    let c = prefix(u64::MAX).into_writer();
    let excessive_count = envelope(&c, &vec![0; MAX_PACKAGE_BYTES - 64 - c.len()], 64);
    let mut uncovered = raw_archive(&[valid_block()]);
    uncovered.resize(MAX_PACKAGE_BYTES, 0);
    uncovered[20..24].copy_from_slice(&u32::try_from(MAX_PACKAGE_BYTES).unwrap().to_le_bytes());
    reseal(&mut uncovered);
    let mut tail_corrupt = uncovered.clone();
    *tail_corrupt.last_mut().unwrap() ^= 1;
    let mut blocks = vec![valid_block(); 64];
    blocks[63][0] = 0x9f; // Indefinite outer block array: valid package/block hashes, bad semantics.
    let late_semantics = raw_archive(&blocks);
    let mut late_digest = valid_64();
    *late_digest.last_mut().unwrap() ^= 1;
    reseal(&mut late_digest); // Preserve the original last-block digest.
    vec![
        Case {
            name: "max-oversized-program",
            bytes: oversized,
            expected: Error::Limit("单节目块 32 KiB"),
        },
        Case {
            name: "max-excessive-directory-count",
            bytes: excessive_count,
            expected: Error::Limit("数组元素数量"),
        },
        Case {
            name: "max-uncovered-file-tail",
            bytes: uncovered,
            expected: Error::Invalid("节目长度总和不符"),
        },
        Case {
            name: "max-corrupt-tail",
            bytes: tail_corrupt,
            expected: Error::Integrity,
        },
        Case {
            name: "last-64th-invalid-semantics",
            bytes: late_semantics,
            expected: Error::AtProgram {
                index: 63,
                message: Error::Version.to_string(),
            },
        },
        Case {
            name: "last-64th-invalid-digest",
            bytes: late_digest,
            expected: Error::Integrity,
        },
    ]
}
