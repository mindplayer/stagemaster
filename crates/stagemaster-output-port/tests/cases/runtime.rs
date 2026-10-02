use crate::support::{project::*, *};
use stagemaster_output_port::*;
use stagemaster_runtime::{Action, Mode, ProgramKey, Status};

#[test]
fn real_installed_project_survives_control_disconnect_then_yields_to_external_output() {
    let project = Project::new();
    let mut runtime = Project::runtime();
    let (mut port, driver) = setup();
    port.shutdown(0).unwrap();
    driver.quiet();
    port.poll(0).unwrap();
    bind(&project, &mut runtime, &mut port, 0);
    let lease = acquire(&mut runtime, 0);
    let key = ProgramKey {
        kind: runtime.catalog()[0].kind,
        id: runtime.catalog()[0].id,
    };
    control(&mut runtime, lease, Action::Select(key), 0);
    control(&mut runtime, lease, Action::Load, 0);
    let step = runtime.steps()[0].id;
    control(&mut runtime, lease, Action::Start { step }, 0);
    let permit = ready(&mut port, &driver);
    let instance = runtime.state().instance;
    runtime.release(lease, 0).unwrap();
    let mut frames = Vec::new();
    for serial in 1..=20 {
        let now = (serial - 1) * 25;
        runtime.tick(now).unwrap();
        let mut slots = [0; 512];
        let info = runtime.render(&mut slots).unwrap().unwrap();
        assert_eq!(info.instance, instance);
        port.submit(
            permit,
            Sample {
                serial,
                sampled_ms: info.sampled_ms,
                universe: info.universe,
                slots: &slots,
            },
            now,
        )
        .unwrap();
        port.poll(now).unwrap();
        driver.finish(now + 1);
        port.poll(now + 1).unwrap();
        frames.push(slots);
    }
    assert!(runtime.state().owner.is_none());
    assert_eq!(runtime.state().status, Some(Status::Running));
    assert!(frames.windows(2).any(|pair| pair[0] != pair[1]));
    assert_eq!(driver.0.borrow().sent, frames);
    port.select(source(SourceKind::External), true, 500)
        .unwrap();
    runtime.tick(500).unwrap();
    let mut slots = [0; 512];
    runtime.render(&mut slots).unwrap().unwrap();
    assert_eq!(
        port.submit(permit, sample(21, 500, &slots), 500),
        Err(Code::Permit)
    );
    driver.quiet();
    port.poll(501).unwrap();
    let external = port.state().permit.unwrap();
    port.submit(external, sample(1, 501, &[66; 512]), 501)
        .unwrap();
    port.poll(501).unwrap();
    driver.finish(502);
    port.poll(502).unwrap();
    assert_eq!(driver.0.borrow().sent.last(), Some(&[66; 512]));
    // Runtime stop produces profile defaults and must not stop another owner.
    let lease = acquire(&mut runtime, 503);
    control(&mut runtime, lease, Action::Stop, 503);
    runtime.render(&mut slots).unwrap().unwrap();
    assert_ne!(slots[0], 0);
    assert_eq!(port.stop(permit, 503), Err(Code::Permit));
    assert_eq!(port.state().permit, Some(external));
    control(&mut runtime, lease, Action::BeginMaintenance, 503);
    assert_eq!(runtime.state().mode, Mode::Quiescing);
    assert_eq!(
        port.with_quiescent(|| panic!("external owner still active")),
        Err(Code::NotQuiet)
    );
    port.stop(external, 504).unwrap();
    assert_eq!(
        port.with_quiescent(|| panic!("await driver")),
        Err(Code::NotQuiet)
    );
    driver.quiet();
    port.poll(505).unwrap();
    bind(&project, &mut runtime, &mut port, 505);
    assert_eq!(runtime.state().mode, Mode::Operation);
    assert!(port.state().quiet);
    assert_eq!(driver.0.borrow().sent.len(), 21);
}

fn bind(project: &Project, runtime: &mut Device, port: &mut Port<SoftwareDriver>, now: u64) {
    port.with_quiescent(|| {
        let maintenance = runtime
            .confirm_quiescent(runtime.quiescence_request().unwrap(), now)
            .unwrap();
        runtime
            .finish_maintenance(maintenance, now, || project.installer.snapshot().map(Some))
            .unwrap();
    })
    .unwrap();
}
