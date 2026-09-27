use crate::{Buffer, Code, Error, IO_BYTES, Shared};
use embedded_storage::nor_flash::NorFlash;

/// Translate arbitrary logical reads through one aligned scratch buffer, including tail padding.
pub(crate) fn read<F: NorFlash>(
    shared: &Shared<F>,
    offset: usize,
    target: &mut [u8],
) -> Result<(), Error<F::Error>> {
    let end = offset.checked_add(target.len()).ok_or(Code::Bounds)?;
    if end > shared.layout.total {
        return Err(Code::Bounds.into());
    }
    if target.is_empty() {
        return Ok(());
    }
    let unit = F::READ_SIZE.max(4);
    let mut flash = shared.flash.try_borrow_mut().map_err(|_| Code::Busy)?;
    let mut buffer = Buffer::erased();
    let mut position = offset;
    let mut remaining = target;
    while !remaining.is_empty() {
        let start = position / unit * unit;
        let skip = position - start;
        let count = remaining.len().min(IO_BYTES - skip);
        let length = (skip + count).div_ceil(unit) * unit;
        flash
            .read(
                u32::try_from(start).map_err(|_| Code::Bounds)?,
                &mut buffer.0[..length],
            )
            .map_err(Error::Flash)?;
        remaining[..count].copy_from_slice(&buffer.0[skip..skip + count]);
        remaining = &mut remaining[count..];
        position += count;
    }
    Ok(())
}
pub(crate) fn compare<F: NorFlash>(
    shared: &Shared<F>,
    offset: usize,
    expected: &[u8],
) -> Result<(), Error<F::Error>> {
    let mut buffer = Buffer::erased();
    for (i, chunk) in expected.chunks(IO_BYTES).enumerate() {
        read(shared, offset + i * IO_BYTES, &mut buffer.0[..chunk.len()])?;
        if buffer.0[..chunk.len()] != *chunk {
            return Err(Code::Integrity.into());
        }
    }
    Ok(())
}
pub(crate) fn write<F: NorFlash>(
    shared: &Shared<F>,
    offset: usize,
    bytes: &[u8],
) -> Result<(), Error<F::Error>> {
    if !offset.is_multiple_of(F::WRITE_SIZE)
        || !bytes.len().is_multiple_of(F::WRITE_SIZE)
        || offset
            .checked_add(bytes.len())
            .is_none_or(|n| n > shared.layout.total)
    {
        return Err(Code::Bounds.into());
    }
    shared
        .flash
        .try_borrow_mut()
        .map_err(|_| Code::Busy)?
        .write(u32::try_from(offset).map_err(|_| Code::Bounds)?, bytes)
        .map_err(Error::Flash)?;
    compare(shared, offset, bytes)
}
pub(crate) fn erase<F: NorFlash>(
    shared: &Shared<F>,
    start: usize,
    end: usize,
) -> Result<(), Error<F::Error>> {
    if start > end
        || end > shared.layout.total
        || !start.is_multiple_of(F::ERASE_SIZE)
        || !end.is_multiple_of(F::ERASE_SIZE)
    {
        return Err(Code::Bounds.into());
    }
    let expected = Buffer::erased();
    for offset in (start..end).step_by(F::ERASE_SIZE) {
        shared
            .flash
            .try_borrow_mut()
            .map_err(|_| Code::Busy)?
            .erase(
                u32::try_from(offset).map_err(|_| Code::Bounds)?,
                u32::try_from(offset + F::ERASE_SIZE).map_err(|_| Code::Bounds)?,
            )
            .map_err(Error::Flash)?;
        for chunk in (offset..offset + F::ERASE_SIZE).step_by(IO_BYTES) {
            compare(
                shared,
                chunk,
                &expected.0[..IO_BYTES.min(offset + F::ERASE_SIZE - chunk)],
            )?;
        }
    }
    Ok(())
}
