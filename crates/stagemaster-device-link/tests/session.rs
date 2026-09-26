use stagemaster_device_link::{Code, EXPIRY_MS, HELLO, PING, Packet, Session};

fn request(kind: u8, session: u64, sequence: u32) -> [u8; 20] {
    Packet {
        kind,
        code: 0,
        session,
        sequence,
        value: 0,
    }
    .encode()
}

#[test]
fn wire_layout_has_independent_little_endian_reference() {
    let bytes = [1, 2, 0, 0, 8, 7, 6, 5, 4, 3, 2, 1, 4, 3, 2, 1, 0, 0, 0, 0];
    let packet = Packet::decode(&bytes).unwrap();
    assert_eq!(packet.session, 0x0102_0304_0506_0708);
    assert_eq!(packet.sequence, 0x0102_0304);
    assert_eq!(packet.encode(), bytes);
}

#[test]
fn handshake_and_fresh_heartbeats_renew_session() {
    let mut session = Session::new(91, 100).unwrap();
    let hello = session.receive(&request(HELLO, 0, 0), 100);
    assert_eq!(
        (hello.kind, hello.code, hello.session, hello.value),
        (129, 0, 91, 2000)
    );
    for sequence in 1..10 {
        let reply = session.receive(
            &request(PING, 91, sequence),
            100 + u64::from(sequence) * 2000,
        );
        assert_eq!((reply.sequence, reply.code), (sequence, 0));
    }
    assert!(!session.poll(24_099).unwrap());
    assert!(session.poll(24_100).unwrap());
}

#[test]
fn duplicate_and_out_of_order_messages_do_not_renew() {
    let mut session = Session::new(91, 0).unwrap();
    assert_eq!(session.receive(&request(HELLO, 0, 0), 0).code, 0);
    assert_eq!(session.receive(&request(PING, 91, 2), 1000).code, 0);
    for (now, sequence) in [(2000, 2), (6999, 1)] {
        assert_eq!(
            session.receive(&request(PING, 91, sequence), now).code,
            Code::Order as u8
        );
    }
    assert!(session.poll(7000).unwrap());
}

#[test]
fn wrong_connection_and_expired_session_cannot_be_revived() {
    let mut first = Session::new(1, 0).unwrap();
    assert_eq!(first.receive(&request(HELLO, 0, 0), 0).code, 0);
    assert_eq!(
        first.receive(&request(PING, 2, 1), 5999).code,
        Code::Session as u8
    );
    assert_eq!(
        first.receive(&request(PING, 1, 2), EXPIRY_MS).code,
        Code::Expired as u8
    );
    assert_eq!(
        first.receive(&request(HELLO, 0, 0), 7000).code,
        Code::Expired as u8
    );
    let mut second = Session::new(2, 7000).unwrap();
    assert_eq!(second.receive(&request(HELLO, 0, 0), 7000).code, 0);
    assert_eq!(
        second.receive(&request(PING, 1, 3), 7001).code,
        Code::Session as u8
    );
    assert_eq!(second.receive(&request(PING, 2, 1), 7002).code, 0);
}

#[test]
fn malformed_unknown_and_wrong_version_never_refresh() {
    let mut session = Session::new(9, 0).unwrap();
    assert_eq!(
        session.receive(&request(PING, 9, 1), 0).code,
        Code::Session as u8
    );
    for length in 0..20 {
        assert_eq!(
            session.receive(&[0; 20][..length], 0).code,
            Code::Malformed as u8
        );
    }
    let mut bytes = request(HELLO, 0, 0);
    bytes[0] = 2;
    assert_eq!(session.receive(&bytes, 1000).code, Code::Version as u8);
    bytes[0] = 1;
    bytes[3] = 1;
    assert_eq!(session.receive(&bytes, 2000).code, Code::Malformed as u8);
    assert_eq!(
        session.receive(&request(99, 0, 0), 5000).code,
        Code::Unsupported as u8
    );
    assert!(session.poll(6000).unwrap());
}

#[test]
fn backwards_time_does_not_consume_sequence() {
    let mut session = Session::new(9, 100).unwrap();
    assert_eq!(session.receive(&request(HELLO, 0, 0), 100).code, 0);
    assert_eq!(
        session.receive(&request(PING, 9, 1), 99).code,
        Code::Clock as u8
    );
    assert_eq!(session.receive(&request(PING, 9, 1), 101).code, 0);
}

#[test]
fn sequence_wrap_requires_new_connection() {
    assert!(Session::new(0, 0).is_err());
    let mut session = Session::new(9, 0).unwrap();
    assert_eq!(session.receive(&request(HELLO, 0, 0), 0).code, 0);
    assert_eq!(session.receive(&request(PING, 9, u32::MAX), 1).code, 0);
    assert_eq!(
        session.receive(&request(PING, 9, 0), 2).code,
        Code::Order as u8
    );
    assert_eq!(
        session.receive(&request(HELLO, 0, 0), 3).code,
        Code::Session as u8
    );
}
