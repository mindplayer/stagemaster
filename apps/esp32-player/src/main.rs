#![no_std]
#![no_main]

#[cfg(all(feature = "binding-readiness", feature = "worker-write-test"))]
compile_error!("binding readiness and local package write probe must be separate images");

extern crate alloc;

use esp_backtrace as _;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};

#[cfg(feature = "binding-readiness")]
mod bindings;
mod ble;
mod board;
mod diagnostics;
mod identity;
#[cfg(feature = "worker-readiness")]
mod installation;
#[cfg(feature = "worker-readiness")]
mod measured_nor;
#[cfg(feature = "storage-readiness")]
mod package_layout;
#[cfg(feature = "storage-readiness")]
mod package_storage;
#[cfg(all(feature = "runtime-readiness", not(feature = "worker-readiness")))]
mod runtime_readiness;
mod self_test;
#[cfg(feature = "session-readiness")]
mod session_probe;
#[cfg(feature = "binding-readiness")]
mod shared_flash;
#[cfg(all(
    feature = "worker-readiness",
    not(any(feature = "binding-readiness", feature = "application-gatt"))
))]
mod worker_probe;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let mut output_disabled = board::OutputDisabled::new(peripherals.GPIO21, peripherals.GPIO17);
    esp_alloc::heap_allocator!(size: 128 * 1024);
    #[cfg(any(feature = "installation-gatt", feature = "application-gatt"))]
    esp_println::println!("StageMaster DEVICE-002: authenticated installation, RS485 disabled");
    #[cfg(not(any(feature = "installation-gatt", feature = "application-gatt")))]
    esp_println::println!("StageMaster DEVICE-002: diagnostics, RS485 disabled");
    #[cfg(all(feature = "storage-readiness", not(feature = "worker-readiness")))]
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
    let identity = identity::Identity::capture();
    #[cfg(feature = "session-readiness")]
    session_probe::verify(&identity);
    #[cfg(feature = "worker-readiness")]
    {
        installation::start(
            peripherals.CPU_CTRL,
            peripherals.FROM_CPU_INTR1,
            peripherals.FLASH,
            identity.boot(),
            &output_disabled,
        );
        #[cfg(not(any(feature = "binding-readiness", feature = "application-gatt")))]
        _spawner.spawn(worker_probe::run(identity.boot()).unwrap());
    }
    embassy_futures::join::join(
        ble::run(controller, identity, diagnostics::snapshot, |connected| {
            output_disabled.connected(connected);
        }),
        diagnostics::run(),
    )
    .await;
}
