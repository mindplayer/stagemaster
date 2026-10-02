#![allow(dead_code)] // Shared by separately compiled integration tests.
pub mod packets;
pub mod peer;
use stagemaster_device_auth::application::{Configuration, Role};
use stagemaster_device_channel::{Channel, Error, RecordIo, StreamRecords};
use stagemaster_device_info::{Description, Firmware, Limits, capability};
use stagemaster_device_session::{Context, SecretKey};
use std::{path::PathBuf, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    time::Instant,
};

pub fn config(role: Role) -> Configuration {
    config_with_trusted(role, if role == Role::Controller { 5 } else { 4 })
}
pub fn config_with_trusted(role: Role, trusted: u8) -> Configuration {
    let (mine, code) = if role == Role::Controller {
        (4, 2)
    } else {
        (5, 1)
    };
    let mut bytes = [0; 160];
    bytes[..8].copy_from_slice(b"SMDV\x01\0\xa0\0");
    bytes[5] = code;
    bytes[8..24].fill(1);
    bytes[24..40].fill(9);
    bytes[40..48].copy_from_slice(&3_u64.to_le_bytes());
    bytes[48..52].copy_from_slice(&60_000_u32.to_le_bytes());
    bytes[56..88].fill(mine);
    bytes[88..120].copy_from_slice(&SecretKey::import([trusted; 32]).unwrap().public());
    bytes[120..152].copy_from_slice(&SecretKey::import([mine; 32]).unwrap().public());
    Configuration::import(&bytes, role).unwrap()
}
pub fn description(connection: u64) -> Description {
    Description {
        device: [1; 16],
        boot: [2; 16],
        session: connection,
        model: 1,
        firmware: Firmware::default(),
        capabilities: capability::DIAGNOSTICS | capability::CATALOG | capability::INSTALLATION,
        authentication: stagemaster_device_session::AUTHENTICATION,
        limits: Limits {
            package_version: 1,
            transfer_version: 1,
            package_bytes: 1024 * 1024,
            programs: 100,
            message_bytes: 1280,
            chunk_bytes: 1024,
            slot_bytes: 2 * 1024 * 1024,
            ..Limits::default()
        },
    }
}
pub fn context(desc: Description) -> Context {
    Context {
        device: desc.device,
        boot: desc.boot,
        connection: desc.session,
    }
}
pub fn entropy(bytes: &mut [u8]) -> Result<(), stagemaster_device_session::Error> {
    getrandom::fill(bytes).map_err(|_| stagemaster_device_session::Error::Entropy)
}
pub fn now(origin: Instant) -> u64 {
    u64::try_from(origin.elapsed().as_millis()).unwrap()
}
pub async fn tcp() -> (StreamRecords<TcpStream>, StreamRecords<TcpStream>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (client, server) = tokio::join!(
        TcpStream::connect(listener.local_addr().unwrap()),
        listener.accept()
    );
    let client = client.unwrap();
    let server = server.unwrap().0;
    client.set_nodelay(true).unwrap();
    server.set_nodelay(true).unwrap();
    (StreamRecords::new(client), StreamRecords::new(server))
}
pub async fn message<R: RecordIo>(channel: &mut Channel<R>) -> Result<Vec<u8>, Error> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(bytes) = channel.receive()? {
                return Ok(bytes);
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap()
}
pub fn temporary() -> tempfile::TempDir {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    std::fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}
pub struct Task<T>(pub tokio::task::JoinHandle<T>);
impl<T> Drop for Task<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}
