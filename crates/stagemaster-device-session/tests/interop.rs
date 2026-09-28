mod support;
use noise_protocol::{HandshakeState, U8Array, patterns};
use noise_rust_crypto::{ChaCha20Poly1305, Sha256, X25519, sensitive::Sensitive};
use stagemaster_device_session::*;
use support::*;

type Independent = HandshakeState<X25519, ChaCha20Poly1305, Sha256>;
const DOMAIN: &[u8; 24] = b"StageMaster/Secure/v1\0\0\0";

fn prologue() -> [u8; 64] {
    let mut bytes = [0; 64];
    bytes[..24].copy_from_slice(DOMAIN);
    bytes[24..40].copy_from_slice(&context().device);
    bytes[40..56].copy_from_slice(&context().boot);
    bytes[56..64].copy_from_slice(&context().connection.to_le_bytes());
    bytes
}
fn independent(initiator: bool) -> Independent {
    Independent::new(
        patterns::noise_ik(),
        initiator,
        prologue(),
        Some(Sensitive::from_slice(&[if initiator { 3 } else { 4 }; 32])),
        // Fixed ephemeral is only an independent interop fixture.
        Some(Sensitive::from_slice(&[9; 32])),
        initiator.then(|| key(4).public()),
        None,
    )
}

#[test]
fn initiator_interoperates_with_independent_responder_and_rejects_invalid_record_type() {
    let mut client = Handshake::initiate(context(), &key(3), key(4).public(), entropy, 0).unwrap();
    let mut server = independent(false);
    let mut first = [0; 96];
    client.write(&mut first, 0).unwrap();
    server.read_message(&first, &mut []).unwrap();
    let mut second = [0; 48];
    server.write_message(&[], &mut second).unwrap();
    client.read(&second, 0).unwrap();
    let mut client = client.finish(0).unwrap();
    let (mut receive, mut send) = server.get_ciphers();
    let mut wire = [0; CIPHERTEXT_BYTES];
    let n = client.confirmation(&mut wire, 0).unwrap();
    let mut confirmation = [0; 33];
    receive.decrypt(&wire[..n], &mut confirmation).unwrap();
    assert_eq!(confirmation[0], 0xf0);
    assert_eq!(&confirmation[1..], server.get_hash());
    confirmation[0] = 0xf1;
    send.encrypt(&confirmation, &mut wire[..49]);
    client.confirm(&wire[..49], 0).unwrap();
    assert_eq!(
        client.peer(0).unwrap().unwrap().transcript(),
        server.get_hash()
    );
    let payload = [0x5a; MAX_PAYLOAD];
    let n = client.seal(Kind::Message, &payload, &mut wire, 1).unwrap();
    let mut clear = [0; PLAINTEXT_BYTES];
    receive.decrypt(&wire[..n], &mut clear).unwrap();
    assert_eq!(clear[0], 1);
    assert_eq!(clear[1..], payload);
    // Authenticated bytes with an unsupported application type must still be rejected.
    send.encrypt(&[0xff], &mut wire[..17]);
    assert!(client.open(&wire[..17], &mut clear, 1).is_err());
    assert_eq!(clear, [0; PLAINTEXT_BYTES]);
    assert_eq!(client.peer(1), Err(Error::Closed));
}

#[test]
fn responder_interoperates_with_independent_initiator_and_requires_fresh_confirmation() {
    let mut client = independent(true);
    let mut server = Handshake::respond(context(), &key(4), entropy, 0).unwrap();
    let mut first = [0; 96];
    client.write_message(&[], &mut first).unwrap();
    server.read(&first, 0).unwrap();
    let mut second = [0; HANDSHAKE_BYTES];
    let n = server.write(&mut second, 0).unwrap();
    client.read_message(&second[..n], &mut []).unwrap();
    let mut server = server.finish(0).unwrap();
    assert!(server.peer(0).unwrap().is_none());
    let (mut send, mut receive) = client.get_ciphers();
    let mut confirmation = [0; 33];
    confirmation[0] = 0xf0;
    confirmation[1..].copy_from_slice(client.get_hash());
    let mut wire = [0; CIPHERTEXT_BYTES];
    send.encrypt(&confirmation, &mut wire[..49]);
    server.confirm(&wire[..49], 1).unwrap();
    assert!(server.peer(1).unwrap().is_none());
    let n = server.confirmation(&mut wire, 1).unwrap();
    receive.decrypt(&wire[..n], &mut confirmation).unwrap();
    assert_eq!(confirmation[0], 0xf1);
    let proof = server.peer(1).unwrap().unwrap();
    assert_eq!(proof.public_key(), &key(3).public());
    let n = server
        .seal(Kind::Message, b"installed digest", &mut wire, 2)
        .unwrap();
    let mut plain = [0; 17];
    receive.decrypt(&wire[..n], &mut plain).unwrap();
    assert_eq!(&plain[1..], b"installed digest");
}
