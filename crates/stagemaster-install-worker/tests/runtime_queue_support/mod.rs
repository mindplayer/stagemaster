use crate::{
    maintenance_support::{Device, epoch},
    operation_support::{admitted_pair_with_duration, all},
};
use stagemaster_device_auth::application::Permissions;
use stagemaster_device_session::{CIPHERTEXT_BYTES, Channel, Kind, PLAINTEXT_BYTES};
use stagemaster_install_worker::runtime_queue::{
    Command, Completion, Endpoint, Error, Gateway, Live,
};
use stagemaster_runtime_protocol::{Body, Offer, Operation, Ready, Request, Response};
use std::cell::Cell;

pub fn assert_reference(device: &Device, index: usize, now: u64) {
    let loaded = device.snapshot().unwrap().load(index).unwrap();
    let mut reference = stagemaster_playback::Player::new(loaded.plan, 0);
    reference.execute(0, 0).unwrap();
    reference.advance(now).unwrap();
    let mut expected = [0; 512];
    loaded
        .output
        .render(reference.values(), &mut expected)
        .unwrap();
    let mut actual = [0; 512];
    device.render(&mut actual).unwrap().unwrap();
    assert_eq!(actual, expected);
}

pub struct Link {
    pub gateway: Gateway,
    pub client: Channel,
    pub slot: Cell<Option<Live>>,
    pub session: [u8; 16],
    serial: u64,
}
impl Link {
    pub fn new(device: &Device, generation: u32, permissions: Permissions, now: u64) -> Self {
        let (client, mut access) =
            admitted_pair_with_duration(device.state().boot, 9, permissions, now, 60_000);
        let session = access.grant(now).unwrap().session();
        let mut gateway = Gateway::new(access, epoch(generation), now).unwrap();
        let slot = Cell::new(gateway.live(now));
        Self {
            gateway,
            client,
            slot,
            session,
            serial: 0,
        }
    }
    pub fn open(device: &mut Device, endpoint: &mut Endpoint, generation: u32, now: u64) -> Self {
        let mut link = Self::new(device, generation, all(), now);
        link.negotiate(device, endpoint, now);
        link
    }
    pub fn negotiate(&mut self, device: &mut Device, endpoint: &mut Endpoint, now: u64) {
        let before = device.state();
        let command = self.offer(now);
        self.work(device, endpoint, command, now).unwrap();
        let (kind, bytes) = self.outgoing(now);
        assert_eq!(kind, Kind::Message);
        let ready = Ready::decode(&bytes).unwrap();
        assert_eq!(ready.peer.session, self.session);
        assert!(ready.access.observe);
        assert_eq!(device.state(), before);
    }
    pub fn offer(&mut self, now: u64) -> Command {
        self.receive(
            Kind::Message,
            Offer::current().encode().unwrap().bytes(),
            now,
        )
        .unwrap()
        .unwrap()
    }
    pub fn receive(
        &mut self,
        kind: Kind,
        bytes: &[u8],
        now: u64,
    ) -> Result<Option<Command>, Error> {
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = self.client.seal(kind, bytes, &mut cipher, now).unwrap();
        let result = self.gateway.receive(&cipher[..n], now);
        self.slot.set(self.gateway.live(now));
        result
    }
    pub fn complete(&mut self, completion: Completion, now: u64) -> Result<bool, Error> {
        let result = self.gateway.complete(completion, now);
        self.slot.set(self.gateway.live(now));
        result
    }
    pub fn work(
        &mut self,
        device: &mut Device,
        endpoint: &mut Endpoint,
        command: Command,
        now: u64,
    ) -> Result<bool, Error> {
        let completion = endpoint.process(device, command, || now, || self.slot.get());
        self.complete(completion, now)
    }
    pub fn outgoing(&mut self, now: u64) -> (Kind, Vec<u8>) {
        let bytes = self.gateway.outbound(now).unwrap().unwrap().to_vec();
        assert_eq!(self.gateway.outbound(now).unwrap().unwrap(), bytes);
        let mut plain = [0; PLAINTEXT_BYTES];
        let record = self.client.open(&bytes, &mut plain, now).unwrap();
        let result = (record.kind, record.payload.to_vec());
        self.gateway.sent(now).unwrap();
        self.slot.set(self.gateway.live(now));
        result
    }
    pub fn heartbeat(&mut self, now: u64) {
        assert!(self.receive(Kind::Heartbeat, &[], now).unwrap().is_none());
        assert_eq!(self.outgoing(now), (Kind::HeartbeatReply, Vec::new()));
    }
    pub fn request(&mut self, device: &Device, operation: Operation) -> Request {
        self.serial += 1;
        Request {
            session: self.session,
            id: self.serial,
            expected_revision: device.state().revision,
            operation,
        }
    }
    pub fn queue(&mut self, request: Request, now: u64) -> Command {
        self.receive(Kind::Message, request.encode().unwrap().bytes(), now)
            .unwrap()
            .unwrap()
    }
    pub fn call(
        &mut self,
        device: &mut Device,
        endpoint: &mut Endpoint,
        operation: Operation,
        now: u64,
    ) -> Response {
        let request = self.request(device, operation);
        self.send(device, endpoint, request, now)
    }
    pub fn send(
        &mut self,
        device: &mut Device,
        endpoint: &mut Endpoint,
        request: Request,
        now: u64,
    ) -> Response {
        let command = self.queue(request, now);
        self.work(device, endpoint, command, now).unwrap();
        let (kind, bytes) = self.outgoing(now);
        assert_eq!(kind, Kind::Message);
        let response = Response::decode(&bytes).unwrap();
        response.correlate(request, device.state().boot).unwrap();
        response
    }
    pub fn ok(
        &mut self,
        device: &mut Device,
        endpoint: &mut Endpoint,
        operation: Operation,
        now: u64,
    ) -> Response {
        let response = self.call(device, endpoint, operation, now);
        if let Body::State { result, .. } = response.body {
            result.unwrap();
        }
        response
    }
    pub fn acquire(&mut self, device: &mut Device, endpoint: &mut Endpoint, now: u64) {
        self.ok(
            device,
            endpoint,
            Operation::Acquire {
                duration_ms: 1000,
                takeover: false,
            },
            now,
        );
    }
    pub fn close(&mut self) {
        self.gateway.close();
        self.slot.set(None);
    }
}
