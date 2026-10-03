use crate::{
    support::{project::*, *},
    *,
};
use stagemaster_runtime::{Action, ProgramKey};

#[test]
fn installed_program_passes_real_queue_and_keeps_only_latest_pending_frame() {
    let project = Project::new();
    let mut runtime = Project::runtime();
    let mut queue = LocalQueue::new();
    let (driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let mut run = Box::pin(rx.run(tx));
    let mut port = Port::new(
        Config {
            boot: [1; 16],
            port: 1,
            universe: 1,
            max_age_ms: 100,
            ack_timeout_ms: 50,
        },
        driver,
        0,
    )
    .unwrap();
    port.shutdown(0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    port.poll(0).unwrap();
    prepare(&project, &mut runtime, &mut port);
    port.select(source(), false, 0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    port.poll(0).unwrap();
    let permit = port.state().permit.unwrap();
    let mut frames = Vec::new();
    for serial in 1..=20 {
        let ms = (serial - 1) * 25;
        time.0.set(ms * 1000);
        runtime.tick(ms).unwrap();
        let mut slots = [0; 512];
        let info = runtime.render(&mut slots).unwrap().unwrap();
        port.submit(
            permit,
            Sample {
                serial,
                sampled_ms: info.sampled_ms,
                universe: info.universe,
                slots: &slots,
            },
            ms,
        )
        .unwrap();
        port.poll(ms).unwrap();
        assert!(poll(run.as_mut()).is_pending());
        assert_ne!(port.state().completed.map(|f| f.serial), Some(serial));
        time.0.set(ms * 1000 + 16);
        assert!(poll(run.as_mut()).is_pending());
        port.poll(ms).unwrap();
        assert_eq!(port.state().completed.unwrap().serial, serial);
        let bytes = &wire.0.borrow().bytes;
        assert_eq!(bytes[bytes.len() - 513], 0);
        assert_eq!(&bytes[bytes.len() - 512..], slots);
        frames.push(slots);
    }
    assert!(frames.windows(2).any(|p| p[0] != p[1]));
    assert!(runtime.state().owner.is_none());

    time.0.set(500_000);
    wire.0.borrow_mut().hold = Some("write");
    for serial in 21..=24 {
        port.submit(
            permit,
            Sample {
                serial,
                sampled_ms: 500,
                universe: 1,
                slots: &[u8::try_from(serial).unwrap(); 512],
            },
            500,
        )
        .unwrap();
        port.poll(500).unwrap();
        assert!(poll(run.as_mut()).is_pending());
    }
    time.0.set(500_016);
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(port.state().in_flight.unwrap().serial, 21);
    assert_eq!(port.state().pending.unwrap().serial, 24);
    wire.0.borrow_mut().hold = None;
    assert!(poll(run.as_mut()).is_pending());
    port.poll(500).unwrap();
    assert_eq!(port.state().completed.unwrap().serial, 21);
    assert!(poll(run.as_mut()).is_pending());
    time.0.set(500_032);
    assert!(poll(run.as_mut()).is_pending());
    port.poll(500).unwrap();
    assert_eq!(port.state().completed.unwrap().serial, 24);
    assert_eq!(wire.0.borrow().bytes.len(), 22 * 513);
    {
        let bytes = &wire.0.borrow().bytes;
        assert_eq!(&bytes[20 * 513 + 1..21 * 513], &[21; 512]);
        assert_eq!(&bytes[21 * 513 + 1..], &[24; 512]);
    }
    port.stop(permit, 500).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    port.poll(500).unwrap();
    assert!(port.state().quiet);
    port.with_quiescent(|| ()).unwrap();
}

fn prepare(project: &Project, runtime: &mut Device, port: &mut Port<impl Driver>) {
    port.with_quiescent(|| {
        let maintenance = runtime
            .confirm_quiescent(runtime.quiescence_request().unwrap(), 0)
            .unwrap();
        runtime
            .finish_maintenance(maintenance, 0, || project.installer.snapshot().map(Some))
            .unwrap();
    })
    .unwrap();
    let lease = acquire(runtime, 0);
    let entry = &runtime.catalog()[0];
    let key = ProgramKey {
        kind: entry.kind,
        id: entry.id,
    };
    control(runtime, lease, Action::Select(key), 0);
    control(runtime, lease, Action::Load, 0);
    let first = runtime.steps()[0].id;
    control(runtime, lease, Action::Start { step: first }, 0);
    runtime.release(lease, 0).unwrap();
}
