mod support;
use stagemaster_device_link::secure::{MAX_RECORD_BYTES, Receiver, Sender};
use stagemaster_device_session::*;
use support::*;

struct Direction {
    tx: Sender,
    rx: Receiver,
}
impl Direction {
    fn new(budget: usize) -> Self {
        Self {
            tx: Sender::new(budget, 0).unwrap(),
            rx: Receiver::new(budget, 0).unwrap(),
        }
    }
    fn transfer(&mut self, bytes: &[u8], now: u64) -> Vec<u8> {
        self.tx.queue(bytes, now).unwrap();
        while let Some(packet) = self.tx.fragment(now).unwrap() {
            let record = self.rx.push(packet.bytes(), now).unwrap();
            let done = self.tx.sent(now).unwrap();
            assert_eq!(done, record.is_some());
            if let Some(record) = record {
                return record.bytes().to_vec();
            }
        }
        panic!("record was not completed")
    }
}

#[test]
fn full_noise_handshake_confirmation_and_bidirectional_messages_cross_both_mtu_limits() {
    assert_eq!(CIPHERTEXT_BYTES, MAX_RECORD_BYTES);
    for budget in [20, 244] {
        let mut forward = Direction::new(budget);
        let mut backward = Direction::new(budget);
        let (mut client, mut server) = handshakes();
        let mut hs = [0; HANDSHAKE_BYTES];
        let n = client.write(&mut hs, 0).unwrap();
        server.read(&forward.transfer(&hs[..n], 0), 0).unwrap();
        let n = server.write(&mut hs, 0).unwrap();
        client.read(&backward.transfer(&hs[..n], 0), 0).unwrap();
        let mut client = client.finish(0).unwrap();
        let mut server = server.finish(0).unwrap();
        let mut wire = [0; CIPHERTEXT_BYTES];
        let n = client.confirmation(&mut wire, 0).unwrap();
        server.confirm(&forward.transfer(&wire[..n], 0), 0).unwrap();
        let n = server.confirmation(&mut wire, 0).unwrap();
        client
            .confirm(&backward.transfer(&wire[..n], 0), 0)
            .unwrap();
        let mut plain = [0; PLAINTEXT_BYTES];
        for now in 1..=256 {
            let payload = [0x5a; MAX_PAYLOAD];
            let n = client
                .seal(Kind::Message, &payload, &mut wire, now)
                .unwrap();
            let received = forward.transfer(&wire[..n], now);
            assert_eq!(
                server.open(&received, &mut plain, now).unwrap().payload,
                payload
            );
            let n = server
                .seal(Kind::HeartbeatReply, &[], &mut wire, now)
                .unwrap();
            assert_eq!(
                client
                    .open(&backward.transfer(&wire[..n], now), &mut plain, now)
                    .unwrap()
                    .kind,
                Kind::HeartbeatReply
            );
        }
        // Framing accepts structurally sound bytes; only Noise establishes authenticity.
        let n = server
            .seal(Kind::Message, b"bad tag", &mut wire, 257)
            .unwrap();
        wire[n - 1] ^= 1;
        let received = backward.transfer(&wire[..n], 257);
        assert_eq!(
            client.open(&received, &mut plain, 257).err(),
            Some(Error::Crypto)
        );
        assert_eq!(plain, [0; PLAINTEXT_BYTES]);
    }
}
