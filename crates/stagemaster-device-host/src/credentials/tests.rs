use super::*;
use stagemaster_device_session::SecretKey;
use std::io::Write;

fn configuration() -> [u8; CONFIGURATION_BYTES] {
    let mut data = [0; CONFIGURATION_BYTES];
    data[..8].copy_from_slice(b"SMDV\x01\x02\xa0\0");
    data[8..24].fill(1);
    data[24..40].fill(2);
    data[40..48].copy_from_slice(&1_u64.to_le_bytes());
    data[48..52].copy_from_slice(&600_000_u32.to_le_bytes());
    data[56..88].fill(3);
    data[88..120].copy_from_slice(&SecretKey::import([4; 32]).unwrap().public());
    data[120..152].copy_from_slice(&SecretKey::import([3; 32]).unwrap().public());
    data
}

#[test]
fn file_loader_rejects_wrong_role_malformed_and_unbounded_files() {
    let directory = tempfile::tempdir().unwrap();
    assert!(read_development_configuration(directory.path()).is_err());
    let mut file = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    file.write_all(&configuration()).unwrap();
    let loaded = read_development_configuration(file.path()).unwrap();
    assert_eq!(loaded.device(), [1; 16]);
    assert_eq!(loaded.principal(), [2; 16]);
    for length in [0, 159, 161, 4096] {
        file.as_file().set_len(length).unwrap();
        assert!(read_development_configuration(file.path()).is_err());
    }
    let mut bytes = configuration();
    bytes[5] = 1;
    std::fs::write(file.path(), bytes).unwrap();
    assert!(read_development_configuration(file.path()).is_err());
    bytes = configuration();
    bytes[120] ^= 1;
    std::fs::write(file.path(), bytes).unwrap();
    assert!(read_development_configuration(file.path()).is_err());
    std::fs::write(file.path(), configuration()).unwrap();
    assert!(read_development_configuration(file.path()).is_ok());
}

#[cfg(unix)]
#[test]
fn file_loader_rejects_symlink_and_shared_permissions() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let directory = tempfile::tempdir().unwrap();
    let mut file = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    file.write_all(&configuration()).unwrap();
    let link = directory.path().join("link");
    symlink(file.path(), &link).unwrap();
    assert!(read_development_configuration(&link).is_err());
    for mode in [0o644, 0o640, 0o660, 0o700] {
        file.as_file()
            .set_permissions(std::fs::Permissions::from_mode(mode))
            .unwrap();
        assert!(read_development_configuration(file.path()).is_err());
    }
    file.as_file()
        .set_permissions(std::fs::Permissions::from_mode(0o600))
        .unwrap();
    assert!(read_development_configuration(file.path()).is_ok());
}
