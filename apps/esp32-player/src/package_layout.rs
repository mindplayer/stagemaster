//! Board policy only. The official partition parser verifies structure and MD5 first.
pub const FLASH_BYTES: usize = 0x0100_0000;
pub const SLOT_BYTES: usize = 0x0020_0000;
pub const PARTITION_OFFSET: u32 = 0x0061_0000;
pub const PARTITION_BYTES: u32 = 0x0040_2000;
pub const PARTITION_LABEL: &str = "stmpkgs";
#[derive(Clone, Copy)]
pub struct Entry {
    pub package_label: bool,
    pub kind: u8,
    pub subtype: u8,
    pub offset: u32,
    pub length: u32,
    pub flags: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Capacity,
    Table,
    Range,
    Overlap,
    Missing,
    Duplicate,
    Profile,
}
pub fn validate(
    capacity: usize,
    count: usize,
    entry: impl Fn(usize) -> Option<Entry>,
) -> Result<usize, Error> {
    if capacity != FLASH_BYTES {
        return Err(Error::Capacity);
    }
    if count == 0 || count > 95 {
        return Err(Error::Table);
    }
    let mut selected = None;
    for index in 0..count {
        let current = entry(index).ok_or(Error::Table)?;
        let end = current
            .offset
            .checked_add(current.length)
            .ok_or(Error::Range)?;
        if current.length == 0
            || current.offset < 0x9000
            || end as usize > capacity
            || !current.offset.is_multiple_of(4096)
            || !current.length.is_multiple_of(4096)
            || (current.kind == 0 && !current.offset.is_multiple_of(65536))
        {
            return Err(Error::Range);
        }
        for previous in 0..index {
            let old = entry(previous).ok_or(Error::Table)?;
            let old_end = old.offset.checked_add(old.length).ok_or(Error::Range)?;
            if current.offset < old_end && old.offset < end {
                return Err(Error::Overlap);
            }
        }
        if current.package_label {
            if selected.is_some() {
                return Err(Error::Duplicate);
            }
            // SDK FlashRegion supports data/undefined, not custom raw type 0x40.
            // Reject older tables before its typed conversion can panic.
            if current.kind != 1
                || current.subtype != 6
                || current.offset != PARTITION_OFFSET
                || current.length != PARTITION_BYTES
                || current.flags != 0
            {
                return Err(Error::Profile);
            }
            selected = Some(index);
        }
    }
    selected.ok_or(Error::Missing)
}
