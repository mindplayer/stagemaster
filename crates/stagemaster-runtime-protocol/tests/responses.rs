mod support;
use stagemaster_install::{Commit, Identity, Slot};
use stagemaster_package::Kind;
use stagemaster_runtime::{Code, Denial, Instance, Lease, Mode, Origin, Owner, ProgramKey, Status};
use stagemaster_runtime_protocol::{Body, Failure, Operation, Program, Response, Step, Text};
use support::{decode_all_truncations, response};

#[test]
fn maximal_step_labels_and_counters_fit_one_existing_secure_message() {
    let name = "灯".repeat(170) + "AB";
    let number = "9".repeat(512);
    let mut r = response();
    r.request.operation = Operation::Step {
        index: u16::MAX - 1,
    };
    r.request.id = u64::MAX;
    r.request.expected_revision = u64::MAX;
    r.observed.revision = u64::MAX;
    r.observed.observed_ms = u64::MAX;
    r.observed.program_count = u16::MAX;
    r.observed.step_count = u16::MAX;
    r.body = Body::Step(Some(Step {
        id: [9; 16],
        name: Text::new(&name).unwrap(),
        number: Text::new(&number).unwrap(),
    }));
    let wire = r.encode().unwrap();
    // Independent v1 size calculation, not merely the encoder's own upper-bound check.
    assert_eq!(wire.bytes().len(), 8 + 1 + 41 + 42 + 2 + 1048);
    assert!(wire.bytes().len() <= 1280);
    assert_eq!(Response::decode(wire.bytes()).unwrap(), r);
    decode_all_truncations(wire.bytes(), Response::decode);
    assert!(Text::new(&(name + "X")).is_err());

    r.request.operation = Operation::Catalog {
        index: u16::MAX - 1,
    };
    r.body = Body::Program(Some(Program {
        key: ProgramKey {
            kind: Kind::Scene,
            id: [8; 16],
        },
        name: Text::new(&number).unwrap(),
        loader_bytes: u32::MAX,
    }));
    let bytes = r.encode().unwrap();
    assert_eq!(Response::decode(bytes.bytes()).unwrap(), r);
    decode_all_truncations(bytes.bytes(), Response::decode);
}

#[test]
fn state_preserves_all_business_identities_owner_status_and_installation() {
    let mut r = response();
    let Body::State { mut state, .. } = r.body else {
        unreachable!()
    };
    state.revision = u64::MAX;
    state.observed_ms = u64::MAX - 1;
    state.bound_package = Some(Commit {
        slot: Slot::B,
        generation: u64::MAX,
        identity: Identity {
            bytes: 4096,
            digest: [4; 32],
        },
    });
    state.selected = Some(ProgramKey {
        kind: Kind::Scene,
        id: [5; 16],
    });
    state.loaded = Some(ProgramKey {
        kind: Kind::Sequence,
        id: [6; 16],
    });
    state.instance = Some(Instance {
        boot: state.boot,
        number: u64::MAX,
    });
    state.step = Some([7; 16]);
    state.elapsed_ms = u64::MAX;
    state.owner = Some(Owner {
        lease: Lease {
            boot: state.boot,
            epoch: u64::MAX,
        },
        principal: [8; 16],
        origin: Origin::Remote,
        expires_ms: u64::MAX,
        serial: u64::MAX,
    });
    r.observed.revision = state.revision;
    r.observed.observed_ms = state.observed_ms;
    for mode in [Mode::Operation, Mode::Quiescing, Mode::Maintenance] {
        for status in [
            Status::Idle,
            Status::Running,
            Status::Paused,
            Status::Finished,
        ] {
            state.mode = mode;
            state.status = Some(status);
            r.body = Body::State {
                state,
                result: Ok(()),
            };
            let bytes = r.encode().unwrap();
            assert_eq!(Response::decode(bytes.bytes()).unwrap(), r);
            decode_all_truncations(bytes.bytes(), Response::decode);
        }
    }
    state.owner.as_mut().unwrap().origin = Origin::Panel;
    state.bound_package.as_mut().unwrap().slot = Slot::A;
    r.body = Body::State {
        state,
        result: Ok(()),
    };
    assert_eq!(Response::decode(r.encode().unwrap().bytes()).unwrap(), r);
}

