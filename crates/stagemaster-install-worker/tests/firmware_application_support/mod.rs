#[path = "../../../../apps/esp32-player/src/ble/application/gate.rs"]
mod gate;
#[path = "../../../stagemaster-device-session/tests/support/mod.rs"]
mod noise;
#[path = "../../../../apps/esp32-player/src/ble/application/protocol.rs"]
mod protocol;
use crate::maintenance_support::Device;
use gate::{Command, Completion, Mode, Port, Validity};
use stagemaster_device_auth::application::{Configuration, Role};
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Channel, Context, HANDSHAKE_BYTES, Handshake, Kind, PLAINTEXT_BYTES,
};
use stagemaster_install_worker::{Epoch, runtime_queue::Endpoint};
use stagemaster_runtime_protocol::{Offer, Operation, Ready, Request, Response};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Default)]
pub struct Slots {
    live: Cell<Option<Validity>>,
    request: RefCell<Option<Command>>,
    completion: RefCell<Option<Completion>>,
    pub full: Cell<bool>,
}
impl Slots {
    pub fn revoked(&self) -> bool {
        self.live.get().is_none()
    }
    pub fn runtime(&self) -> Option<stagemaster_install_worker::runtime_queue::Live> {
        match self.live.get() {
            Some(Validity::Runtime(live)) => Some(live),
            _ => None,
        }
    }
    pub fn installation(&self) -> Option<Epoch> {
        match self.live.get() {
            Some(Validity::Installation(epoch)) => Some(epoch),
            _ => None,
        }
    }
}
struct Queues(Rc<Slots>);
impl Port for Queues {
    fn publish(&mut self, live: Option<Validity>) {
        self.0.live.set(live);
    }
    fn enqueue(&mut self, command: Command) -> gate::Result<()> {
        assert!(
            !self.0.revoked(),
            "must publish before enqueueing on another core"
        );
        if self.0.full.get() || self.0.request.borrow().is_some() {
            return Err("队列已满");
        }
        *self.0.request.borrow_mut() = Some(command);
        Ok(())
    }
    fn completion(&mut self) -> Option<Completion> {
        self.0.completion.borrow_mut().take()
    }
}
pub fn configuration(version: u8, bits: u8) -> Configuration {
    let mut data = [0; 160];
    data[..8].copy_from_slice(b"SMDV\x01\x01\xa0\0");
    data[4] = version;
    data[8..24].fill(1);
    data[24..40].fill(9);
    data[40..48].copy_from_slice(&1_u64.to_le_bytes());
    data[48..52].copy_from_slice(&60_000_u32.to_le_bytes());
    data[52] = bits;
    data[56..88].fill(4);
    data[88..120].copy_from_slice(&noise::key(3).public());
    data[120..152].copy_from_slice(&noise::key(4).public());
    Configuration::import(&data, Role::Device).unwrap()
}
pub struct Link {
    server: protocol::Protocol<Queues>,
    pub client: Channel,
    pub slots: Rc<Slots>,
    pub endpoint: Endpoint,
    pub serial: u64,
}
impl Link {
    pub fn connect(device: &Device, runtime: bool, version: u8, bits: u8) -> gate::Result<Self> {
        let config = configuration(version, bits);
        let context = Context {
            device: config.device(),
            boot: device.state().boot,
            connection: 1,
        };
        let mode = if runtime {
            Mode::Runtime
        } else {
            Mode::Installation
        };
        let slots = Rc::new(Slots::default());
        let mut server = protocol::Protocol::new(
            protocol::Settings {
                context,
                epoch: Epoch::new(1).unwrap(),
                mode,
            },
            &config,
            noise::entropy,
            0,
            Queues(slots.clone()),
        )?;
        let mut client = Handshake::initiate(
            context,
            &noise::key(3),
            config.key().public(),
            noise::entropy,
            0,
        )
        .unwrap();
        let mut handshake = [0; HANDSHAKE_BYTES];
        let n = client.write(&mut handshake, 0).unwrap();
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = server.receive(&handshake[..n], &mut cipher, 0)?.unwrap();
        client.read(&cipher[..n], 0).unwrap();
        let mut client = client.finish(0).unwrap();
        let n = client.confirmation(&mut cipher, 0).unwrap();
        let mut out = [0; CIPHERTEXT_BYTES];
        let n = server.receive(&cipher[..n], &mut out, 0)?.unwrap();
        client.confirm(&out[..n], 0).unwrap();
        Ok(Self {
            server,
            client,
            slots,
            endpoint: Endpoint::default(),
            serial: 0,
        })
    }
    pub fn pump(&mut self, device: &mut Device, now: u64) -> gate::Result<()> {
        let command = self.slots.request.borrow_mut().take();
        if let Some(command) = command {
            let completion = match command {
                Command::Installation(command) => {
                    Completion::Installation(
                        device.process(command, now, || self.slots.installation()),
                    )
                }
                Command::Runtime(command) => Completion::Runtime(self.endpoint.process(
                    device,
                    command,
                    || now,
                    || self.slots.runtime(),
                )),
            };
            assert!(self.slots.completion.borrow().is_none());
            *self.slots.completion.borrow_mut() = Some(completion);
        }
        self.server.poll(now)
    }
    pub fn receive(&mut self, kind: Kind, bytes: &[u8], now: u64) -> gate::Result<()> {
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = self.client.seal(kind, bytes, &mut cipher, now).unwrap();
        assert!(
            self.server
                .receive(&cipher[..n], &mut [0; CIPHERTEXT_BYTES], now)?
                .is_none()
        );
        Ok(())
    }
    pub fn outgoing(&mut self, now: u64) -> (Kind, Vec<u8>) {
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = self.server.outbound(&mut cipher, now).unwrap().unwrap();
        let mut plain = [0; PLAINTEXT_BYTES];
        let record = self.client.open(&cipher[..n], &mut plain, now).unwrap();
        let result = (record.kind, record.payload.to_vec());
        self.server.sent(now).unwrap();
        result
    }
    pub fn negotiate(&mut self, device: &mut Device) {
        self.receive(Kind::Message, Offer::current().encode().unwrap().bytes(), 0)
            .unwrap();
        self.pump(device, 0).unwrap();
        Ready::decode(&self.outgoing(0).1).unwrap();
    }
    pub fn call(&mut self, device: &mut Device, operation: Operation, now: u64) -> Response {
        self.serial += 1;
        let request = Request {
            session: self.client.peer(now).unwrap().unwrap().session(),
            id: self.serial,
            expected_revision: device.state().revision,
            operation,
        };
        self.receive(Kind::Message, request.encode().unwrap().bytes(), now)
            .unwrap();
        self.pump(device, now).unwrap();
        let response = Response::decode(&self.outgoing(now).1).unwrap();
        response.correlate(request, device.state().boot).unwrap();
        response
    }
    pub fn poll(&mut self, now: u64) -> gate::Result<()> {
        self.server.poll(now)
    }
    pub fn close(&mut self) {
        self.server.close();
    }
}
