#[path = "../../../stagemaster-device-session/tests/support/mod.rs"]
mod noise;
use core::sync::atomic::{AtomicU64, Ordering};
use stagemaster_device_auth::application::{DevelopmentPermit, Permissions, Scope, Session};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Context, HANDSHAKE_BYTES, Handshake};
use stagemaster_install::Storage;
use stagemaster_install_worker::{
    ManagedWorker,
    operations::{Connection, Error, Operation, Reply, Request},
};
use stagemaster_runtime::{Action, PlaybackPolicy, ProgramKey};

pub fn all() -> Permissions {
    Permissions::only(Scope::Observe)
        .with(Scope::Control)
        .with(Scope::Installation)
}
pub fn access(boot: [u8; 16], principal: u8, permissions: Permissions, now: u64) -> Session {
    admitted_pair(boot, principal, permissions, now).1
}
pub fn admitted_pair(
    boot: [u8; 16],
    principal: u8,
    permissions: Permissions,
    now: u64,
) -> (stagemaster_device_session::Channel, Session) {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let context = Context {
        device: [1; 16],
        boot,
        connection: NEXT.fetch_add(1, Ordering::Relaxed),
    };
    let mut client = Handshake::initiate(
        context,
        &noise::key(3),
        noise::key(4).public(),
        noise::entropy,
        now,
    )
    .unwrap();
    let mut server = Handshake::respond(context, &noise::key(4), noise::entropy, now).unwrap();
    let mut wire = [0; HANDSHAKE_BYTES];
    let n = client.write(&mut wire, now).unwrap();
    server.read(&wire[..n], now).unwrap();
    let n = server.write(&mut wire, now).unwrap();
    client.read(&wire[..n], now).unwrap();
    let mut client = client.finish(now).unwrap();
    let mut server = server.finish(now).unwrap();
    let mut cipher = [0; CIPHERTEXT_BYTES];
    let n = client.confirmation(&mut cipher, now).unwrap();
    server.confirm(&cipher[..n], now).unwrap();
    let n = server.confirmation(&mut cipher, now).unwrap();
    client.confirm(&cipher[..n], now).unwrap();
    let session = Session::admit(
        server,
        DevelopmentPermit::scoped(
            context.device,
            noise::key(3).public(),
            [principal; 16],
            1,
            10_000,
            permissions,
        )
        .unwrap(),
        context,
        now,
    )
    .unwrap();
    (client, session)
}
pub struct Peer {
    pub access: Session,
    pub connection: Connection,
    pub session: [u8; 16],
    id: u64,
}
impl Peer {
    pub fn new<S: Storage, P: PlaybackPolicy>(
        device: &ManagedWorker<S, P>,
        principal: u8,
        permissions: Permissions,
        now: u64,
    ) -> Self {
        let mut access = access(device.state().boot, principal, permissions, now);
        let session = access.grant(now).unwrap().session();
        let connection = Connection::open(device, now, |time| access.grant(time).ok()).unwrap();
        Self {
            access,
            connection,
            session,
            id: 0,
        }
    }
    pub fn send<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        operation: Operation,
        now: u64,
    ) -> Result<Reply, Error> {
        self.id += 1;
        let request = Request {
            session: self.session,
            id: self.id,
            expected_revision: device.state().revision,
            operation,
        };
        self.request(device, request, now)
    }
    pub fn request<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        request: Request,
        now: u64,
    ) -> Result<Reply, Error> {
        self.connection
            .process(device, request, || now, |time| self.access.grant(time).ok())
    }
    pub fn apply<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        action: Action,
        now: u64,
    ) -> Reply {
        let reply = self.send(device, Operation::Apply(action), now).unwrap();
        reply.result.unwrap();
        reply
    }
    pub fn prepare<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
    ) -> usize {
        self.send(
            device,
            Operation::Acquire {
                duration_ms: 1000,
                takeover: false,
            },
            0,
        )
        .unwrap()
        .result
        .unwrap();
        self.send(device, Operation::FinishMaintenance, 0)
            .unwrap()
            .result
            .unwrap();
        let index = device
            .catalog()
            .iter()
            .position(|e| e.kind == stagemaster_package::Kind::Sequence)
            .unwrap();
        let entry = &device.catalog()[index];
        let key = ProgramKey {
            kind: entry.kind,
            id: entry.id,
        };
        self.apply(device, Action::Select(key), 0);
        self.apply(device, Action::Load, 0);
        index
    }
}