#[test]
fn every_business_failure_survives_without_becoming_success() {
    let codes = [
        Code::Identity,
        Code::Clock,
        Code::Busy,
        Code::Lease,
        Code::Sequence,
        Code::Revision,
        Code::Exhausted,
        Code::Mode,
        Code::Empty,
        Code::Selection,
        Code::NotLoaded,
        Code::Step,
        Code::State,
        Code::Budget,
        Code::Read,
        Code::Integrity,
        Code::Package,
        Code::Allocation,
        Code::Playback,
        Code::Permission(Denial::Missing),
        Code::Permission(Denial::Expired),
        Code::Permission(Denial::UncertainTime),
        Code::Permission(Denial::Restricted),
    ];
    for (tag, failure) in codes
        .into_iter()
        .map(Failure::Runtime)
        .enumerate()
        .chain([(100, Failure::Storage), (101, Failure::Bounds)])
    {
        let mut r = response();
        let Body::State { state, .. } = r.body else {
            unreachable!()
        };
        r.body = Body::State {
            state,
            result: Err(failure),
        };
        let bytes = r.encode().unwrap();
        assert_eq!(usize::from(*bytes.bytes().last().unwrap()), tag);
        assert_eq!(Response::decode(bytes.bytes()).unwrap(), r);
        decode_all_truncations(bytes.bytes(), Response::decode);
        let mut invalid = bytes.bytes().to_vec();
        *invalid.last_mut().unwrap() = 23;
        assert!(Response::decode(&invalid).is_err());
    }
}

#[test]
fn pages_cannot_invent_end_markers_or_mismatch_expected_content_revision() {
    let mut r = response();
    r.request.operation = Operation::Catalog { index: 0 };
    r.body = Body::Program(None);
    assert_eq!(Response::decode(r.encode().unwrap().bytes()).unwrap(), r);
    r.observed.program_count = 1;
    assert!(r.encode().is_err());
    r.body = Body::Step(None);
    assert!(r.encode().is_err());
    r.body = Body::Program(None);
    r.request.operation = Operation::Catalog { index: 1 };
    assert!(r.encode().is_ok());
    r.observed.revision += 1;
    assert!(r.encode().is_err());
    r = response();
    r.request.operation = Operation::Step { index: 0 };
    r.body = Body::Step(None);
    assert_eq!(Response::decode(r.encode().unwrap().bytes()).unwrap(), r);
}

#[test]
fn stale_boot_request_or_changed_intent_is_not_an_acceptable_reply() {
    let r = response();
    r.correlate(r.request, r.observed.boot).unwrap();
    assert!(r.correlate(r.request, [9; 16]).is_err());
    for field in 0..4 {
        let mut request = r.request;
        match field {
            0 => request.id += 1,
            1 => request.session = [9; 16],
            2 => request.expected_revision += 1,
            _ => request.operation = Operation::Release,
        }
        assert!(r.correlate(request, r.observed.boot).is_err());
    }
    let mut inconsistent = r;
    inconsistent.observed.observed_ms += 1;
    assert!(inconsistent.encode().is_err());
    let Body::State { mut state, .. } = r.body else {
        unreachable!()
    };
    state.owner = Some(Owner {
        lease: Lease {
            boot: [9; 16],
            epoch: 1,
        },
        principal: [8; 16],
        origin: Origin::Remote,
        expires_ms: 200,
        serial: 0,
    });
    inconsistent = r;
    inconsistent.body = Body::State {
        state,
        result: Ok(()),
    };
    assert!(inconsistent.encode().is_err());
}
