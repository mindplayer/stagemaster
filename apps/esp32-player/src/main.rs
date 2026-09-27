#![no_std]
#![no_main]

extern crate alloc;

use esp_backtrace as _;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};

mod ble;
mod board;
mod diagnostics;
#[cfg(feature = "storage-readiness")]
mod package_layout;
#[cfg(feature = "storage-readiness")]
mod package_storage;
#[cfg(feature = "runtime-readiness")]
mod runtime_readiness;
mod self_test;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let _output_disabled = board::OutputDisabled::new(peripherals.GPIO21);
    esp_alloc::heap_allocator!(size: 128 * 1024);
    esp_println::println!("StageMaster PLAYER-002A: RS485 disabled, diagnostic only");
    #[cfg(feature = "storage-readiness")]
    package_storage::inspect(peripherals.FLASH);
    self_test::verify();
    esp_println::println!("SELFTEST PASS: delay fade pause resume follow jump clock stop loop");
    let mut player = self_test::benchmark_player();
    player.execute(0, 0).unwrap();
    let used_before = esp_alloc::HEAP.used();
    let start = esp_hal::time::Instant::now();
    for time in 1..=10_000 {
        player.advance(time * 25).unwrap();
        core::hint::black_box(player.values());
    }
    let micros = start.elapsed().as_micros();
    assert_eq!(used_before, esp_alloc::HEAP.used());
    esp_println::println!(
        "BENCH 10000 advances, 512 attributes: {} us; value buffers {} B; heap used {} B",
        micros,
        player.plan().value_buffer_bytes(),
        used_before
    );
    drop(player);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);
    let connector =
        esp_radio::ble::controller::BleConnector::new(peripherals.BT, Default::default()).unwrap();
    let controller = trouble_host::prelude::ExternalController::<_, 20>::new(connector);
    embassy_futures::join::join(
        ble::run(controller, diagnostics::snapshot),
        diagnostics::run(),
    )
    .await;
}
