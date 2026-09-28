//! Same-core handles, one physical driver. Reuse SDK range/encryption checks.
use alloc::{rc::Rc, vec};
use core::cell::RefCell;
use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use esp_bootloader_esp_idf::partitions::{
    self, NorFlashRegion, PARTITION_TABLE_MAX_LEN, PartitionEntry,
};
use esp_storage::FlashStorage;

type Driver = NorFlashRegion<'static, 'static, 'static>;
#[derive(Debug)]
pub enum Error {
    Busy,
    Bounds,
    Driver(partitions::Error),
}
impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::Bounds => NorFlashErrorKind::OutOfBounds,
            Self::Busy => NorFlashErrorKind::Other,
            Self::Driver(error) => error.kind(),
        }
    }
}
pub struct SharedFlash(Rc<RefCell<FlashStorage<'static>>>);
impl SharedFlash {
    pub fn new(flash: FlashStorage<'static>) -> Self {
        Self(Rc::new(RefCell::new(flash)))
    }
    pub fn partition(&self, label: &str) -> Option<PartitionNor> {
        let mut flash = self.0.try_borrow_mut().ok()?;
        // Validates every interval, including overlaps with either dedicated region.
        crate::package_storage::partition(&mut flash)?;
        let mut bytes = vec![0; PARTITION_TABLE_MAX_LEN];
        let table = partitions::read_partition_table(&mut flash, &mut bytes).ok()?;
        let mut found = None;
        for i in 0..table.len() {
            let entry = table.get_partition(i).ok()?;
            if entry.label_as_str() == label {
                if found.is_some() {
                    return None;
                }
                found = Some(entry);
            }
        }
        let entry = found?;
        if entry.raw_type() != 1 || entry.raw_subtype() != 6 || entry.flags() != 0 {
            return None;
        }
        Some(PartitionNor {
            flash: self.0.clone(),
            entry,
        })
    }
}

pub struct PartitionNor {
    flash: Rc<RefCell<FlashStorage<'static>>>,
    entry: PartitionEntry,
}
impl PartitionNor {
    pub fn offset(&self) -> u32 {
        self.entry.offset()
    }
    fn bounds(&self, offset: u32, len: usize) -> Result<(), Error> {
        let len = u32::try_from(len).map_err(|_| Error::Bounds)?;
        if offset
            .checked_add(len)
            .is_none_or(|end| end > self.entry.len())
        {
            return Err(Error::Bounds);
        }
        Ok(())
    }
}
impl ErrorType for PartitionNor {
    type Error = Error;
}
impl ReadNorFlash for PartitionNor {
    const READ_SIZE: usize = <Driver as ReadNorFlash>::READ_SIZE;
    fn capacity(&self) -> usize {
        self.entry.len() as usize
    }
    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        self.bounds(offset, bytes.len())?;
        let mut flash = self.flash.try_borrow_mut().map_err(|_| Error::Busy)?;
        let mut region = self.entry.as_flash_region(&mut flash);
        region
            .as_nor_flash()
            .map_err(Error::Driver)?
            .read(offset, bytes)
            .map_err(Error::Driver)
    }
}
impl NorFlash for PartitionNor {
    const WRITE_SIZE: usize = <Driver as NorFlash>::WRITE_SIZE;
    const ERASE_SIZE: usize = <Driver as NorFlash>::ERASE_SIZE;
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        self.bounds(offset, bytes.len())?;
        let mut flash = self.flash.try_borrow_mut().map_err(|_| Error::Busy)?;
        let mut region = self.entry.as_flash_region(&mut flash);
        region
            .as_nor_flash()
            .map_err(Error::Driver)?
            .write(offset, bytes)
            .map_err(Error::Driver)
    }
    fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let len = to.checked_sub(from).ok_or(Error::Bounds)?;
        self.bounds(from, len as usize)?;
        let mut flash = self.flash.try_borrow_mut().map_err(|_| Error::Busy)?;
        let mut region = self.entry.as_flash_region(&mut flash);
        region
            .as_nor_flash()
            .map_err(Error::Driver)?
            .erase(from, to)
            .map_err(Error::Driver)
    }
}
