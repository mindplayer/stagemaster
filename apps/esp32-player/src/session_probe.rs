//! Local resource verification only: no GATT privilege, persisted keys or NOR writes.
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Channel, Context, Error, HANDSHAKE_BYTES, Handshake, Kind, MAX_PAYLOAD,
    PLAINTEXT_BYTES, SecretKey,
};

#[allow(clippy::unnecessary_wraps)] // Platform adapter for fallible entropy API.
fn entropy(out: &mut [u8]) -> Result<(), Error> {
    // main calls this only after BleConnector enables the RF entropy source.
    esp_hal::rng::Rng::new().read(out);
    Ok(())
}
fn now() -> u64 {
    embassy_time::Instant::now().as_millis()
}

#[inline(never)]
pub fn verify(identity: &crate::identity::Identity) {
    let desc = stagemaster_device_info::Description::decode(&identity.describe(1, false)).unwrap();
    let context = Context {
        device: desc.device,
        boot: desc.boot,
        connection: desc.session,
    };
    let before = esp_alloc::HEAP.used();
    let start = esp_hal::time::Instant::now();
    let client_key = SecretKey::generate(entropy).unwrap();
    let device_key = SecretKey::generate(entropy).unwrap();
    let mut client =
        Handshake::initiate(context, &client_key, device_key.public(), entropy, now()).unwrap();
    let mut server = Handshake::respond(context, &device_key, entropy, now()).unwrap();
    let handshake_heap = esp_alloc::HEAP.used();
    let mut wire = [0; HANDSHAKE_BYTES];
    let n = client.write(&mut wire, now()).unwrap();
    server.read(&wire[..n], now()).unwrap();
    let n = server.write(&mut wire, now()).unwrap();
    client.read(&wire[..n], now()).unwrap();
    let mut client = client.finish(now()).unwrap();
    let mut server = server.finish(now()).unwrap();
    let mut cipher = [0; CIPHERTEXT_BYTES];
    let n = client.confirmation(&mut cipher, now()).unwrap();
    server.confirm(&cipher[..n], now()).unwrap();
    let n = server.confirmation(&mut cipher, now()).unwrap();
    client.confirm(&cipher[..n], now()).unwrap();
    assert_eq!(
        client.peer(now()).unwrap().unwrap().public_key(),
        &device_key.public()
    );
    assert_eq!(
        server.peer(now()).unwrap().unwrap().public_key(),
        &client_key.public()
    );
    let handshake_us = start.elapsed().as_micros();
    let channel_heap = esp_alloc::HEAP.used();
    let records_start = esp_hal::time::Instant::now();
    let mut plaintext = [0; PLAINTEXT_BYTES];
    for _ in 0..128 {
        let payload = [0x5a; MAX_PAYLOAD];
        let n = client
            .seal(Kind::Message, &payload, &mut cipher, now())
            .unwrap();
        let record = server.open(&cipher[..n], &mut plaintext, now()).unwrap();
        assert_eq!(record.payload, payload);
        let n = server
            .seal(Kind::HeartbeatReply, &[], &mut cipher, now())
            .unwrap();
        assert_eq!(
            client
                .open(&cipher[..n], &mut plaintext, now())
                .unwrap()
                .kind,
            Kind::HeartbeatReply
        );
    }
    let records_us = records_start.elapsed().as_micros();
    // One altered tag closes the real crypto state and clears the output buffer.
    let n = client
        .seal(Kind::Heartbeat, &[], &mut cipher, now())
        .unwrap();
    cipher[n - 1] ^= 1;
    assert!(server.open(&cipher[..n], &mut plaintext, now()).is_err());
    assert_eq!(plaintext, [0; PLAINTEXT_BYTES]);
    assert!(server.peer(now()).is_err());
    drop((client, server, client_key, device_key));
    esp_println::println!(
        "SESSION LOCAL PASS handshake_us={} 128_roundtrips_us={} heap_before={} handshake_heap={} channel_heap={} heap_after={} heap_peak={} objects={}/{}",
        handshake_us,
        records_us,
        before,
        handshake_heap,
        channel_heap,
        esp_alloc::HEAP.used(),
        esp_alloc::HEAP.stats().max_usage,
        core::mem::size_of::<Handshake>(),
        core::mem::size_of::<Channel>()
    );
}
