use super::{DATABASE_PAGES, PAGE_BYTES, STORAGE_BYTES, nor::IoError};
use crate::Code;
use ekv::config as c;
use embedded_storage::nor_flash::NorFlash;

// EKV environment overrides or transitive feature unification must fail at build
// time instead of silently creating a second format with the same identifier.
const _: () = assert!(
    c::CRC
        && c::ALIGN == 4
        && c::PAGE_SIZE == PAGE_BYTES
        && c::MAX_PAGE_COUNT == DATABASE_PAGES
        && c::ERASE_VALUE == 255
        && c::MAX_CHUNK_SIZE == 256
        && c::BRANCHING_FACTOR == 2
        && c::MAX_KEY_SIZE == 64
        && c::MAX_VALUE_SIZE == 1024
        && c::SCRATCH_PAGE_COUNT == 4
);
const HEADER_BYTES: usize = 256;
const TAG: &[u8] = b"SM-BINDING-VAULT-v1;EKV=1.0.0;CRC=1;ALIGN=4;PAGE=4096;PAGES=32;CHUNK=256;BRANCH=2;KEY=64;VALUE=1024;SCRATCH=4;ERASE=255;";
fn header() -> [u8; HEADER_BYTES] {
    let mut header = [0; HEADER_BYTES];
    header[..TAG.len()].copy_from_slice(TAG);
    header
}
pub(super) fn check<F: NorFlash>(flash: &mut F) -> Result<(), IoError<F::Error>> {
    let mut bytes = [0; HEADER_BYTES];
    flash.read(0, &mut bytes).map_err(IoError::Flash)?;
    if bytes != header() {
        return Err(IoError::Code(Code::Format));
    }
    // Reserve the entire format page, not only the prefix recognized by this version.
    for offset in (HEADER_BYTES..PAGE_BYTES).step_by(HEADER_BYTES) {
        flash
            .read(u32::try_from(offset).unwrap(), &mut bytes)
            .map_err(IoError::Flash)?;
        if bytes != [0xff; HEADER_BYTES] {
            return Err(IoError::Code(Code::Format));
        }
    }
    Ok(())
}
pub(super) fn require_blank<F: NorFlash>(flash: &mut F) -> Result<(), IoError<F::Error>> {
    let mut bytes = [0; HEADER_BYTES];
    for offset in (0..STORAGE_BYTES).step_by(HEADER_BYTES) {
        flash
            .read(u32::try_from(offset).unwrap(), &mut bytes)
            .map_err(IoError::Flash)?;
        if bytes != [0xff; HEADER_BYTES] {
            return Err(IoError::Code(Code::NotBlank));
        }
    }
    Ok(())
}
pub(super) fn write<F: NorFlash>(flash: &mut F) -> Result<(), IoError<F::Error>> {
    flash.write(0, &header()).map_err(IoError::Flash)?;
    check(flash)
}
