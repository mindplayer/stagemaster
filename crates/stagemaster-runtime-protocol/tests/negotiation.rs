mod support;
use stagemaster_runtime_protocol::{Access, Error, Offer, Ready};
use support::{decode_all_truncations, ready};

#[test]
fn version_offer_is_stable_and_does_not_silently_downgrade_or_expand_buffers() {
    let offer = Offer::current();
    let expected = b"SMRT\x01\0\0\0\x83\x01\x01\x19\x05\0";
    assert_eq!(offer.encode().unwrap().bytes(), expected);
    assert_eq!(Offer::decode(expected).unwrap(), offer);
    decode_all_truncations(expected, Offer::decode);
    assert_eq!(offer.select(), Ok(1));
    for (min_version, max_version, message_bytes, error) in [
        (2, 3, 1280, Error::Version),
        (0, 1, 1280, Error::Version),
        (3, 1, 1280, Error::Version),
        (1, 1, 1279, Error::Bounds),
        (1, 1, 1281, Error::Bounds),
    ] {
        assert_eq!(
            Offer {
                min_version,
                max_version,
                message_bytes
            }
            .select(),
            Err(error)
        );
    }
    assert_eq!(
        Offer {
            min_version: 1,
            max_version: u16::MAX,
            message_bytes: 1280
        }
        .select(),
        Ok(1)
    );
}

#[test]
fn readiness_matches_exact_secure_context_rights_and_remaining_development_permission() {
    let expected = ready();
    let wire = expected.encode().unwrap();
    assert_eq!(wire.bytes().len(), 90);
    assert_eq!(&wire.bytes()[..9], b"SMRT\x01\x01\0\0\x8b");
    let actual = Ready::decode(wire.bytes()).unwrap();
    actual.correlate(Offer::current(), &expected).unwrap();
    decode_all_truncations(wire.bytes(), Ready::decode);
    for field in 0..9 {
        let mut foreign = actual;
        match field {
            0 => foreign.peer.device[0] ^= 1,
            1 => foreign.peer.boot[0] ^= 1,
            2 => foreign.peer.connection += 1,
            3 => foreign.peer.session[0] ^= 1,
            4 => foreign.peer.principal[0] ^= 1,
            5 => foreign.peer.permission_revision += 1,
            6 => foreign.access.control = false,
            7 => foreign.access.installation = true,
            8 => foreign.remaining_ms += 1,
            _ => unreachable!(),
        }
        assert_eq!(
            foreign.correlate(Offer::current(), &expected),
            Err(Error::Correlation)
        );
    }
    let mut shorter = actual;
    shorter.remaining_ms -= 1;
    shorter.correlate(Offer::current(), &expected).unwrap();
    let mut bytes = wire.bytes().to_vec();
    bytes[83] = 8;
    assert!(Ready::decode(&bytes).is_err());
}

#[test]
fn installation_only_zero_identity_and_unbounded_readiness_are_refused() {
    let mut r = ready();
    r.access = Access {
        observe: false,
        control: false,
        installation: true,
    };
    assert!(r.encode().is_err());
    r = ready();
    r.remaining_ms = 600_001;
    assert!(r.encode().is_err());
    r.remaining_ms = 0;
    assert!(r.encode().is_err());
    r = ready();
    r.peer.session = [0; 16];
    assert!(r.encode().is_err());
    r = ready();
    r.peer.connection = 0;
    assert!(r.encode().is_err());
    r = ready();
    r.peer.permission_revision = 0;
    assert!(r.encode().is_err());
    r = ready();
    r.version = 2;
    assert!(r.encode().is_err());
    r = ready();
    r.message_bytes = 1279;
    assert!(r.encode().is_err());
}
