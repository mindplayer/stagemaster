use stagemaster_device_link::management::{ApplicationReceipt, Receipt};
fn receipt() -> ApplicationReceipt {
    ApplicationReceipt {
        device: [1; 16],
        boot: [2; 16],
        diagnostic: 3,
        session: [4; 16],
        principal: [5; 16],
        revision: 6,
        remaining_ms: 600_000,
    }
}
#[test]
fn explicit_new_wire_profile_does_not_relabel_lesc() {
    let value = receipt();
    let bytes = value.encode().unwrap();
    assert_eq!(&bytes[..8], b"SMAP\x01\x01\x70\0");
    assert_eq!(
        &bytes[88..104],
        &[1, 0, 0, 0, 2, 0, 1, 0, 0, 5, 0, 0, 112, 23, 0, 0]
    );
    assert_eq!(ApplicationReceipt::decode(&bytes).unwrap(), value);
    assert!(Receipt::decode(&bytes).is_err());
    for index in (0..8).chain(88..104).chain(108..112) {
        let mut bad = bytes;
        bad[index] ^= 0x80;
        assert!(ApplicationReceipt::decode(&bad).is_err(), "byte {index}");
    }
    for length in 0..112 {
        assert!(ApplicationReceipt::decode(&bytes[..length]).is_err());
    }
    let mut extra = bytes.to_vec();
    extra.push(0);
    assert!(ApplicationReceipt::decode(&extra).is_err());
}
#[test]
fn validate_all_identity_and_budget_boundaries() {
    for choice in 0..9 {
        let mut value = receipt();
        match choice {
            0 => value.device = [0; 16],
            1 => value.boot = [0; 16],
            2 => value.diagnostic = 0,
            3 => value.session = [0; 16],
            4 => value.principal = [0; 16],
            5 => value.revision = 0,
            6 => value.remaining_ms = 0,
            7 => value.remaining_ms = 600_001,
            _ => value.remaining_ms = u32::MAX,
        }
        assert!(value.encode().is_err());
    }
}
#[test]
fn foreign_or_stale_metadata_cannot_correlate() {
    for choice in 0..7 {
        let mut value = receipt();
        match choice {
            0 => value.device[0] += 1,
            1 => value.boot[0] += 1,
            2 => value.diagnostic += 1,
            3 => value.session[0] += 1,
            4 => value.principal[0] += 1,
            5 => value.revision += 1,
            _ => value.remaining_ms += 1,
        }
        assert!(value.correlate(&receipt()).is_err());
    }
    let mut shortened = receipt();
    shortened.remaining_ms -= 1;
    shortened.correlate(&receipt()).unwrap();
}
