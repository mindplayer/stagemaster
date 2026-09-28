mod support;
use stagemaster_device_session::*;
use support::*;

#[test]
fn mutual_confirmation_then_bounded_bidirectional_records() {
    let (mut client, mut server) = channels();
    let a = client.peer(0).unwrap().unwrap();
    let b = server.peer(0).unwrap().unwrap();
    assert_eq!(a.context(), context());
    assert_eq!(a.session(), b.session());
    assert_ne!(a.session(), [0; 16]);
    let mut wire = [0; CIPHERTEXT_BYTES];
    let mut plain = [0; PLAINTEXT_BYTES];
    for length in [1, 16, 240, 1024, MAX_PAYLOAD] {
        let data = vec![0x42; length];
        let n = client.seal(Kind::Message, &data, &mut wire, 1).unwrap();
        assert_eq!(n, length + 17);
        let record = server.open(&wire[..n], &mut plain, 1).unwrap();
        assert_eq!(record.kind, Kind::Message);
        assert_eq!(record.payload, data);
    }
    let n = server
        .seal(Kind::HeartbeatReply, &[], &mut wire, 2)
        .unwrap();
    assert_eq!(
        client.open(&wire[..n], &mut plain, 2).unwrap().kind,
        Kind::HeartbeatReply
    );
    assert_eq!(client.peer(2).unwrap().unwrap().session(), a.session());
}

#[test]
fn malformed_wrong_order_and_unconfirmed_traffic_fail_closed() {
    let (mut client, _) = handshakes();
    assert_eq!(client.read(&[0; 48], 0), Err(Error::State));
    assert_eq!(
        client.write(&mut [0; HANDSHAKE_BYTES], 0),
        Err(Error::Closed)
    );
    for length in [0, 1, 47, 95, 97, 1000] {
        let (_, mut server) = handshakes();
        assert_eq!(server.read(&vec![0; length], 0), Err(Error::Invalid));
        assert!(server.finish(0).is_err());
    }
    let (mut client, _) = unconfirmed();
    assert_eq!(
        client.seal(Kind::Message, b"install", &mut [0; CIPHERTEXT_BYTES], 0),
        Err(Error::State)
    );
    assert_eq!(client.peer(0), Err(Error::Closed));
    let (_, mut server) = unconfirmed();
    assert!(server.open(&[0; 49], &mut [0; PLAINTEXT_BYTES], 0).is_err());
    assert_eq!(server.peer(0), Err(Error::Closed));
}

#[test]
fn wrong_keys_context_and_low_order_dh_are_rejected() {
    for field in 0..4 {
        let mut ctx = context();
        match field {
            0 => ctx.device[0] ^= 1,
            1 => ctx.boot[0] ^= 1,
            2 => ctx.connection += 1,
            _ => {}
        }
        let device_key = if field == 3 { key(5) } else { key(4) };
        let mut client =
            Handshake::initiate(context(), &key(3), device_key.public(), entropy, 0).unwrap();
        let mut server = Handshake::respond(ctx, &key(4), entropy, 0).unwrap();
        let mut wire = [0; HANDSHAKE_BYTES];
        let n = client.write(&mut wire, 0).unwrap();
        assert_eq!(server.read(&wire[..n], 0), Err(Error::Crypto));
        assert!(server.finish(0).is_err());
    }
    for low_order in [0, 1] {
        let (_, mut server) = handshakes();
        let mut wire = [0; HANDSHAKE_BYTES];
        wire[0] = low_order;
        assert_eq!(server.read(&wire, 0), Err(Error::Crypto));
    }
}

#[test]
fn replay_gap_corruption_and_cross_session_packets_close_and_clear_plaintext() {
    for failure in 0..4 {
        let (mut client, mut server) = channels();
        let mut wire = [0; CIPHERTEXT_BYTES];
        let n = client
            .seal(Kind::Message, b"private program", &mut wire, 0)
            .unwrap();
        let mut plain = [0x55; PLAINTEXT_BYTES];
        match failure {
            0 => {
                server.open(&wire[..n], &mut plain, 0).unwrap();
            }
            1 => {
                client
                    .seal(Kind::Message, b"private program", &mut wire, 0)
                    .unwrap();
            }
            2 => {
                wire[n - 1] ^= 1;
            }
            _ => {
                server = channels().1;
            }
        }
        assert!(server.open(&wire[..n], &mut plain, 0).is_err());
        assert_eq!(plain, [0; PLAINTEXT_BYTES]);
        assert_eq!(server.peer(0), Err(Error::Closed));
    }
}

