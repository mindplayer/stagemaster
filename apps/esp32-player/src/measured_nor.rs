//! Observes the official partition-bounded NOR driver; never accepts chip offsets.
use core::sync::atomic::{AtomicU32, Ordering};
use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};

static READ_US: AtomicU32 = AtomicU32::new(0);
static READS: AtomicU32 = AtomicU32::new(0);
static WRITE_US: AtomicU32 = AtomicU32::new(0);
static ERASE_US: AtomicU32 = AtomicU32::new(0);
static WRITES: AtomicU32 = AtomicU32::new(0);
static ERASES: AtomicU32 = AtomicU32::new(0);

pub struct MeasuredNor<F>(F);
impl<F> MeasuredNor<F> {
    pub fn new(flash: F) -> Self {
        Self(flash)
    }
}
fn record(start: esp_hal::time::Instant, maximum: &AtomicU32) {
    maximum.fetch_max(
        start.elapsed().as_micros().min(u64::from(u32::MAX)) as u32,
        Ordering::Relaxed,
    );
}
impl<F: ErrorType> ErrorType for MeasuredNor<F> {
    type Error = F::Error;
}
impl<F: ReadNorFlash> ReadNorFlash for MeasuredNor<F> {
    const READ_SIZE: usize = F::READ_SIZE;
    fn capacity(&self) -> usize {
        self.0.capacity()
    }
    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        crate::installation::sample_stack();
        let start = esp_hal::time::Instant::now();
        let result = self.0.read(offset, bytes);
        READS.fetch_add(1, Ordering::Relaxed);
        record(start, &READ_US);
        result
    }
}
impl<F: NorFlash> NorFlash for MeasuredNor<F> {
    const WRITE_SIZE: usize = F::WRITE_SIZE;
    const ERASE_SIZE: usize = F::ERASE_SIZE;
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        crate::installation::sample_stack();
        let start = esp_hal::time::Instant::now();
        let result = self.0.write(offset, bytes);
        record(start, &WRITE_US);
        WRITES.fetch_add(1, Ordering::Relaxed);
        // Radio gets scheduling time between bounded physical operations.
        esp_rtos::CurrentThreadHandle::get().delay(esp_hal::time::Duration::from_millis(1));
        result
    }
    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        crate::installation::sample_stack();
        let start = esp_hal::time::Instant::now();
        let result = self.0.erase(from, to);
        record(start, &ERASE_US);
        ERASES.fetch_add(1, Ordering::Relaxed);
        esp_rtos::CurrentThreadHandle::get().delay(esp_hal::time::Duration::from_millis(1));
        result
    }
}
pub fn report() {
    esp_println::println!(
        "NOR max_read_us={} max_write_us={} max_erase_us={} writes={} erases={} reads={}",
        READ_US.load(Ordering::Relaxed),
        WRITE_US.load(Ordering::Relaxed),
        ERASE_US.load(Ordering::Relaxed),
        WRITES.load(Ordering::Relaxed),
        ERASES.load(Ordering::Relaxed),
        READS.load(Ordering::Relaxed)
    );
}
