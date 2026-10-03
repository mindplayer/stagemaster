use stagemaster_device_info::{Description, Error, Firmware, Limits, capability as c};

fn runtime() -> Description {
    Description {
        device: [1; 16],
        boot: [2; 16],
        session: 1,
        model: 99,
        firmware: Firmware::default(),
        authentication: 2,
        capabilities: c::DIAGNOSTICS | c::CATALOG | c::PLAYBACK | c::RUNTIME_APPLICATION,
        limits: Limits {
            package_version: 1,
            package_bytes: 65536,
            programs: 8,
            universes: 1,
            message_bytes: 1280,
            loader_bytes: 65536,
            frame_ms: 25,
            ..Limits::default()
        },
    }
}
#[test]
fn runtime_endpoint_can_exist_without_installation_or_physical_output() {
    let desc = runtime();
    let bytes = desc.encode().unwrap();
    assert_eq!(bytes.len(), 96);
    assert_eq!(
        u32::from_le_bytes(bytes[56..60].try_into().unwrap()),
        1 | 2 | 8 | 64
    );
    assert_eq!(Description::decode(&bytes).unwrap(), desc);
    assert!(!desc.declares(c::INSTALLATION));
    assert!(!desc.declares(c::DMX_OUTPUT));
    let mut old = desc;
    old.capabilities &= !c::RUNTIME_APPLICATION;
    old.limits.message_bytes = 0;
    assert!(old.validate().is_ok());
    assert!(
        !Description::decode(&old.encode().unwrap())
            .unwrap()
            .declares(c::RUNTIME_APPLICATION)
    );
}
#[test]
fn runtime_declaration_requires_implemented_playback_authentication_and_message_budget() {
    let mut desc = runtime();
    desc.authentication = 0;
    assert_eq!(desc.validate(), Err(Error::Capabilities));
    desc = runtime();
    desc.capabilities &= !c::PLAYBACK;
    assert_eq!(desc.validate(), Err(Error::Capabilities));
    desc = runtime();
    desc.limits.message_bytes = 0;
    assert_eq!(desc.validate(), Err(Error::Limits));
    desc = runtime();
    desc.limits.chunk_bytes = 128;
    assert_eq!(desc.validate(), Err(Error::Limits));
    desc = runtime();
    desc.limits.slot_bytes = 65536;
    assert_eq!(desc.validate(), Err(Error::Limits));
}
