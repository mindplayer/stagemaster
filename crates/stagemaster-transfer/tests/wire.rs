use stagemaster_install::{Commit, Identity, Phase, Progress, Slot, Transaction};
use stagemaster_transfer::{
    Action, Assembler, Command, Error, MAX_FRAME_BYTES, RemoteError, Request, Response, Service,
    State, Upload,
};

fn transaction() -> Transaction {
    Transaction {
        boot: [2; 16],
        counter: 1,
    }
}
fn identity() -> Identity {
    Identity {
        bytes: 2048,
        digest: [3; 32],
    }
}
fn state() -> State {
    State {
        boot: [2; 16],
        head: None,
        progress: None,
        owned: false,
        max_chunk: 1024,
        max_package: 2 * 1024 * 1024,
    }
}
fn status() -> Request<'static> {
    Request {
        link: [1; 16],
        id: 1,
        action: Action::Status,
    }
}

#[test]
fn independent_smp_status_golden_and_all_commands_round_trip_strictly() {
    // Independently specified SMP v2 READ, 36-byte CBOR map, group 0x5354, sequence 1, command 0.
    let mut golden = vec![0x08, 0, 0, 36, 0x53, 0x54, 1, 0, 0xa4, 0x61, b'v', 1, 0x64];
    golden.extend_from_slice(b"link");
    golden.push(0x50);
    golden.extend_from_slice(&[1; 16]);
    golden.extend_from_slice(&[0x62, b'i', b'd', 1, 0x64]);
    golden.extend_from_slice(b"body");
    golden.push(0x80);
    assert_eq!(status().encode().unwrap().bytes(), golden);
    assert_eq!(Request::decode(&golden).unwrap(), status());
    let block = [0xab; 1024];
    for id in [1, 255, 256, 257, u64::MAX] {
        for action in [
            Action::Status,
            Action::Begin {
                transaction: transaction(),
                identity: identity(),
            },
            Action::Write {
                transaction: transaction(),
                offset: 1024,
                bytes: &block,
            },
            Action::Verify(transaction()),
            Action::Commit(transaction()),
            Action::Cancel(transaction()),
            Action::Reconcile(transaction()),
        ] {
            let request = Request {
                id,
                action,
                ..status()
            };
            let encoded = request.encode().unwrap();
            assert!(encoded.bytes().len() <= MAX_FRAME_BYTES);
            assert_eq!(Request::decode(encoded.bytes()).unwrap(), request);
            for length in 0..encoded.bytes().len() {
                assert!(Request::decode(&encoded.bytes()[..length]).is_err());
            }
            let mut extra = encoded.bytes().to_vec();
            extra.push(0);
            assert!(Request::decode(&extra).is_err());
            assert!(Response::decode(encoded.bytes()).is_err());
        }
    }
}

#[test]
fn response_errors_phases_and_semantic_consistency_are_checked() {
    let commit = Commit {
        slot: Slot::A,
        generation: 1,
        identity: identity(),
    };
    for phase in [
        Phase::Receiving,
        Phase::Verified,
        Phase::Committed,
        Phase::Cancelled,
        Phase::Failed,
        Phase::Uncertain,
    ] {
        for result in [
            Ok(()),
            Err(RemoteError::Storage),
            Err(RemoteError::Uncertain),
            Err(RemoteError::Ownership),
        ] {
            let progress = Progress {
                transaction: transaction(),
                identity: identity(),
                received: 2048,
                phase,
                commit,
            };
            let state = State {
                progress: Some(progress),
                head: (phase == Phase::Committed).then_some(commit),
                owned: true,
                ..state()
            };
            let response = Response {
                link: [1; 16],
                id: 257,
                command: Command::Commit,
                result,
                state,
            };
            let bytes = response.encode().unwrap();
            assert_eq!(Response::decode(bytes.bytes()).unwrap(), response);
            assert!(Request::decode(bytes.bytes()).is_err());
            for length in 0..bytes.bytes().len() {
                assert!(Response::decode(&bytes.bytes()[..length]).is_err());
            }
        }
    }
    let invalid = State {
        progress: Some(Progress {
            transaction: transaction(),
            identity: identity(),
            received: 2048,
            phase: Phase::Committed,
            commit,
        }),
        owned: true,
        ..state()
    };
    let response = Response {
        link: [1; 16],
        id: 1,
        command: Command::Status,
        result: Ok(()),
        state: invalid,
    };
    assert!(response.encode().is_err()); // A planned commit cannot impersonate a durable head.
    assert!(
        Response {
            state: State {
                owned: true,
                ..state()
            },
            ..response
        }
        .encode()
        .is_err()
    );
    assert!(
        Response {
            state: State {
                max_chunk: 1025,
                ..state()
            },
            ..response
        }
        .encode()
        .is_err()
    );
}

