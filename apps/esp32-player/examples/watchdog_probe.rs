//! Authorized board fault injection only. Never enables RS485 or opens flash storage.
#![no_std]
#![no_main]
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    rtc_cntl::{SocResetReason, reset_reason},
    system::{Cpu, Stack},
    timer::timg::TimerGroup,
};
use esp_rtos::embassy::Executor;
use static_cell::{ConstStaticCell, StaticCell};
#[path = "../src/board/watchdog/mod.rs"]
mod watchdog;
const CASE: &str = env!("STAGEMASTER_WATCHDOG_CASE");
static INJECT: AtomicBool = AtomicBool::new(false);
esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn worker() {
    loop {
        if INJECT.load(Ordering::Acquire) {
            match CASE {
                "worker-stall" => loop {
                    core::hint::spin_loop();
                },
                "worker-failed" => {
                    watchdog::WORKER.fail();
                    core::future::pending::<()>().await;
                }
                _ => {}
            }
        }
        watchdog::WORKER.beat();
        Timer::after_millis(25).await;
    }
}

#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let _disabled = Output::new(p.GPIO21, Level::Low, OutputConfig::default());
    let _tx_mark = Output::new(p.GPIO17, Level::High, OutputConfig::default());
    esp_alloc::heap_allocator!(size: 128 * 1024);
    let reset = reset_reason(Cpu::ProCpu);
    esp_println::println!(
        "WATCHDOG PROBE case={} reset={:?}; RS485=disabled",
        CASE,
        reset
    );
    let timer = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timer.timer0, p.FROM_CPU_INTR0);
    // A hardware watchdog reboot is observed without triggering a reset loop.
    if reset == Some(SocResetReason::CoreMwdt1) {
        let mut seconds = 0_u64;
        loop {
            Timer::after_secs(1).await;
            seconds += 1;
            esp_println::println!(
                "WATCHDOG RECOVERED case={} stable_seconds={}; RS485=disabled",
                CASE,
                seconds
            );
        }
    }
    assert!(matches!(
        CASE,
        "sender-stall" | "sender-exit" | "worker-stall" | "worker-failed"
    ));
    static STACK: ConstStaticCell<Stack<8192>> = ConstStaticCell::new(Stack::new());
    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    esp_rtos::start_second_core(p.CPU_CTRL, p.FROM_CPU_INTR1, STACK.take(), || {
        EXECUTOR.init(Executor::new()).run(|spawner| {
            spawner.spawn(worker().unwrap());
        });
    });
    let mut watchdog = watchdog::Watchdog::arm(p.TIMG1);
    esp_println::println!(
        "WATCHDOG ARMED case={} reset_ms={} fault_after_ms=3000",
        CASE,
        watchdog::RESET_MS
    );
    let fault = watchdog
        .supervise(async {
            Timer::after_secs(3).await;
            esp_println::println!(
                "WATCHDOG INJECT case={} worker_progress={}",
                CASE,
                watchdog::WORKER.observed()
            );
            INJECT.store(true, Ordering::Release);
            match CASE {
                "sender-stall" => loop {
                    core::hint::spin_loop();
                },
                "sender-exit" => {}
                _ => core::future::pending::<()>().await,
            }
        })
        .await;
    esp_println::println!("WATCHDOG TRIPPED case={} fault={:?}", CASE, fault);
    core::future::pending::<()>().await;
}
