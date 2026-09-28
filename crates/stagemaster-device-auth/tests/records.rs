use stagemaster_device_auth::{Address, Binding, Code, LocalIdentity, Secret, Vault};

fn key(n: u8) -> Secret {
    Secret::new([n; 16]).unwrap()
}
fn binding(n: u8) -> Binding {
    Binding::new(
        [n; 16],
        Address::new(false, [n; 6]).unwrap(),
        key(n + 10),
        Some(key(n + 20)),
    )
    .unwrap()
}
fn vault() -> Vault {
    Vault::new(
        LocalIdentity::new(Address::new(true, [1, 2, 3, 4, 5, 0xc6]).unwrap(), key(30)).unwrap(),
    )
}

#[test]
fn strict_roundtrip_and_zero_spare_entries() {
    let mut v = vault();
    for n in 1..=4 {
        v = v.enroll(binding(n)).unwrap();
    }
    assert_eq!(Vault::decode(&v.encode()[..]).unwrap(), v);
    assert_eq!(v.enroll(binding(5)), Err(Code::Full));
    let reduced = v.revoke([2; 16]).unwrap();
    assert_eq!(Vault::decode(&reduced.encode()[..]).unwrap(), reduced);
    assert_eq!(
        reduced
            .bindings()
            .map(Binding::principal)
            .collect::<Vec<_>>(),
        [[1; 16], [3; 16], [4; 16]]
    );
}
#[test]
fn reject_unknown_format_reserved_bytes_and_truncation() {
    let encoded = vault().enroll(binding(1)).unwrap().encode();
    for len in 0..encoded.len() {
        assert!(Vault::decode(&encoded[..len]).is_err());
    }
    for index in [0, 7, 22, 23, 41, 63, 120, 143, 144, 383] {
        let mut changed = encoded.to_vec();
        changed[index] ^= 1;
        assert!(Vault::decode(&changed).is_err(), "offset {index}");
    }
    let mut extra = encoded.to_vec();
    extra.push(0);
    assert!(Vault::decode(&extra).is_err());
}
#[test]
fn duplicate_peer_or_principal_cannot_replace_another_binding() {
    let first = binding(1);
    let v = vault().enroll(first.clone()).unwrap();
    let reused = Binding::new([2; 16], first.address(), key(99), None).unwrap();
    assert_eq!(v.enroll(reused), Err(Code::Duplicate));
    let irk_collision =
        Binding::new([2; 16], binding(2).address(), key(99), first.irk().cloned()).unwrap();
    assert_eq!(v.enroll(irk_collision), Err(Code::Duplicate));
    let different = Binding::new([1; 16], binding(2).address(), key(99), None).unwrap();
    assert_eq!(v.enroll(different), Err(Code::Conflict));
    let refreshed = Binding::new([1; 16], first.address(), key(99), first.irk().cloned()).unwrap();
    assert_eq!(
        v.enroll(refreshed)
            .unwrap()
            .find([1; 16])
            .unwrap()
            .ltk()
            .bytes(),
        &[99; 16]
    );
    assert_eq!(v.revoke([2; 16]), Err(Code::Missing));
}
#[test]
fn malformed_keys_addresses_and_duplicate_encoded_entries_fail_closed() {
    assert!(Secret::new([0; 16]).is_err());
    for bytes in [
        [0; 6],
        [0xff; 6],
        [0, 0, 0, 0, 0, 0xc0],
        [1, 2, 3, 4, 5, 0x46],
    ] {
        assert!(Address::new(true, bytes).is_err());
    }
    let v = vault().enroll(binding(1)).unwrap();
    let mut bytes = v.encode().to_vec();
    bytes[40] = 2;
    let first = bytes[64..144].to_vec();
    bytes[144..224].copy_from_slice(&first);
    assert_eq!(Vault::decode(&bytes), Err(Code::Duplicate));
    bytes[40] = 1;
    bytes[144..224].fill(0);
    bytes[88..104].fill(0);
    assert!(Vault::decode(&bytes).is_err());
}
#[test]
fn generation_exhaustion_and_secret_debug() {
    let v = vault().enroll(binding(1)).unwrap();
    let debug = format!("{v:?} {:?}", v.encode());
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("[11, 11"));
    assert!(!debug.contains("[21, 21"));
    let mut bytes = v.encode().to_vec();
    bytes[8..16].fill(0xff);
    let exhausted = Vault::decode(&bytes).unwrap();
    assert_eq!(exhausted.revoke([1; 16]), Err(Code::Exhausted));
    bytes[8..16].fill(0);
    assert!(Vault::decode(&bytes).is_err());
}
