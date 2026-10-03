//! Probe-only playback workload and status. The transport receives a snapshot callback.
use core::sync::atomic::{AtomicU32, Ordering};
use embassy_time::{Instant, Timer};

#[cfg(feature = "runtime-gatt")]
mod chunked;

pub fn report_line(args: core::fmt::Arguments<'_>) {
    #[cfg(feature = "runtime-gatt")]
    // esp-println's ordinary macro holds its interrupt-masking lock for the
    // entire formatted line. Release between bounded writes so timer wakes run.
    chunked::write_line(args, esp_println::Printer::write_bytes).unwrap();
    #[cfg(not(feature = "runtime-gatt"))]
    esp_println::println!("{}", args);
}

static LIVE_TICKS: AtomicU32 = AtomicU32::new(0);

pub fn snapshot() -> [u8; 20] {
    let mut bytes = [0; 20];
    bytes[0] = 1;
    bytes[1] = 3; // self-test passed; RS485 transmitter disabled
    bytes[4..8].copy_from_slice(&(Instant::now().as_millis() as u32).to_le_bytes());
    bytes[8..12].copy_from_slice(&LIVE_TICKS.load(Ordering::Relaxed).to_le_bytes());
    bytes[12..16].copy_from_slice(&(esp_alloc::HEAP.used() as u32).to_le_bytes());
    bytes[16..20].copy_from_slice(&(esp_alloc::HEAP.free() as u32).to_le_bytes());
    bytes
}

pub async fn run() {
    let mut player = crate::self_test::benchmark_player();
    let start = Instant::now();
    player.execute(0, 0).unwrap();
    let mut ticks = 0u32;
    loop {
        player.advance(start.elapsed().as_millis()).unwrap();
        core::hint::black_box(player.values());
        ticks = ticks.wrapping_add(1);
        LIVE_TICKS.store(ticks, Ordering::Relaxed);
        if ticks.is_multiple_of(200) {
            report_line(format_args!(
                "LIVE ticks={} elapsed_ms={} value={} heap_used={}",
                ticks,
                start.elapsed().as_millis(),
                player.values()[0],
                esp_alloc::HEAP.used()
            ));
            #[cfg(feature = "runtime-gatt")]
            crate::installation::report();
        }
        Timer::after_millis(25).await;
    }
}
