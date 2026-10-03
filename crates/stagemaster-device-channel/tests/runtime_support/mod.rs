#![allow(dead_code)]
pub mod device;
use crate::support;
pub use device::{Server, installed};
use stagemaster_device_auth::application::{Permissions, Scope};
use stagemaster_device_channel::{RecordIo, runtime::RuntimeClient};
use stagemaster_device_info::{Description, capability};
use stagemaster_runtime_protocol::{Access, Body, Operation, Response};
use std::time::Duration;

pub fn rights() -> Access {
    Access {
        observe: true,
        control: true,
        installation: false,
    }
}
pub fn scopes(access: Access) -> Permissions {
    let mut p = if access.observe {
        Permissions::only(Scope::Observe)
    } else {
        Permissions::only(Scope::Control)
    };
    if access.control {
        p = p.with(Scope::Control);
    }
    if access.installation {
        p = p.with(Scope::Installation);
    }
    p
}
pub fn description(connection: u64) -> Description {
    let mut desc = support::description(connection);
    desc.capabilities = capability::DIAGNOSTICS
        | capability::CATALOG
        | capability::PLAYBACK
        | capability::RUNTIME_APPLICATION;
    desc.limits.transfer_version = 0;
    desc.limits.chunk_bytes = 0;
    desc.limits.slot_bytes = 0;
    desc.limits.universes = 1;
    desc.limits.loader_bytes = 64 * 1024;
    desc.limits.frame_ms = 25;
    desc.validate().unwrap();
    desc
}
pub async fn reply<R: RecordIo>(client: &mut RuntimeClient<R>) -> Response {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(r) = client.receive().unwrap() {
                return r;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap()
}
pub async fn command<R: RecordIo>(client: &mut RuntimeClient<R>, operation: Operation) -> Response {
    let revision = client.last_response().map_or(0, |r| r.observed.revision);
    client.send(operation, revision).await.unwrap();
    // The reply may precede a heartbeat acknowledgement, including on fragmented carriers.
    client.heartbeat().await.unwrap();
    let r = reply(client).await;
    if let Body::State { result, .. } = r.body {
        result.unwrap();
    }
    r
}

pub type TestRecords = stagemaster_device_channel::StreamRecords<tokio::io::DuplexStream>;
pub fn connected(
    access: Access,
) -> impl std::future::Future<
    Output = (
        tempfile::TempDir,
        RuntimeClient<TestRecords>,
        Server<TestRecords>,
    ),
> {
    Box::pin(connect(access))
}
async fn connect(
    access: Access,
) -> (
    tempfile::TempDir,
    RuntimeClient<TestRecords>,
    Server<TestRecords>,
) {
    let dir = support::temporary();
    let device = installed(dir.path());
    let (client, server) = tokio::io::duplex(4096);
    let origin = tokio::time::Instant::now();
    let config = support::config(stagemaster_device_auth::application::Role::Controller);
    let (channel, peer) = tokio::join!(
        stagemaster_device_channel::Channel::prepare_runtime(
            TestRecords::new(client),
            description(7),
            &config,
            access
        ),
        Server::accept(
            TestRecords::new(server),
            description(7),
            device,
            origin,
            access
        ),
    );
    (dir, RuntimeClient::new(channel.unwrap()).unwrap(), peer)
}
