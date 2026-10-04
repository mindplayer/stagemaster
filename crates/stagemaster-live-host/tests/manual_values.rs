mod support;
use stagemaster_live::{Change, Command};
use stagemaster_live_host::{Action, Patch, State};
use stagemaster_runtime::Request;
use stagemaster_runtime_host::Backend;
use std::sync::Arc;
use support::{grant, prepared};

#[test]
fn manual_snapshots_are_pre_level_immutable_and_shared_between_ticks() {
    let (mut backend, keys) = prepared();
    let lease = backend.acquire(grant(1, 10_000), false, 0).unwrap();
    let empty = backend.state();
    assert!(empty.manual_values[0].is_none());
    assert!(
        empty.manual_values[2]
            .as_ref()
            .unwrap()
            .iter()
            .all(Option::is_none)
    );
    let patch = |value| Action::Patch {
        source: keys[2],
        patch: Patch::new(&[Change {
            attribute: 0,
            value,
        }])
        .unwrap(),
    };
    let mut serial = 0;
    let mut send = |backend: &mut stagemaster_live_host::LiveBackend, action, now| {
        serial += 1;
        backend
            .submit(
                Request {
                    lease,
                    serial,
                    expected_revision: backend.state().revision,
                    action,
                },
                now,
            )
            .unwrap()
    };
    let receipt = send(&mut backend, patch(Some(24_576)), 1);
    receipt.result.unwrap();
    let stored = receipt.state.manual_values[2].as_ref().unwrap();
    assert_eq!(stored[0], Some(24_576));
    assert_eq!(empty.manual_values[2].as_ref().unwrap()[0], None);
    for now in 2..102 {
        backend.tick(now).unwrap();
        let state = backend.state();
        assert!(Arc::ptr_eq(
            stored,
            state.manual_values[2].as_ref().unwrap()
        ));
    }
    for (action, now) in [
        (
            Action::Level {
                source: keys[2],
                level: 0,
            },
            102,
        ),
        (patch(Some(24_576)), 103),
    ] {
        let state = send(&mut backend, action, now).state;
        assert!(Arc::ptr_eq(
            stored,
            state.manual_values[2].as_ref().unwrap()
        ));
        assert_eq!(state.sources[2].unwrap().manual_held.unwrap()[0] & 1, 1);
    }
    let zero = send(&mut backend, patch(Some(0)), 104).state;
    assert_eq!(zero.manual_values[2].as_ref().unwrap()[0], Some(0));
    assert!(!Arc::ptr_eq(
        stored,
        zero.manual_values[2].as_ref().unwrap()
    ));
    let released = send(&mut backend, patch(None), 105).state;
    assert_eq!(released.manual_values[2].as_ref().unwrap()[0], None);
    assert_eq!(released.sources[2].unwrap().manual_held.unwrap()[0] & 1, 0);
    send(&mut backend, patch(Some(65535)), 106).result.unwrap();
    let stopped = send(
        &mut backend,
        Action::Control {
            source: keys[2],
            command: Command::Stop,
        },
        107,
    )
    .state;
    assert!(
        stopped.manual_values[2]
            .as_ref()
            .unwrap()
            .iter()
            .all(Option::is_none)
    );
    assert_eq!(stored[0], Some(24_576));
    assert_eq!(zero.manual_values[2].as_ref().unwrap()[0], Some(0));
    // Prevent accidentally embedding 64 complete attribute buffers into every copied state.
    assert!(std::mem::size_of::<State>() < 32 * 1024);
}

#[test]
fn rejected_patch_cannot_publish_partial_values() {
    let (mut backend, keys) = prepared();
    let lease = backend.acquire(grant(1, 1000), false, 0).unwrap();
    let before = backend.state();
    let receipt = backend
        .submit(
            Request {
                lease,
                serial: 1,
                expected_revision: before.revision,
                action: Action::Patch {
                    source: keys[2],
                    patch: Patch::new(&[
                        Change {
                            attribute: 0,
                            value: Some(123),
                        },
                        Change {
                            attribute: 511,
                            value: Some(321),
                        },
                    ])
                    .unwrap(),
                },
            },
            1,
        )
        .unwrap();
    assert!(receipt.result.is_err());
    assert!(Arc::ptr_eq(
        before.manual_values[2].as_ref().unwrap(),
        receipt.state.manual_values[2].as_ref().unwrap()
    ));
}
