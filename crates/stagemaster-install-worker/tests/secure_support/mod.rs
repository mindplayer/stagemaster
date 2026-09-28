#[path = "../../../stagemaster-device-session/tests/support/mod.rs"]
pub mod noise;
use stagemaster_device_auth::application::{DevelopmentPermit, Session};
use stagemaster_device_link::management::ApplicationReceipt;
use stagemaster_device_session::{CIPHERTEXT_BYTES, Channel, Kind, PLAINTEXT_BYTES};
use stagemaster_install_worker::{Command, Completion, Epoch, Reply, secure::Gateway};

pub struct Pair {
    pub client: Channel,
    pub gateway: Gateway,
    pub session: [u8; 16],
    upstream: Wire,
    downstream: Wire,
}
struct Wire {
    sender: stagemaster_device_link::secure::Sender,
    receiver: stagemaster_device_link::secure::Receiver,
}
impl Wire {
    fn new(budget: usize) -> Self {
        Self {
            sender: stagemaster_device_link::secure::Sender::new(budget, 0).unwrap(),
            receiver: stagemaster_device_link::secure::Receiver::new(budget, 0).unwrap(),
        }
    }
    fn pass(&mut self, bytes: &[u8], now: u64) -> Vec<u8> {
        self.sender.queue(bytes, now).unwrap();
        let mut received = None;
        while let Some(packet) = self.sender.fragment(now).unwrap() {
            assert!(received.is_none());
            received = self
                .receiver
                .push(packet.bytes(), now)
                .unwrap()
                .map(|r| r.bytes().to_vec());
            assert_eq!(self.sender.sent(now).unwrap(), received.is_some());
        }
        received.unwrap()
    }
}
impl Pair {
    pub fn new(epoch: u32, duration: u32) -> (Self, Command) {
        let (mut client, server) = noise::channels();
        let context = noise::context();
        let permit = DevelopmentPermit::installation(
            context.device,
            noise::key(3).public(),
            [9; 16],
            7,
            duration,
        )
        .unwrap();
        let access = Session::admit(server, permit, context, 0).unwrap();
        let session = client.peer(0).unwrap().unwrap().session();
        let (gateway, command) = Gateway::open(access, Epoch::new(epoch).unwrap(), 0).unwrap();
        (
            Self {
                client,
                gateway,
                session,
                upstream: Wire::new(20),
                downstream: Wire::new(244),
            },
            command,
        )
    }
    pub fn mark_opened(&mut self, epoch: u32) {
        self.gateway
            .complete(
                Completion {
                    epoch: Epoch::new(epoch).unwrap(),
                    result: Ok(Reply::Opened),
                },
                0,
            )
            .unwrap();
        self.ready();
    }
    pub fn ready(&mut self) {
        let bytes = self.receive(Kind::Message, 0);
        let receipt = ApplicationReceipt::decode(&bytes).unwrap();
        let context = noise::context();
        receipt
            .correlate(&ApplicationReceipt {
                device: context.device,
                boot: context.boot,
                diagnostic: context.connection,
                session: self.session,
                principal: [9; 16],
                revision: 7,
                remaining_ms: 600_000,
            })
            .unwrap();
    }
    pub fn cipher(&mut self, kind: Kind, bytes: &[u8], now: u64) -> Vec<u8> {
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = self.client.seal(kind, bytes, &mut cipher, now).unwrap();
        cipher[..n].to_vec()
    }
    pub fn send(&mut self, kind: Kind, bytes: &[u8], now: u64) -> Option<Command> {
        let cipher = self.cipher(kind, bytes, now);
        let cipher = self.upstream.pass(&cipher, now);
        self.gateway.receive(&cipher, now).unwrap()
    }
    pub fn receive(&mut self, kind: Kind, now: u64) -> Vec<u8> {
        let cipher = self.gateway.outbound(now).unwrap().unwrap().to_vec();
        assert_eq!(self.gateway.outbound(now).unwrap().unwrap(), cipher);
        let cipher = self.downstream.pass(&cipher, now);
        let mut plain = [0; PLAINTEXT_BYTES];
        let record = self.client.open(&cipher, &mut plain, now).unwrap();
        assert_eq!(record.kind, kind);
        let bytes = record.payload.to_vec();
        self.gateway.sent(now).unwrap();
        bytes
    }
    pub fn query(&self) -> stagemaster_transfer::Frame {
        stagemaster_transfer::Request {
            link: self.session,
            id: 1,
            action: stagemaster_transfer::Action::Status,
        }
        .encode()
        .unwrap()
    }
}
