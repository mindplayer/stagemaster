use stagemaster_device_info::{
    Description, Error, Firmware, Limits, capability as c, esp32_device_id,
};

// Independently written byte fixture; no encoder constructs this reference.
const FULL: [u8; 96] = [
    0x53, 0x4d, 0x44, 0x43, 1, 0, 96, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
    18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 8, 7, 6, 5, 4, 3, 2, 1, 1, 0, 0, 0,
    2, 0, 3, 0, 31, 0, 0, 0, 1, 0, 1, 0, 0, 0, 32, 0, 64, 0, 1, 0, 0, 5, 0, 4, 0, 0, 32, 0, 0, 0,
    1, 0, 25, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
];

#[test]
fn independent_wire_and_little_endian_limits_are_exact() {
    let info = Description::decode(&FULL).unwrap();
    assert_eq!(
        info.device,
        core::array::from_fn(|i| u8::try_from(i + 1).unwrap())
    );
    assert_eq!(
        info.boot,
        core::array::from_fn(|i| u8::try_from(i + 17).unwrap())
    );
    assert_eq!(info.session, 0x0102_0304_0506_0708);
    assert_eq!(
        info.firmware,
        Firmware {
            major: 0,
            minor: 2,
            patch: 3
        }
    );
    assert_eq!(
        info.limits,
        Limits {
            package_version: 1,
            transfer_version: 1,
            package_bytes: 2 * 1024 * 1024,
            programs: 64,
            universes: 1,
            message_bytes: 1280,
            chunk_bytes: 1024,
            slot_bytes: 2 * 1024 * 1024,
            loader_bytes: 65536,
            frame_ms: 25,
        }
    );
    assert_eq!(info.encode().unwrap(), FULL);
    assert_eq!(info.check_session(1), Err(Error::Session));
    assert_eq!(info.check_session(0), Err(Error::Session));
    info.check_session(0x0102_0304_0506_0708).unwrap();
}

#[test]
fn diagnostic_only_devices_cannot_claim_storage_or_output_limits() {
    let mut info = Description::decode(&FULL).unwrap();
    info.capabilities = c::DIAGNOSTICS;
    info.authentication = 0;
    assert_eq!(info.encode(), Err(Error::Limits));
    info.limits = Limits::default();
    assert_eq!(Description::decode(&info.encode().unwrap()).unwrap(), info);
    assert!(!info.declares(c::INSTALLATION));
    assert!(!info.declares(0));
    info.capabilities |= c::DMX_OUTPUT;
    assert_eq!(info.validate(), Err(Error::Capabilities));
}

#[test]
fn capabilities_are_claims_and_unknown_extensions_do_not_become_known_features() {
    let mut info = Description::decode(&FULL).unwrap();
    info.model = 999;
    info.authentication = 999;
    info.capabilities |= 1 << 30;
    let decoded = Description::decode(&info.encode().unwrap()).unwrap();
    assert_eq!(decoded, info);
    assert_eq!(decoded.unknown_capabilities(), 1 << 30);
    // This crate intentionally exposes no AuthorizedLink, principal or permission conversion.
    info.authentication = 0;
    assert_eq!(info.encode(), Err(Error::Capabilities));
    info = Description::decode(&FULL).unwrap();
    info.capabilities &= !c::CATALOG;
    assert_eq!(info.encode(), Err(Error::Capabilities));
}

#[test]
fn malformed_headers_zero_ids_and_contradictory_limits_are_rejected() {
    for length in 0..96 {
        assert_eq!(Description::decode(&FULL[..length]), Err(Error::Length));
    }
    let mut extended = FULL.to_vec();
    extended.push(0);
    assert_eq!(Description::decode(&extended), Err(Error::Length));
    for offset in [5, 90, 91, 92, 93, 94, 95] {
        let mut bytes = FULL;
        bytes[offset] = 1;
        assert_eq!(Description::decode(&bytes), Err(Error::Reserved));
    }
    for (offset, value, error) in [
        (0, 0, Error::Magic),
        (4, 2, Error::Version),
        (6, 95, Error::Length),
    ] {
        let mut bytes = FULL;
        bytes[offset] = value;
        assert_eq!(Description::decode(&bytes), Err(error));
    }
    for (range, error) in [
        (8..24, Error::Identity),
        (24..40, Error::Identity),
        (40..48, Error::Session),
    ] {
        let mut bytes = FULL;
        bytes[range].fill(0);
        assert_eq!(Description::decode(&bytes), Err(error));
    }
    let mut info = Description::decode(&FULL).unwrap();
    info.limits.slot_bytes -= 1;
    assert_eq!(info.validate(), Err(Error::Limits));
    info = Description::decode(&FULL).unwrap();
    info.limits.message_bytes = 8;
    assert_eq!(info.validate(), Err(Error::Limits));
    info = Description::decode(&FULL).unwrap();
    info.limits.chunk_bytes = info.limits.message_bytes;
    assert_eq!(info.validate(), Err(Error::Limits));
}

#[test]
fn stable_id_uses_factory_identity_without_wireless_or_boot_state() {
    let mac = [0x24, 0x6f, 0x28, 0x12, 0x34, 0x56];
    assert_eq!(
        esp32_device_id(mac).unwrap(),
        *b"SMESP32S3\0\x24\x6f\x28\x12\x34\x56"
    );
    assert_ne!(
        esp32_device_id(mac),
        esp32_device_id([0x24, 0x6f, 0x28, 0x12, 0x34, 0x57])
    );
    for invalid in [[0; 6], [255; 6], [1, 2, 3, 4, 5, 6]] {
        assert_eq!(esp32_device_id(invalid), Err(Error::Identity));
    }
}
