#![cfg(feature = "application")]
#[allow(dead_code)]
mod maintenance_support;
#[allow(dead_code)]
mod operation_support;
use maintenance_support::{fixture, install};
use operation_support::{Peer, all};
use stagemaster_device_auth::application::{Permissions, Scope};
use stagemaster_install_worker::operations::{Detail, Failure, Operation};
use stagemaster_runtime::{Action, Code, Mode};

#[test]
fn catalog_and_step_pages_preserve_exact_identity_and_boundaries() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut controller = Peer::new(&device, 9, all(), 0);
    controller.prepare(&mut device);
    let mut reader = Peer::new(&device, 10, Permissions::only(Scope::Observe), 0);
    let count = u16::try_from(device.catalog().len()).unwrap();
    for index in 0..count {
        let reply = reader
            .send(&mut device, Operation::Catalog { index }, 0)
            .unwrap();
        assert_eq!(reply.program_count, count);
        let Detail::Program(Some(entry)) = reply.result.unwrap() else {
            panic!("missing entry")
        };
        let original = &device.catalog()[usize::from(index)];
        assert_eq!(entry.name.as_str(), original.name);
        assert_eq!(entry.key.id, original.id);
        assert_eq!(entry.key.kind, original.kind);
    }
    assert_eq!(
        reader
            .send(&mut device, Operation::Catalog { index: count }, 0)
            .unwrap()
            .result,
        Ok(Detail::Program(None))
    );
    assert_eq!(
        reader
            .send(&mut device, Operation::Catalog { index: u16::MAX }, 0)
            .unwrap()
            .result,
        Err(Failure::Runtime(Code::Selection))
    );
    let steps = u16::try_from(device.steps().len()).unwrap();
    for index in 0..steps {
        let reply = reader
            .send(&mut device, Operation::Step { index }, 0)
            .unwrap();
        assert_eq!(reply.step_count, steps);
        let Detail::Step(Some(entry)) = reply.result.unwrap() else {
            panic!("missing step")
        };
        let original = &device.steps()[usize::from(index)];
        assert_eq!(entry.id, original.id);
        assert_eq!(entry.name.as_str(), original.name);
        assert_eq!(entry.number.as_str(), original.number);
    }
    assert_eq!(
        reader
            .send(&mut device, Operation::Step { index: steps }, 0)
            .unwrap()
            .result,
        Ok(Detail::Step(None))
    );
    assert_eq!(
        reader
            .send(&mut device, Operation::Step { index: u16::MAX }, 0)
            .unwrap()
            .result,
        Err(Failure::Runtime(Code::Step))
    );
}

#[test]
fn stale_pages_are_refused_and_historical_owned_text_survives_catalog_release() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut controller = Peer::new(&device, 9, all(), 0);
    controller.prepare(&mut device);
    let mut reader = Peer::new(&device, 10, Permissions::only(Scope::Observe), 0);
    let old = reader
        .send(&mut device, Operation::Step { index: 0 }, 0)
        .unwrap();
    let Detail::Step(Some(step)) = old.result.unwrap() else {
        panic!("missing step")
    };
    let name = step.name.as_str().to_owned();
    controller.apply(&mut device, Action::BeginMaintenance, 0);
    device
        .confirm_quiescent(device.quiescence_request().unwrap(), 0)
        .unwrap();
    assert!(device.catalog().is_empty());
    assert!(device.steps().is_empty());
    assert_eq!(step.name.as_str(), name);
    assert_eq!(reader.request(&mut device, old.request, 0).unwrap(), old);
    let mut stale = old.request;
    stale.id += 1;
    assert_eq!(
        reader.request(&mut device, stale, 0).unwrap().result,
        Err(Failure::Runtime(Code::Revision))
    );
    stale.id += 1;
    stale.operation = Operation::Status;
    let fresh = reader.request(&mut device, stale, 0).unwrap();
    assert_eq!(fresh.state.mode, Mode::Maintenance);
    assert_eq!(fresh.step_count, 0);
    assert_eq!(fresh.program_count, 0);
}
