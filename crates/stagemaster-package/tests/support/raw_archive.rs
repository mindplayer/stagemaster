use sha2::{Digest, Sha256};

pub fn reseal(bytes: &mut [u8]) {
    let mut h = Sha256::new();
    h.update(&bytes[..32]);
    h.update(&bytes[64..]);
    bytes[32..64].copy_from_slice(&h.finalize());
}
pub fn raw_archive(block: &[u8], semantics: u16) -> Vec<u8> {
    let mut e = minicbor::Encoder::new(Vec::new());
    e.array(6)
        .unwrap()
        .str(if semantics == 1 {
            stagemaster_package::COMPILER
        } else {
            stagemaster_package::SNAP_COMPILER
        })
        .unwrap()
        .bytes(&[1; 16])
        .unwrap()
        .bytes(&[2; 16])
        .unwrap()
        .bytes(&[3; 32])
        .unwrap()
        .str("独立编码")
        .unwrap()
        .array(1)
        .unwrap()
        .array(6)
        .unwrap()
        .u8(1)
        .unwrap()
        .bytes(&[4; 16])
        .unwrap()
        .str("节目")
        .unwrap()
        .u32(0)
        .unwrap()
        .u32(u32::try_from(block.len()).unwrap())
        .unwrap()
        .bytes(&Sha256::digest(block))
        .unwrap();
    let catalog = e.into_writer();
    let mut bytes = vec![0; 64];
    bytes[..8].copy_from_slice(b"STMPLAY\0");
    bytes[8..16].copy_from_slice(&[1, 0, 64, 0, 1, 0, 1, 0]);
    bytes[12..14].copy_from_slice(&semantics.to_le_bytes());
    bytes[16..20].copy_from_slice(&u32::try_from(catalog.len()).unwrap().to_le_bytes());
    let total = 64 + catalog.len() + block.len();
    bytes[20..24].copy_from_slice(&u32::try_from(total).unwrap().to_le_bytes());
    bytes[24] = 1;
    bytes.extend(catalog);
    bytes.extend_from_slice(block);
    reseal(&mut bytes);
    bytes
}
