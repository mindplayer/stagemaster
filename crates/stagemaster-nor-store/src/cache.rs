//! Bounded read-through cache. Mutations always invalidate before touching the device.
use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

pub const CACHE_BLOCK_BYTES: usize = 1024;
const ENTRY_BYTES: usize = CACHE_BLOCK_BYTES + 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub bytes: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CacheError<E> {
    Bounds,
    Device(E),
}
impl<E: NorFlashError> NorFlashError for CacheError<E> {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::Bounds => NorFlashErrorKind::OutOfBounds,
            Self::Device(error) => error.kind(),
        }
    }
}

/// Scratch storage may reside in PSRAM. The wrapped device must be exclusively owned,
/// and may temporarily disable that memory's cache only during its own calls.
/// No reference into scratch is passed to the device, including on cache misses.
pub struct CachedNor<'a, F> {
    device: F,
    scratch: &'a mut [u8],
    stats: CacheStats,
}
impl<'a, F: NorFlash> CachedNor<'a, F> {
    /// # Errors
    /// Reject unsupported geometry before any physical operation. Empty scratch disables caching.
    pub fn new(device: F, scratch: &'a mut [u8]) -> Result<Self, CacheError<F::Error>> {
        if F::READ_SIZE == 0 || !CACHE_BLOCK_BYTES.is_multiple_of(F::READ_SIZE) {
            return Err(CacheError::Bounds);
        }
        let len = scratch.len() / ENTRY_BYTES * ENTRY_BYTES;
        let mut cache = Self {
            device,
            scratch: &mut scratch[..len],
            stats: CacheStats {
                bytes: len,
                ..CacheStats::default()
            },
        };
        cache.invalidate();
        Ok(cache)
    }

    #[must_use]
    pub const fn stats(&self) -> CacheStats {
        self.stats
    }

    fn invalidate(&mut self) {
        for entry in self.scratch.chunks_exact_mut(ENTRY_BYTES) {
            entry[..4].fill(0xff);
        }
    }

    fn range(
        &self,
        offset: u32,
        length: usize,
        unit: usize,
    ) -> Result<usize, CacheError<F::Error>> {
        let start = usize::try_from(offset).map_err(|_| CacheError::Bounds)?;
        if unit == 0
            || !start.is_multiple_of(unit)
            || !length.is_multiple_of(unit)
            || start
                .checked_add(length)
                .is_none_or(|end| end > self.device.capacity())
        {
            return Err(CacheError::Bounds);
        }
        Ok(start)
    }
}
impl<F: NorFlash> ErrorType for CachedNor<'_, F> {
    type Error = CacheError<F::Error>;
}

#[repr(align(16))]
struct Buffer([u8; CACHE_BLOCK_BYTES]);

impl<F: NorFlash> ReadNorFlash for CachedNor<'_, F> {
    const READ_SIZE: usize = F::READ_SIZE;
    fn capacity(&self) -> usize {
        self.device.capacity()
    }

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let mut position = self.range(offset, bytes.len(), F::READ_SIZE)?;
        if self.scratch.is_empty() {
            return self.device.read(offset, bytes).map_err(CacheError::Device);
        }
        let mut remaining = bytes;
        let mut buffer = Buffer([0; CACHE_BLOCK_BYTES]);
        while !remaining.is_empty() {
            let block = position / CACHE_BLOCK_BYTES * CACHE_BLOCK_BYTES;
            let key = u32::try_from(block).map_err(|_| CacheError::Bounds)?;
            let slot =
                (block / CACHE_BLOCK_BYTES) % (self.scratch.len() / ENTRY_BYTES) * ENTRY_BYTES;
            let available = CACHE_BLOCK_BYTES.min(self.device.capacity() - block);
            let skip = position - block;
            let count = remaining.len().min(available - skip);
            if self.scratch[slot..slot + 4] == key.to_le_bytes() {
                self.stats.hits = self.stats.hits.saturating_add(1);
            } else {
                self.stats.misses = self.stats.misses.saturating_add(1);
                // Invalidate the old tag first: a failed read must never publish a partial block.
                self.scratch[slot..slot + 4].fill(0xff);
                self.device
                    .read(key, &mut buffer.0[..available])
                    .map_err(CacheError::Device)?;
                self.scratch[slot + 4..slot + 4 + available]
                    .copy_from_slice(&buffer.0[..available]);
                self.scratch[slot..slot + 4].copy_from_slice(&key.to_le_bytes());
            }
            remaining[..count]
                .copy_from_slice(&self.scratch[slot + 4 + skip..slot + 4 + skip + count]);
            remaining = &mut remaining[count..];
            position += count;
        }
        Ok(())
    }
}
impl<F: NorFlash> NorFlash for CachedNor<'_, F> {
    const WRITE_SIZE: usize = F::WRITE_SIZE;
    const ERASE_SIZE: usize = F::ERASE_SIZE;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.range(offset, bytes.len(), F::WRITE_SIZE)?;
        self.invalidate();
        self.device.write(offset, bytes).map_err(CacheError::Device)
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let length = to.checked_sub(from).ok_or(CacheError::Bounds)?;
        self.range(
            from,
            usize::try_from(length).map_err(|_| CacheError::Bounds)?,
            F::ERASE_SIZE,
        )?;
        self.invalidate();
        self.device.erase(from, to).map_err(CacheError::Device)
    }
}
