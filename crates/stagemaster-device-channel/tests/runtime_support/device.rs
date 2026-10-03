use crate::support::{self, peer::Peer};
use stagemaster_device_channel::{Error, RecordIo};
use stagemaster_device_info::Description;
use stagemaster_device_session::Kind;
use stagemaster_install::{Identity, Installer};
use stagemaster_install_store::FileStore;
use stagemaster_install_worker::{ManagedWorker, operations::Connection};
use stagemaster_runtime::{Denial, Permission, PlaybackPolicy};
use stagemaster_runtime_protocol::Access;
use std::{path::Path, time::Duration};
use tokio::time::Instant;

pub struct Allow;
impl PlaybackPolicy for Allow {
    fn authorize(&mut self, _: Permission) -> Result<(), Denial> {
        Ok(())
    }
}
pub type Device = ManagedWorker<FileStore, Allow>;
pub fn installed(path: &Path) -> Device {
    let bytes = support::package::package();
    let mut installer = Installer::open(FileStore::open(path).unwrap(), [2; 16])
        .unwrap()
        .0;
    let t = installer.transaction(1);
    installer
        .begin(
            t,
            Identity::from_archive(&stagemaster_package::Archive::open(bytes.as_slice()).unwrap()),
        )
        .unwrap();
    for (i, block) in bytes.chunks(1024).enumerate() {
        installer.write(t, i * 1024, block).unwrap();
    }
    installer.verify(t).unwrap();
    installer.commit(t).unwrap();
    drop(installer);
    let installer = Installer::open(FileStore::open(path).unwrap(), [2; 16])
        .unwrap()
        .0;
    let mut device = ManagedWorker::new(installer, 0, 64 * 1024, Allow).unwrap();
    device
        .confirm_quiescent(device.quiescence_request().unwrap(), 0)
        .unwrap();
    device.finish_maintenance(0).unwrap();
    device
}
pub struct Server<R: RecordIo> {
    pub peer: Peer<R>,
    pub device: Device,
    pub connection: Connection,
}
impl<R: RecordIo> Server<R> {
    pub async fn accept(
        io: R,
        desc: Description,
        mut device: Device,
        origin: Instant,
        access: Access,
    ) -> Self {
        let mut peer = Peer::authenticate_with(io, desc, Some(super::scopes(access)), origin)
            .await
            .unwrap();
        let now = support::now(origin);
        let mut connection = Connection::open(&device, now, |t| peer.secure.grant(t).ok()).unwrap();
        let (kind, offer) = peer.receive().await.unwrap();
        assert_eq!(kind, Kind::Message);
        let frame = connection
            .negotiate(&mut device, &offer, support::now(origin), |t| {
                peer.secure.grant(t).ok()
            })
            .unwrap();
        peer.send(Kind::Message, frame.bytes()).await.unwrap();
        Self {
            peer,
            device,
            connection,
        }
    }
    pub fn process(&mut self, bytes: &[u8]) -> stagemaster_runtime_protocol::Frame {
        let origin = self.peer.origin;
        self.connection
            .process_message(
                &mut self.device,
                bytes,
                || support::now(origin),
                |t| self.peer.secure.grant(t).ok(),
            )
            .unwrap()
    }
    pub async fn serve(mut self) -> Device {
        let mut interval = tokio::time::interval(Duration::from_millis(5));
        loop {
            let incoming = tokio::select! {
                _ = interval.tick() => {
                    let now = support::now(self.peer.origin);
                    self.device.tick(now).unwrap();
                    if self.connection.poll(&mut self.device, now, |t| self.peer.secure.grant(t).ok()).is_err() { break; }
                    continue;
                }
                result = self.peer.receive() => result,
            };
            let Ok((kind, bytes)) = incoming else {
                break;
            };
            let sent = match kind {
                Kind::Heartbeat => self.peer.send(Kind::HeartbeatReply, &[]).await,
                Kind::Message => {
                    let frame = self.process(&bytes);
                    self.peer.send(Kind::Message, frame.bytes()).await
                }
                Kind::HeartbeatReply => Err(Error::Denied),
            };
            if sent.is_err() {
                break;
            }
        }
        self.connection.close(&mut self.device).unwrap();
        self.peer.secure.revoke();
        self.peer.io.close();
        tokio::time::sleep(Duration::from_millis(30)).await;
        self.device.tick(support::now(self.peer.origin)).unwrap();
        self.device
    }
}
