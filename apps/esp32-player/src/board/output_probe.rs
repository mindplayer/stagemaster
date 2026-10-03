//! Original application assembly's UART owner. The isolated bus remains disabled.
use super::{
    dmx::{Monotonic, PreparedLine},
    logic_dmx::LogicLine,
    watchdog::Watchdog,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::{
    interrupt::Priority,
    peripherals::{FROM_CPU_INTR2, TIMG1},
};
use esp_rtos::embassy::InterruptExecutor;
use stagemaster_output_port::dmx::{
    Transmitter,
    queued::{Queue, Receiver},
};
use static_cell::StaticCell;

pub use super::output_driver::Driver;

pub struct Setup {
    line: PreparedLine,
    timer: TIMG1<'static>,
    interrupt: FROM_CPU_INTR2<'static>,
}
impl Setup {
    pub fn new(
        line: PreparedLine,
        timer: TIMG1<'static>,
        interrupt: FROM_CPU_INTR2<'static>,
    ) -> Self {
        Self {
            line,
            timer,
            interrupt,
        }
    }
    /// Call once on core 1: UART IRQ and sender executor must share that core.
    pub fn start(self, report_failure: fn()) -> Driver {
        static QUEUE: StaticCell<Queue<CriticalSectionRawMutex>> = StaticCell::new();
        let (driver, rx) = QUEUE.init(Queue::new()).split().unwrap();
        static EXECUTOR: StaticCell<InterruptExecutor<2>> = StaticCell::new();
        let spawner = EXECUTOR
            .init(InterruptExecutor::new(self.interrupt))
            .start(Priority::Priority2);
        spawner.spawn(transmit(rx, self.line, self.timer, report_failure).unwrap());
        Driver::new(driver)
    }
}

#[embassy_executor::task]
async fn transmit(
    rx: Receiver<'static, CriticalSectionRawMutex>,
    line: PreparedLine,
    timer: esp_hal::peripherals::TIMG1<'static>,
    report_failure: fn(),
) {
    // Bind in the destination executor: async HAL drivers are deliberately !Send.
    let line = LogicLine(line.into_async());
    let mut watchdog = Watchdog::arm(timer);
    crate::diagnostics::report_line(format_args!(
        "OUTPUT WATCHDOG armed reset_ms={} startup_ms={} progress_ms={}; RS485=disabled",
        super::watchdog::RESET_MS,
        super::watchdog::policy::STARTUP_MS,
        super::watchdog::policy::PROGRESS_MS,
    ));
    let fault = watchdog
        .supervise(rx.run(Transmitter::new(line, Monotonic, 40_000).unwrap()))
        .await;
    // The sender has been dropped (DE disabled). No further feeding or restart.
    crate::diagnostics::report_line(format_args!(
        "OUTPUT WATCHDOG fault={fault:?}; awaiting hardware reset; RS485=disabled"
    ));
    super::output_driver::report();
    super::dmx::metrics::report();
    report_failure();
    core::future::pending::<()>().await;
}
