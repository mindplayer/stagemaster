//! Sole TIMG1 watchdog owner. There is deliberately no recovery/disable operation.
pub mod policy;
mod progress;
use core::future::Future;
use embassy_futures::select::{Either, select};
use embassy_time::{Instant, Timer};
use esp_hal::{
    peripherals::TIMG1,
    time::Duration,
    timer::timg::{MwdtStage, TimerGroup, Wdt},
};
use policy::{Fault, Policy};
pub use progress::Progress;

pub const RESET_MS: u64 = 1_000;
pub static WORKER: Progress = Progress::new();

pub struct Watchdog {
    hardware: Wdt<TIMG1<'static>>,
    policy: Policy,
}
impl Watchdog {
    pub fn arm(timer: TIMG1<'static>) -> Self {
        let mut hardware = TimerGroup::new(timer).wdt;
        hardware.set_timeout(MwdtStage::Stage0, Duration::from_millis(RESET_MS));
        // Locked esp-hal enable sets Stage0 to ResetSystem and later stages Off.
        hardware.enable();
        hardware.feed();
        Self {
            hardware,
            policy: Policy::new(Instant::now().as_millis()),
        }
    }

    /// Run beside the real sender in the SAME task. A synchronous sender stall
    /// blocks feeding; an asynchronously completed sender stops feeding too.
    /// Dropping this future does not disable the hardware watchdog.
    pub async fn supervise(&mut self, sender: impl Future<Output = ()>) -> Fault {
        match select(sender, self.feed_while_healthy()).await {
            Either::First(()) => self.policy.trip(Fault::SenderExited).unwrap_err(),
            Either::Second(fault) => fault,
        }
    }

    async fn feed_while_healthy(&mut self) -> Fault {
        loop {
            if let Err(fault) = self
                .policy
                .check(Instant::now().as_millis(), WORKER.observed())
            {
                return fault;
            }
            self.hardware.feed();
            Timer::after_millis(25).await;
        }
    }
}
