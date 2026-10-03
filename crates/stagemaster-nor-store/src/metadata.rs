use crate::{Buffer, Code, Error, Shared, io};
use embedded_storage::nor_flash::NorFlash;
use sha2::{Digest, Sha256};
use stagemaster_install::{Commit, RECORD_BYTES, Record, Slot};

const MAGIC: &[u8; 8] = b"STMNOR\0\x01";
const SEAL: &[u8; 8] = b"STMDONE1";
const UNIT: usize = 256;

pub(crate) fn read<F: NorFlash>(shared: &Shared<F>, slot: Slot) -> Result<Record, Error<F::Error>> {
    let mut buffer = Buffer::erased();
    io::read(
        shared,
        shared.layout.metadata_offset(slot),
        &mut buffer.0[..2 * UNIT],
    )?;
    let b = &buffer.0;
    if b[..2 * UNIT].iter().all(|v| *v == 0xff) {
        return Ok(Record::Absent);
    }
    // The common envelope hash also covers the version. A torn erase can change that
    // byte alone; only a valid envelope may claim an unsupported version.
    let hash_valid = Sha256::digest(&b[..128])[..] == b[128..160];
    if b[..7] == MAGIC[..7] && hash_valid && b[7] != MAGIC[7] {
        return Err(Code::Format.into());
    }
    // Explicit padding predicates avoid the S3 nested-call array comparison failure
    // observed in MEMORY-002, while retaining every required byte check.
    if &b[..8] != MAGIC
        || b[12..32].iter().any(|v| *v != 0)
        || b[160..256].iter().any(|v| *v != 0xff)
        || !hash_valid
    {
        return Ok(Record::Invalid);
    }
    let capacity = u32::from_le_bytes(core::array::from_fn(|i| b[8 + i]));
    if usize::try_from(capacity).ok() != Some(shared.layout.slot_bytes()) {
        return Err(Code::Layout.into());
    }
    if &b[UNIT..UNIT + 8] != SEAL || b[UNIT + 8..2 * UNIT].iter().any(|v| *v != 0) {
        return Ok(Record::Invalid);
    }
    let mut record = [0; RECORD_BYTES];
    record.copy_from_slice(&b[32..128]);
    Ok(Record::Bytes(record))
}
pub(crate) fn write<F: NorFlash>(
    shared: &Shared<F>,
    commit: Commit,
) -> Result<(), Error<F::Error>> {
    let record = commit.encode().map_err(|_| Code::State)?;
    let mut buffer = Buffer::erased();
    buffer.0[..8].copy_from_slice(MAGIC);
    buffer.0[8..12].copy_from_slice(
        &u32::try_from(shared.layout.slot_bytes())
            .map_err(|_| Code::Capacity)?
            .to_le_bytes(),
    );
    buffer.0[12..32].fill(0);
    buffer.0[32..128].copy_from_slice(&record);
    let hash = Sha256::digest(&buffer.0[..128]);
    buffer.0[128..160].copy_from_slice(&hash);
    let offset = shared.layout.metadata_offset(commit.slot);
    io::write(shared, offset, &buffer.0[..UNIT])?;
    buffer.0[..UNIT].fill(0);
    buffer.0[..8].copy_from_slice(SEAL);
    io::write(shared, offset + UNIT, &buffer.0[..UNIT])
}
