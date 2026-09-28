#![allow(dead_code)]
use core::sync::atomic::{AtomicU32, Ordering};
use stagemaster_device_session::*;

// Deterministic test entropy only; never linked into the firmware or application.
#[allow(clippy::unnecessary_wraps)] // Implements the production fallible entropy signature.
pub fn entropy(out: &mut [u8]) -> Result<(), Error> {
    static NEXT: AtomicU32 = AtomicU32::new(1);
    let value = NEXT.fetch_add(1, Ordering::SeqCst).to_le_bytes();
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = value[index % 4];
    }
    Ok(())
}
pub fn context() -> Context {
    Context {
        device: [1; 16],
        boot: [2; 16],
        connection: 3,
    }
}
pub fn key(value: u8) -> SecretKey {
    SecretKey::import([value; 32]).unwrap()
}
pub fn handshakes() -> (Handshake, Handshake) {
    (
        Handshake::initiate(context(), &key(3), key(4).public(), entropy, 0).unwrap(),
        Handshake::respond(context(), &key(4), entropy, 0).unwrap(),
    )
}
pub fn unconfirmed() -> (Channel, Channel) {
    let (mut client, mut server) = handshakes();
    let mut wire = [0; HANDSHAKE_BYTES];
    let n = client.write(&mut wire, 0).unwrap();
    assert_eq!(n, 96);
    server.read(&wire[..n], 0).unwrap();
    let n = server.write(&mut wire, 0).unwrap();
    assert_eq!(n, 48);
    client.read(&wire[..n], 0).unwrap();
    (client.finish(0).unwrap(), server.finish(0).unwrap())
}
pub fn channels() -> (Channel, Channel) {
    let (mut client, mut server) = unconfirmed();
    assert!(client.peer(0).unwrap().is_none());
    assert!(server.peer(0).unwrap().is_none());
    let mut wire = [0; CIPHERTEXT_BYTES];
    let n = client.confirmation(&mut wire, 0).unwrap();
    server.confirm(&wire[..n], 0).unwrap();
    assert!(server.peer(0).unwrap().is_none());
    let n = server.confirmation(&mut wire, 0).unwrap();
    client.confirm(&wire[..n], 0).unwrap();
    assert_eq!(
        client.peer(0).unwrap().unwrap().public_key(),
        &key(4).public()
    );
    assert_eq!(
        server.peer(0).unwrap().unwrap().public_key(),
        &key(3).public()
    );
    (client, server)
}
