#![cfg(feature = "application")]
use stagemaster_device_auth::application::{Configuration, Role};
use stagemaster_device_session::SecretKey;
fn bytes(role: u8) -> [u8; 160] {
    let mut bytes = [0; 160];
    bytes[..8].copy_from_slice(b"SMDV\x01\0\xa0\0");
    bytes[5] = role;
    bytes[8..24].fill(1);
    bytes[24..40].fill(2);
    bytes[40..48].copy_from_slice(&3_u64.to_le_bytes());
    bytes[48..52].copy_from_slice(&600_000_u32.to_le_bytes());
    bytes[56..88].fill(4);
    bytes[88..120].copy_from_slice(&SecretKey::import([5; 32]).unwrap().public());
    bytes[120..152].copy_from_slice(&SecretKey::import([4; 32]).unwrap().public());
    bytes
}
#[test]
fn trusted_configuration_binds_roles_and_known_public_identities() {
    let data = bytes(1);
    let config = Configuration::import(&data, Role::Device).unwrap();
    assert_eq!(config.device(), [1; 16]);
    assert_eq!(config.principal(), [2; 16]);
    assert_eq!(config.revision(), 3);
    assert_eq!(config.duration_ms(), 600_000);
    config.permit().unwrap();
    assert!(Configuration::import(&data, Role::Controller).is_err());
    let controller = Configuration::import(&bytes(2), Role::Controller).unwrap();
    assert!(controller.permit().is_err());
    assert_eq!(controller.key().public(), config.key().public());
}
#[test]
fn malformed_secrets_public_keys_fields_and_roles_fail_closed() {
    let data = bytes(1);
    for length in 0..160 {
        assert!(Configuration::import(&data[..length], Role::Device).is_err());
    }
    for index in (0..8).chain(52..56).chain(120..160) {
        let mut bad = data;
        bad[index] ^= 0x80;
        assert!(
            Configuration::import(&bad, Role::Device).is_err(),
            "field {index}"
        );
    }
    for span in [8..24, 24..40, 40..48, 48..52, 56..88, 88..120] {
        let mut bad = data;
        bad[span].fill(0);
        assert!(Configuration::import(&bad, Role::Device).is_err());
    }
    let mut same = data;
    same[88..120].copy_from_slice(&data[120..152]);
    assert!(Configuration::import(&same, Role::Device).is_err());
    let mut large = data;
    large[48..52].copy_from_slice(&600_001_u32.to_le_bytes());
    assert!(Configuration::import(&large, Role::Device).is_err());
}
