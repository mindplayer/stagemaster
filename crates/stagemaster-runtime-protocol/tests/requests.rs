mod support;
use stagemaster_package::Kind;
use stagemaster_runtime::{Action, ProgramKey};
use stagemaster_runtime_protocol::{Operation, Request};
use support::{decode_all_truncations, request};

#[test]
fn status_has_a_fixed_independent_wire_vector() {
    let mut expected = b"SMRT\x01\x02\0\0\x84\x50".to_vec();
    expected.extend_from_slice(&[3; 16]);
    expected.extend_from_slice(&[1, 0, 0x81, 0]);
    assert_eq!(
        request(Operation::Status).encode().unwrap().bytes(),
        expected
    );
    assert_eq!(
        Request::decode(&expected).unwrap(),
        request(Operation::Status)
    );
    decode_all_truncations(&expected, Request::decode);
    let mut indefinite = expected.clone();
    indefinite[8] = 0x9f;
    indefinite.push(0xff);
    assert!(Request::decode(&indefinite).is_err());
    let mut count = expected.clone();
    count[8] = 0x85;
    assert!(Request::decode(&count).is_err());
    let mut unknown = expected;
    *unknown.last_mut().unwrap() = 16;
    assert!(Request::decode(&unknown).is_err());
}

#[test]
fn all_operations_keep_explicit_tags_and_exact_full_width_values() {
    let cases = [
        Operation::Status,
        Operation::Catalog { index: u16::MAX },
        Operation::Step { index: u16::MAX },
        Operation::Acquire {
            duration_ms: u64::MAX,
            takeover: true,
        },
        Operation::Renew {
            duration_ms: u64::MAX,
        },
        Operation::Release,
        Operation::FinishMaintenance,
        Operation::Apply(Action::Select(ProgramKey {
            kind: Kind::Sequence,
            id: [9; 16],
        })),
        Operation::Apply(Action::Load),
        Operation::Apply(Action::Start { step: [8; 16] }),
        Operation::Apply(Action::Pause),
        Operation::Apply(Action::Resume),
        Operation::Apply(Action::Next),
        Operation::Apply(Action::Stop),
        Operation::Apply(Action::BeginMaintenance),
        Operation::Apply(Action::CancelMaintenance),
    ];
    for (tag, operation) in cases.into_iter().enumerate() {
        let mut value = request(operation);
        value.id = u64::MAX;
        value.expected_revision = u64::MAX;
        let encoded = value.encode().unwrap();
        let mut d = minicbor::Decoder::new(&encoded.bytes()[8..]);
        assert_eq!(d.array().unwrap(), Some(4));
        d.bytes().unwrap();
        d.u64().unwrap();
        d.u64().unwrap();
        d.array().unwrap();
        assert_eq!(usize::from(d.u8().unwrap()), tag);
        assert_eq!(Request::decode(encoded.bytes()).unwrap(), value);
        decode_all_truncations(encoded.bytes(), Request::decode);
    }
    let scene = request(Operation::Apply(Action::Select(ProgramKey {
        kind: Kind::Scene,
        id: [1; 16],
    })));
    assert_eq!(
        Request::decode(scene.encode().unwrap().bytes()).unwrap(),
        scene
    );
    let unclaimed = request(Operation::Acquire {
        duration_ms: 0,
        takeover: false,
    });
    assert_eq!(
        Request::decode(unclaimed.encode().unwrap().bytes()).unwrap(),
        unclaimed
    );
    // Duration rejection is a business receipt in the existing runtime, not a decoder grant.
}

#[test]
fn wrong_types_zero_id_and_boolean_impostors_are_rejected() {
    let mut r = request(Operation::Status);
    r.id = 0;
    assert!(r.encode().is_err());
    r.id = 1;
    r.session = [0; 16];
    assert!(r.encode().is_err());
    r.session = [1; 16];
    r.operation = Operation::Apply(Action::Start { step: [0; 16] });
    assert!(r.encode().is_err());
    let r = request(Operation::Acquire {
        duration_ms: 1000,
        takeover: false,
    });
    let mut bytes = r.encode().unwrap().bytes().to_vec();
    *bytes.last_mut().unwrap() = 1; // Integer one is not CBOR true.
    assert!(Request::decode(&bytes).is_err());
    bytes = request(Operation::Status)
        .encode()
        .unwrap()
        .bytes()
        .to_vec();
    bytes[9] = 0x70; // Text instead of binary identity.
    assert!(Request::decode(&bytes).is_err());
    assert!(Request::decode(&vec![0; 1281]).is_err());
    assert!(Request::decode(b"SMAP\x01\x01\x70\0").is_err());
}
