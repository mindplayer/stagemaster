use stagemaster_device_link::{
    Code, Packet, Session,
    client::{Client, Diagnostics, Error},
};

#[test]
fn host_and_device_match_and_old_receipts_do_not_renew() {
    let mut host = Client::default();
    assert_eq!(host.session_id(), None);
    let mut device = Session::new(17, 0).unwrap();
    let hello = host.request().unwrap();
    assert_eq!(host.request(), Err(Error::State));
    let reply = device.receive(&hello, 10).encode();
    host.accept(&reply).unwrap();
    assert_eq!(host.session_id(), Some(17));
    assert_eq!(host.accept(&reply), Err(Error::State));
    for i in 1..40 {
        let ping = host.request().unwrap();
        assert_eq!(Packet::decode(&ping).unwrap().sequence, i);
        assert_eq!(host.accept(&reply), Err(Error::Correlation));
        let valid = device.receive(&ping, u64::from(i) * 2000).encode();
        let mut wrong = Packet::decode(&valid).unwrap();
        wrong.session = 19;
        assert_eq!(host.accept(&wrong.encode()), Err(Error::Correlation));
        host.accept(&valid).unwrap();
    }
}

#[test]
fn invalid_and_failed_replies_never_complete_a_request() {
    let mut host = Client::default();
    let request = host.request().unwrap();
    let mut device = Session::new(3, 0).unwrap();
    let valid = device.receive(&request, 0).encode();
    assert_eq!(
        host.accept(&valid[..19]),
        Err(Error::Packet(Code::Malformed))
    );
    let mut invalid = valid;
    invalid[0] = 2;
    assert_eq!(host.accept(&invalid), Err(Error::Packet(Code::Version)));
    invalid = valid;
    invalid[2] = 5;
    assert_eq!(host.accept(&invalid), Err(Error::Rejected(5)));
    invalid = valid;
    invalid[16] ^= 1;
    assert_eq!(host.accept(&invalid), Err(Error::Correlation));
    assert_eq!(host.request(), Err(Error::State));
    host.accept(&valid).unwrap();
    let ping = host.request().unwrap();
    let expired = device.receive(&ping, 6000).encode();
    assert_eq!(host.accept(&expired), Err(Error::Rejected(5)));
    assert_eq!(host.request(), Err(Error::State));
}

#[test]
fn diagnostics_validate_version_flags_and_heap_without_mistaking_wrapping_counters() {
    let mut bytes = [0; 20];
    bytes[0] = 1;
    bytes[1] = 3;
    bytes[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[12..16].copy_from_slice(&40960_u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&90112_u32.to_le_bytes());
    let value = Diagnostics::decode(&bytes).unwrap();
    assert!(value.self_test && value.output_disabled);
    assert_eq!(value.uptime_ms, u32::MAX);
    for index in [0, 1, 2, 3, 12, 16] {
        let mut bad = bytes;
        bad[index] ^= 0x80;
        assert_eq!(Diagnostics::decode(&bad), Err(Error::Diagnostics));
    }
    bytes[1] = 0;
    let failed = Diagnostics::decode(&bytes).unwrap();
    assert!(!failed.self_test && !failed.output_disabled);
}
