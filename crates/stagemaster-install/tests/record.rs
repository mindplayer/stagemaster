use sha2::{Digest, Sha256};
use stagemaster_install::{Code, Commit, Identity, Installer, Progress, RECORD_BYTES, Slot};

#[test]
fn protocol_state_is_bounded_independently_of_package_size() {
    assert!(core::mem::size_of::<Installer<()>>() <= 384);
    assert!(core::mem::size_of::<Progress>() <= 160);
    assert!(core::mem::size_of::<Commit>() <= 64);
}
#[test]
fn fixed_record_rejects_every_damaged_byte_and_valid_hash_invalid_fields() {
    let commit = Commit {
        slot: Slot::A,
        generation: 1,
        identity: Identity {
            bytes: 1024,
            digest: [7; 32],
        },
    };
    let bytes = commit.encode().unwrap();
    assert_eq!(Commit::decode(Slot::A, &bytes).unwrap(), commit);
    assert_eq!(Commit::decode(Slot::B, &bytes), Err(Code::Metadata));
    for offset in 0..RECORD_BYTES {
        let mut bad = bytes;
        bad[offset] ^= 1;
        assert!(Commit::decode(Slot::A, &bad).is_err());
    }
    for (offset, value) in [
        (8, 2),
        (9, 1),
        (10, 2),
        (11, 1),
        (28, 1),
        (16, 0),
        (26, 255),
    ] {
        let mut bad = bytes;
        bad[offset] = value;
        let hash = Sha256::digest(&bad[..64]);
        bad[64..].copy_from_slice(&hash);
        assert!(Commit::decode(Slot::A, &bad).is_err(), "{offset}");
    }
    assert!(
        Commit {
            generation: 0,
            ..commit
        }
        .encode()
        .is_err()
    );
    assert!(
        Commit {
            identity: Identity {
                bytes: usize::MAX,
                ..commit.identity
            },
            ..commit
        }
        .encode()
        .is_err()
    );
}
