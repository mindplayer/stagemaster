use crate::support::{project::*, *};
use core::pin::pin;
use stagemaster_output_port::Sample;
use stagemaster_runtime::{Action, ProgramKey};

#[test]
fn installed_dynamic_program_generates_exact_wire_slots_without_control_connection() {
    let project = Project::new();
    let mut runtime = Project::runtime();
    let (mut tx, wire, time) = transmitter();
    let (mut port, driver) = setup();
    port.shutdown(0).unwrap();
    let quiet_request = driver.0.borrow_mut().stop.take().unwrap();
    driver.0.borrow_mut().event = Some(finish(pin!(tx.quiesce(quiet_request)), &time).unwrap());
    port.poll(0).unwrap();
    port.with_quiescent(|| {
        let maintenance = runtime
            .confirm_quiescent(runtime.quiescence_request().unwrap(), 0)
            .unwrap();
        runtime
            .finish_maintenance(maintenance, 0, || project.installer.snapshot().map(Some))
            .unwrap();
    })
    .unwrap();
    let lease = acquire(&mut runtime, 0);
    let entry = &runtime.catalog()[0];
    let key = ProgramKey {
        kind: entry.kind,
        id: entry.id,
    };
    control(&mut runtime, lease, Action::Select(key), 0);
    control(&mut runtime, lease, Action::Load, 0);
    let step = runtime.steps()[0].id;
    control(&mut runtime, lease, Action::Start { step }, 0);
    runtime.release(lease, 0).unwrap();
    port.select(source(), false, 0).unwrap();
    let quiet_request = driver.0.borrow_mut().stop.take().unwrap();
    driver.0.borrow_mut().event = Some(finish(pin!(tx.quiesce(quiet_request)), &time).unwrap());
    port.poll(0).unwrap();
    let permit = port.state().permit.unwrap();
    let mut frames = Vec::new();
    for frame in 0..20 {
        let ms = frame * 25;
        time.0.set(ms * 1000);
        runtime.tick(ms).unwrap();
        let mut slots = [0; 512];
        let info = runtime.render(&mut slots).unwrap().unwrap();
        assert!(info.instance.is_some());
        wire.0.borrow_mut().bytes.clear();
        port.submit(
            permit,
            Sample {
                serial: frame + 1,
                sampled_ms: info.sampled_ms,
                universe: info.universe,
                slots: &slots,
            },
            ms,
        )
        .unwrap();
        port.poll(ms).unwrap();
        let (ticket, queued, until) = driver.0.borrow_mut().frame.take().unwrap();
        driver.0.borrow_mut().event =
            Some(finish(pin!(tx.send(ticket, &queued, until)), &time).unwrap());
        port.poll(time.0.get() / 1000).unwrap();
        assert_eq!(port.state().completed.unwrap().serial, frame + 1);
        let actual = &wire.0.borrow().bytes;
        assert_eq!(actual[0], 0);
        assert_eq!(&actual[1..], slots);
        frames.push(slots);
    }
    assert!(frames.windows(2).any(|p| p[0] != p[1]));
    assert!(runtime.state().owner.is_none());
}
