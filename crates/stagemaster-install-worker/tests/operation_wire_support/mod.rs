use crate::{maintenance_support::Device, operation_support::admitted_pair};
use stagemaster_device_auth::application::{Permissions, Session};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Channel, Kind, PLAINTEXT_BYTES};
use stagemaster_install_worker::operations::{Connection, Error};
use stagemaster_runtime_protocol::{
    Access, Body, Offer, Operation, Peer, Ready, Request, Response,
};

pub struct Link {
    pub client: Channel,
    pub access: Session,
    pub connection: Connection,
    pub ready: Ready,
    pub serial: u64,
}
impl Link {
    pub fn open(device: &mut Device, permissions: Permissions, now: u64) -> Self {
        let state = device.state();
        let (client, mut access) = admitted_pair(state.boot, 9, permissions, now);
        let g = access.grant(now).unwrap();
        let connection = Connection::open(device, now, |t| access.grant(t).ok()).unwrap();
        let mut link = Self {
            client,
            access,
            connection,
            serial: 0,
            ready: Ready {
                peer: Peer {
                    device: g.context().device,
                    boot: g.context().boot,
                    connection: g.context().connection,
                    session: g.session(),
                    principal: g.principal(),
                    permission_revision: g.revision(),
                },
                version: 1,
                message_bytes: 1280,
                remaining_ms: u32::try_from(g.expires_at() - now).unwrap(),
                access: Access {
                    observe: permissions
                        .contains(stagemaster_device_auth::application::Scope::Observe),
                    control: permissions
                        .contains(stagemaster_device_auth::application::Scope::Control),
                    installation: permissions
                        .contains(stagemaster_device_auth::application::Scope::Installation),
                },
            },
        };
        let plain = link.transmit_to_device(Offer::current().encode().unwrap().bytes(), now);
        let frame = link
            .connection
            .negotiate(device, &plain, now, |t| link.access.grant(t).ok())
            .unwrap();
        let plain = link.transmit_to_client(frame.bytes(), now);
        let actual = Ready::decode(&plain).unwrap();
        actual.correlate(Offer::current(), &link.ready).unwrap();
        link.ready = actual;
        // Same-time negotiation only validates/ticks; no install gate or implicit input.
        assert_eq!(device.state(), state);
        link
    }
    fn transmit_to_device(&mut self, payload: &[u8], now: u64) -> Vec<u8> {
        let mut wire = [0; CIPHERTEXT_BYTES];
        let n = self
            .client
            .seal(Kind::Message, payload, &mut wire, now)
            .unwrap();
        let mut plain = [0; PLAINTEXT_BYTES];
        let record = self.access.open(&wire[..n], &mut plain, now).unwrap();
        assert_eq!(record.kind, Kind::Message);
        record.payload.to_vec()
    }
    fn transmit_to_client(&mut self, payload: &[u8], now: u64) -> Vec<u8> {
        let mut wire = [0; CIPHERTEXT_BYTES];
        let n = self
            .access
            .seal(Kind::Message, payload, &mut wire, now)
            .unwrap();
        let mut plain = [0; PLAINTEXT_BYTES];
        let record = self.client.open(&wire[..n], &mut plain, now).unwrap();
        assert_eq!(record.kind, Kind::Message);
        record.payload.to_vec()
    }
    pub fn send(
        &mut self,
        device: &mut Device,
        operation: Operation,
        now: u64,
    ) -> Result<Response, Error> {
        self.serial += 1;
        self.request(
            device,
            Request {
                session: self.ready.peer.session,
                id: self.serial,
                expected_revision: device.state().revision,
                operation,
            },
            now,
        )
    }
    pub fn request(
        &mut self,
        device: &mut Device,
        request: Request,
        now: u64,
    ) -> Result<Response, Error> {
        let wire = request.encode().unwrap();
        let decoded = self.transmit_to_device(wire.bytes(), now);
        let frame = self.connection.process_message(
            device,
            &decoded,
            || now,
            |t| self.access.grant(t).ok(),
        )?;
        let received = self.transmit_to_client(frame.bytes(), now);
        let reply = Response::decode(&received).unwrap();
        reply.correlate(request, self.ready.peer.boot).unwrap();
        Ok(reply)
    }
    pub fn ok(&mut self, device: &mut Device, operation: Operation, now: u64) -> Response {
        let reply = self.send(device, operation, now).unwrap();
        if let Body::State { result, .. } = reply.body {
            result.unwrap();
        }
        reply
    }
    pub fn raw(&mut self, device: &mut Device, bytes: &[u8], now: u64) -> Result<(), Error> {
        let decoded = self.transmit_to_device(bytes, now);
        self.connection
            .process_message(device, &decoded, || now, |t| self.access.grant(t).ok())
            .map(|_| ())
    }
}