#[test]
fn handshake_deadline_is_fixed_and_poll_or_send_does_not_extend_lease() {
    let (mut client, mut server) = handshakes();
    let mut handshake = [0; HANDSHAKE_BYTES];
    let n = client.write(&mut handshake, HANDSHAKE_MS - 1).unwrap();
    assert_eq!(
        server.read(&handshake[..n], HANDSHAKE_MS),
        Err(Error::Expired)
    );
    let (mut client, _) = unconfirmed();
    assert_eq!(
        client.confirmation(&mut [0; CIPHERTEXT_BYTES], HANDSHAKE_MS),
        Err(Error::Expired)
    );
    let (mut client, mut server) = channels();
    let mut wire = [0; CIPHERTEXT_BYTES];
    client
        .seal(Kind::Heartbeat, &[], &mut wire, LEASE_MS - 1)
        .unwrap();
    for now in 1..LEASE_MS {
        server.poll(now).unwrap();
    }
    assert_eq!(client.peer(LEASE_MS), Err(Error::Expired));
    assert_eq!(server.peer(LEASE_MS), Err(Error::Expired));
}

#[test]
fn verified_incoming_heartbeat_renews_only_its_receiver() {
    let (mut client, mut server) = channels();
    let mut wire = [0; CIPHERTEXT_BYTES];
    let n = client.seal(Kind::Heartbeat, &[], &mut wire, 5000).unwrap();
    server
        .open(&wire[..n], &mut [0; PLAINTEXT_BYTES], 5000)
        .unwrap();
    assert!(server.peer(10_999).unwrap().is_some());
    assert_eq!(server.peer(11_000), Err(Error::Expired));
    assert_eq!(client.peer(6000), Err(Error::Expired));
}

#[test]
fn entropy_clock_and_payload_failures_do_not_fall_back() {
    fn unavailable(_: &mut [u8]) -> Result<(), Error> {
        Err(Error::Entropy)
    }
    assert!(SecretKey::generate(unavailable).is_err());
    assert!(SecretKey::import([0; 32]).is_err());
    let mut client =
        Handshake::initiate(context(), &key(3), key(4).public(), unavailable, 0).unwrap();
    let mut wire = [0x55; HANDSHAKE_BYTES];
    assert_eq!(client.write(&mut wire, 0), Err(Error::Entropy));
    assert_eq!(wire, [0; HANDSHAKE_BYTES]);
    let (mut client, _) = channels();
    client.poll(100).unwrap();
    assert_eq!(client.peer(99), Err(Error::Clock));
    assert!(Handshake::respond(context(), &key(4), entropy, u64::MAX).is_err());
    for (kind, data) in [
        (Kind::Message, vec![]),
        (Kind::Message, vec![0; MAX_PAYLOAD + 1]),
        (Kind::Heartbeat, vec![1]),
    ] {
        let (mut client, _) = channels();
        let mut wire = [0x55; CIPHERTEXT_BYTES];
        assert_eq!(client.seal(kind, &data, &mut wire, 0), Err(Error::Invalid));
        assert_eq!(wire, [0; CIPHERTEXT_BYTES]);
        assert_eq!(client.peer(0), Err(Error::Closed));
    }
}

#[test]
fn delayed_responder_confirmation_cannot_extend_handshake_or_receive_lease() {
    for (received, sent) in [(9_999, 10_000), (1_000, 7_000)] {
        let (mut client, mut server) = unconfirmed();
        let mut wire = [0; CIPHERTEXT_BYTES];
        let n = client.confirmation(&mut wire, received).unwrap();
        server.confirm(&wire[..n], received).unwrap();
        assert_eq!(server.confirmation(&mut wire, sent), Err(Error::Expired));
        assert_eq!(wire, [0; CIPHERTEXT_BYTES]);
        assert_eq!(server.peer(sent), Err(Error::Closed));
    }
}

#[test]
fn replaying_first_handshake_cannot_reuse_old_confirmation() {
    let (mut client, mut first_server) = handshakes();
    let mut first = [0; HANDSHAKE_BYTES];
    let n = client.write(&mut first, 0).unwrap();
    first_server.read(&first[..n], 0).unwrap();
    let mut response = [0; HANDSHAKE_BYTES];
    let n = first_server.write(&mut response, 0).unwrap();
    client.read(&response[..n], 0).unwrap();
    let mut client = client.finish(0).unwrap();
    let mut confirmation = [0; CIPHERTEXT_BYTES];
    let confirmation_len = client.confirmation(&mut confirmation, 0).unwrap();
    let mut new_server = Handshake::respond(context(), &key(4), entropy, 0).unwrap();
    new_server.read(&first, 0).unwrap();
    new_server.write(&mut response, 0).unwrap();
    let mut new_server = new_server.finish(0).unwrap();
    assert!(new_server.peer(0).unwrap().is_none());
    assert_eq!(
        new_server.confirm(&confirmation[..confirmation_len], 0),
        Err(Error::Crypto)
    );
    assert_eq!(new_server.peer(0), Err(Error::Closed));
}
