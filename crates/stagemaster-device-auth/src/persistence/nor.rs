use super::{DATABASE_PAGES, PAGE_BYTES, STORAGE_BYTES};
use crate::Code;
use ekv::flash::{Flash, PageID};
use embedded_storage::nor_flash::NorFlash;

#[derive(Debug)]
pub enum IoError<E> {
    Flash(E),
    Code(Code),
}
pub(super) struct DatabaseFlash<F> {
    pub inner: F,
    pub faulted: bool,
}
impl<F: NorFlash> DatabaseFlash<F> {
    pub fn new(inner: F) -> Result<Self, Code> {
        if inner.capacity() != STORAGE_BYTES
            || F::ERASE_SIZE != PAGE_BYTES
            || !matches!(F::READ_SIZE, 1 | 2 | 4)
            || !matches!(F::WRITE_SIZE, 1 | 2 | 4)
        {
            return Err(Code::Geometry);
        }
        Ok(Self {
            inner,
            faulted: false,
        })
    }
    fn offset(page: PageID, offset: usize, bytes: usize) -> Result<u32, IoError<F::Error>> {
        if page.index() >= DATABASE_PAGES
            || offset > PAGE_BYTES
            || bytes > PAGE_BYTES - offset
            || !offset.is_multiple_of(4)
            || !bytes.is_multiple_of(4)
        {
            return Err(IoError::Code(Code::Geometry));
        }
        u32::try_from((page.index() + 1) * PAGE_BYTES + offset)
            .map_err(|_| IoError::Code(Code::Geometry))
    }
}
impl<F: NorFlash> Flash for DatabaseFlash<F> {
    type Error = IoError<F::Error>;
    fn page_count(&self) -> usize {
        DATABASE_PAGES
    }
    async fn read(
        &mut self,
        page: PageID,
        offset: usize,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        let result = self
            .inner
            .read(Self::offset(page, offset, data.len())?, data)
            .map_err(IoError::Flash);
        self.faulted |= result.is_err();
        result
    }
    async fn write(&mut self, page: PageID, offset: usize, data: &[u8]) -> Result<(), Self::Error> {
        let result = self
            .inner
            .write(Self::offset(page, offset, data.len())?, data)
            .map_err(IoError::Flash);
        self.faulted |= result.is_err();
        result
    }
    async fn erase(&mut self, page: PageID) -> Result<(), Self::Error> {
        let start = Self::offset(page, 0, PAGE_BYTES)?;
        let result = self
            .inner
            .erase(start, start + u32::try_from(PAGE_BYTES).unwrap())
            .map_err(IoError::Flash);
        self.faulted |= result.is_err();
        result
    }
}
