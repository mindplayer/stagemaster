//! Board-local external memory owner. Radio, stacks and the global allocator stay internal.
use core::sync::atomic::{AtomicUsize, Ordering};
use esp_hal::{
    peripherals::PSRAM,
    psram::{Psram, PsramConfig, PsramMode},
};

const MAX_PSRAM: usize = 8 * 1024 * 1024;
const CACHE_BYTES: usize = 2 * 1024 * 1024;
static DETECTED: AtomicUsize = AtomicUsize::new(0);
static CACHE: AtomicUsize = AtomicUsize::new(0);

pub fn initialize(peripheral: PSRAM<'static>) -> &'static mut [u8] {
    let psram = Psram::new(
        peripheral,
        PsramConfig {
            mode: PsramMode::OctalSpi,
            ..PsramConfig::default()
        },
    );
    let (pointer, bytes) = psram.raw_parts();
    DETECTED.store(bytes, Ordering::Relaxed);
    if pointer.is_null() || bytes == 0 || bytes > MAX_PSRAM || !bytes.is_multiple_of(4) {
        esp_println::println!(
            "MEMORY PSRAM unavailable size={}; internal-only fallback",
            bytes
        );
        return &mut [];
    }
    // SAFETY: called once before radio/second-core start, consumes the sole PSRAM token.
    // HAL owns and maps this region. No allocator or other reference is registered.
    // Volatile full-region passes exercise more than the cache capacity and detect aliases.
    let valid = unsafe { verify(pointer.cast(), bytes / 4) };
    if !valid {
        esp_println::println!("MEMORY PSRAM self-test failed; cache disabled");
        return &mut [];
    }
    let capacity = bytes.min(CACHE_BYTES);
    CACHE.store(capacity, Ordering::Relaxed);
    esp_println::println!(
        "MEMORY PSRAM self-test PASS detected={} cache={} reserved={}",
        bytes,
        capacity,
        bytes - capacity
    );
    // Psram has no destructor: HAL's mapping remains for the device lifetime.
    // SAFETY: unique ownership is transferred to the worker, never shared with core 0.
    // Initialize all bytes before forming an ordinary byte slice.
    unsafe {
        pointer.write_bytes(0, capacity);
        core::slice::from_raw_parts_mut(pointer, capacity)
    }
}

unsafe fn verify(pointer: *mut u32, words: usize) -> bool {
    for mask in [0xaaaa_5555, 0x5555_aaaa] {
        for index in 0..words {
            unsafe { pointer.add(index).write_volatile((index as u32) ^ mask) };
        }
        for index in 0..words {
            if unsafe { pointer.add(index).read_volatile() } != (index as u32) ^ mask {
                return false;
            }
        }
    }
    true
}

pub fn report() {
    let stats = esp_alloc::HEAP.stats();
    esp_println::println!(
        "MEMORY internal_used={} internal_free={} internal_peak={} psram_detected={} cache_capacity={}",
        esp_alloc::HEAP.used(),
        esp_alloc::HEAP.free(),
        stats.max_usage,
        DETECTED.load(Ordering::Relaxed),
        CACHE.load(Ordering::Relaxed),
    );
}
