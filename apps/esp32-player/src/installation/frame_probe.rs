//! Board adapter: publish bounded snapshots, format only on the diagnostic core.
use super::{
    frame_metrics::{Metrics, Report, Sample, fingerprint},
    runtime, runtime_io,
};
use core::cell::Cell;
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use stagemaster_install::Storage;
use stagemaster_install_worker::{ManagedWorker, runtime_queue::Endpoint};
use stagemaster_runtime::PlaybackPolicy;

static LATEST: Mutex<CriticalSectionRawMutex, Cell<Option<Report>>> = Mutex::new(Cell::new(None));

#[derive(Default)]
pub(super) struct Sampler {
    metrics: Metrics,
    published_us: u64,
}
impl Sampler {
    pub(super) fn command(&mut self) {
        self.metrics.command();
    }

    pub(super) fn failed(&mut self) {
        self.metrics.failed();
        self.publish();
    }

    pub(super) fn sample<S: Storage, P: PlaybackPolicy>(
        &mut self,
        endpoint: &mut Endpoint,
        worker: &mut ManagedWorker<S, P>,
        frame: &mut [u8; 512],
        backpressured: bool,
    ) -> bool {
        let start_us = esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros();
        let result = endpoint
            .tick(worker, runtime::now(), runtime_io::live)
            .and_then(|()| worker.render(frame));
        let hash = if matches!(result, Ok(Some(_))) {
            fingerprint(frame)
        } else {
            0
        };
        let state = worker.state();
        let online = runtime_io::live().is_some_and(|live| live.grant(runtime::now()).is_some());
        let finish_us = esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros();
        self.metrics.record(
            &Sample {
                start_us,
                finish_us,
                online,
                backpressured,
                state,
            },
            &result,
            hash,
        );
        if result.is_err() || finish_us.saturating_sub(self.published_us) >= 1_000_000 {
            self.published_us = finish_us;
            self.publish();
        }
        result.is_ok()
    }

    fn publish(&self) {
        let report = self.metrics.report();
        LATEST.lock(|slot| slot.set(Some(report)));
    }
}

pub(super) fn report() {
    let latest = LATEST.lock(Cell::get);
    if let Some(report) = latest {
        // Never format or perform USB I/O while holding the cross-core publication lock.
        crate::diagnostics::report_line(format_args!("{}", report));
    }
}
