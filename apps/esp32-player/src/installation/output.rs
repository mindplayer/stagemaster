//! Observes the real queue without publishing a physical DMX capability.
use super::runtime;
use core::cell::Cell;
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use stagemaster_install::Storage;
use stagemaster_install_worker::{
    ManagedWorker,
    output::{Error, LocalOutput},
};
use stagemaster_output_port::{Config, Phase};
use stagemaster_runtime::{Mode, PlaybackPolicy};

pub(super) type Output = LocalOutput<crate::board::output_probe::Driver>;
#[derive(Clone, Copy)]
struct Report {
    phase: Phase,
    quiet: bool,
    submitted: u64,
    completed: u64,
    fault: Option<Error>,
}
static LATEST: Mutex<CriticalSectionRawMutex, Cell<Option<Report>>> = Mutex::new(Cell::new(None));

pub(super) async fn initialize<S: Storage, P: PlaybackPolicy>(
    driver: crate::board::output_probe::Driver,
    boot: [u8; 16],
    worker: &mut ManagedWorker<S, P>,
) -> Option<Output> {
    let mut output = Output::new(
        Config {
            boot,
            port: 1,
            universe: 1,
            max_age_ms: 100,
            ack_timeout_ms: 100,
        },
        driver,
        runtime::now(),
    )
    .ok()?;
    loop {
        let result = output.service(worker, runtime::now());
        publish(&output);
        result.ok()?;
        if worker.state().mode == Mode::Maintenance {
            return Some(output);
        }
        embassy_time::Timer::after_millis(1).await;
    }
}
pub(super) fn publish(output: &Output) {
    let state = output.state();
    let report = Report {
        phase: state.phase,
        quiet: state.quiet,
        submitted: state.submitted.map_or(0, |f| f.serial),
        completed: state.completed.map_or(0, |f| f.serial),
        fault: output.fault(),
    };
    LATEST.lock(|slot| slot.set(Some(report)));
}
pub(super) fn report() {
    if let Some(r) = LATEST.lock(Cell::get) {
        crate::diagnostics::report_line(format_args!(
            "UART LOGIC ONLY phase={:?} quiet={} submitted={} completed={} fault={:?}; RS485=disabled",
            r.phase, r.quiet, r.submitted, r.completed, r.fault,
        ));
    }
}