#[test]
fn malformed_versions_headers_duplicate_fields_and_oversized_lengths_are_rejected() {
    let original = status().encode().unwrap();
    for (index, value) in [
        (0, 0),
        (0, 0x48),
        (1, 1),
        (2, 255),
        (4, 1),
        (6, 2),
        (7, 8),
        (11, 2),
    ] {
        let mut bad = original.bytes().to_vec();
        bad[index] = value;
        assert!(Request::decode(&bad).is_err(), "index {index}");
    }
    let mut duplicate = original.bytes().to_vec();
    let at = duplicate.windows(4).position(|v| v == b"body").unwrap();
    duplicate[at..at + 4].copy_from_slice(b"link");
    assert!(Request::decode(&duplicate).is_err());
    for bytes in [&[][..], &[1; 1025][..]] {
        assert!(
            Request {
                action: Action::Write {
                    transaction: transaction(),
                    offset: 0,
                    bytes
                },
                ..status()
            }
            .encode()
            .is_err()
        );
    }
    assert!(
        Request {
            action: Action::Write {
                transaction: transaction(),
                offset: usize::MAX,
                bytes: &[1]
            },
            ..status()
        }
        .encode()
        .is_err()
    );
    assert!(Request { id: 0, ..status() }.encode().is_err());
    assert!(
        Request {
            link: [0; 16],
            ..status()
        }
        .encode()
        .is_err()
    );
}

#[test]
fn arbitrary_fragment_boundaries_poisoning_and_fixed_memory_limits() {
    let data = [7; 1024];
    let frame = Request {
        action: Action::Write {
            transaction: transaction(),
            offset: 0,
            bytes: &data,
        },
        ..status()
    }
    .encode()
    .unwrap();
    for mtu in [1, 7, 8, 20, 185, 244, 512, 1280] {
        let mut assembler = Assembler::new();
        for (i, part) in frame.bytes().chunks(mtu).enumerate() {
            assert_eq!(
                assembler.push(part).unwrap(),
                (i + 1) * mtu >= frame.bytes().len()
            );
        }
        assert_eq!(assembler.take().unwrap(), frame);
        assert!(assembler.take().is_none());
        assert!(assembler.push(original_status().bytes()).unwrap());
    }
    let mut assembler = Assembler::new();
    assert_eq!(assembler.push(&[]), Err(Error::State));
    assert_eq!(assembler.push(frame.bytes()), Err(Error::Connection));
    assert!(assembler.take().is_none());
    let mut double = frame.bytes().to_vec();
    double.push(0);
    assert_eq!(Assembler::new().push(&double), Err(Error::Bounds));
    assert!(core::mem::size_of::<Assembler>() <= 1400);
    assert!(core::mem::size_of::<Service<()>>() <= 2048);
    assert!(core::mem::size_of::<Upload<&[u8]>>() <= 2048);
}
fn original_status() -> stagemaster_transfer::Frame {
    status().encode().unwrap()
}

#[test]
fn arbitrary_input_does_not_panic_or_allocate_from_declared_lengths() {
    let mut seed = 0x1234_5678_u32;
    for length in 0..=1400 {
        let mut bytes = vec![0; length];
        for b in &mut bytes {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            *b = seed.to_le_bytes()[0];
        }
        let _ = Request::decode(&bytes);
        let _ = Response::decode(&bytes);
        let _ = Assembler::new().push(&bytes);
    }
}
