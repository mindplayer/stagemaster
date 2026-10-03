pub use crate::{
    project::*,
    wire::{Time, poll, transmitter},
};
use core::pin::Pin;
pub use embassy_sync::blocking_mutex::raw::NoopRawMutex;
pub use stagemaster_install_worker::output::{Error, LocalOutput};
pub use stagemaster_output_port::{
    Config,
    dmx::queued::{Queue, QueuedDriver},
};
pub use stagemaster_runtime::{Action, FrameInfo, Lease, Mode, ProgramKey};
pub type Output<'a> = LocalOutput<QueuedDriver<'a, NoopRawMutex>>;

pub fn config() -> Config {
    Config {
        boot: [7; 16],
        port: 1,
        universe: 1,
        max_age_ms: 100,
        ack_timeout_ms: 200,
    }
}
pub fn prepare(
    worker: &mut Device,
    bytes: &[u8],
    output: &mut Output<'_>,
    mut run: Pin<&mut impl Future<Output = ()>>,
) -> Lease {
    output.service(worker, 0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    output.service(worker, 0).unwrap();
    assert_eq!(worker.state().mode, Mode::Maintenance);
    open(worker, 1, 0);
    let mut upload = stagemaster_transfer::Upload::new(bytes).unwrap();
    upload.connect([1; 16]).unwrap();
    transfer(worker, &mut upload, 1, 0);
    worker.finish_maintenance(0).unwrap();
    let lease = acquire(worker, 0);
    let entry = worker.catalog().last().unwrap();
    let key = ProgramKey {
        kind: entry.kind,
        id: entry.id,
    };
    apply(worker, lease, Action::Select(key), 0).unwrap();
    apply(worker, lease, Action::Load, 0).unwrap();
    lease
}
pub fn start(worker: &mut Device, lease: Lease, now: u64) {
    let step = worker.steps()[0].id;
    apply(worker, lease, Action::Start { step }, now).unwrap();
}
pub fn render(worker: &mut Device, now: u64) -> (FrameInfo, [u8; 512]) {
    worker.tick(now).unwrap();
    let mut slots = [0; 512];
    let info = worker.render(&mut slots).unwrap().unwrap();
    (info, slots)
}
pub fn settle(
    output: &mut Output<'_>,
    worker: &mut Device,
    mut run: Pin<&mut impl Future<Output = ()>>,
    time: &Time,
    ms: u64,
) {
    time.0.set(time.0.get().max(ms * 1000));
    output.service(worker, ms).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    time.0.set(time.0.get() + 16);
    assert!(poll(run.as_mut()).is_pending());
    output.service(worker, ms).unwrap();
}
