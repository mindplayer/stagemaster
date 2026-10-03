use crate::{
    runtime_support::{self, Server, device::Device},
    support,
};
use stagemaster_device_auth::application::Role;
use stagemaster_device_channel::{Channel, StreamRecords, runtime::RuntimeClient};
use stagemaster_device_host::{
    Candidate, Phase, Problem, ProblemCode as C, Request, RuntimeIntent, Service, Snapshot,
    Transport,
};
use stagemaster_device_link::Session;
use stagemaster_runtime_protocol::{Access, Ready, Request as Command, Response};
use std::{
    future::pending,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{io::DuplexStream, time::Instant};

#[path = "server.rs"]
mod server;
type Client = RuntimeClient<support::faults::FaultIo<StreamRecords<DuplexStream>>>;
pub type Host = Service<Adapter>;
pub struct State {
    pub device: Option<Device>,
    pub observation: stagemaster_runtime::State,
    pub commands: Vec<Command>,
    pub hold_reply: bool,
    pub wrong_description: bool,
    pub connects: u8,
    pub disconnects: u8,
    pub beats: usize,
    pub send_fault: Arc<std::sync::atomic::AtomicU8>,
    pub(crate) origin: Instant,
    pub(crate) _dir: tempfile::TempDir,
}
pub struct Adapter {
    data: Arc<Mutex<State>>,
    client: Option<Client>,
    task: Option<tokio::task::JoinHandle<()>>,
    diagnostic: Option<Session>,
    reply: Vec<u8>,
    discovered: bool,
    epoch: u64,
}
impl Drop for Adapter {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
pub fn setup() -> (Host, Arc<Mutex<State>>) {
    let dir = support::temporary();
    let device = runtime_support::installed(dir.path());
    let data = Arc::new(Mutex::new(State {
        observation: device.state(),
        device: Some(device),
        commands: vec![],
        hold_reply: false,
        wrong_description: false,
        connects: 0,
        disconnects: 0,
        beats: 0,
        send_fault: Arc::new(std::sync::atomic::AtomicU8::new(0)),
        origin: Instant::now(),
        _dir: dir,
    }));
    let adapter = Adapter {
        data: data.clone(),
        client: None,
        task: None,
        diagnostic: None,
        reply: vec![],
        discovered: false,
        epoch: 0,
    };
    (Service::new(adapter), data)
}
impl Adapter {
    fn init(&mut self) {
        self.data.lock().unwrap().connects += 1;
        self.epoch += 1;
        self.diagnostic = Some(Session::new(self.epoch, 0).unwrap());
    }
}
impl Transport for Adapter {
    async fn start_scan(&mut self) -> Result<(), Problem> {
        self.discovered = false;
        Ok(())
    }
    async fn discover(&mut self) -> Result<Candidate, Problem> {
        if std::mem::replace(&mut self.discovered, true) {
            pending().await
        } else {
            Ok(Candidate {
                id: "runtime".into(),
                name: "运行服务验收".into(),
                rssi: None,
            })
        }
    }
    async fn stop_scan(&mut self) -> Result<(), Problem> {
        Ok(())
    }
    async fn connect(&mut self, _: &str) -> Result<(), Problem> {
        self.init();
        Ok(())
    }
    async fn connect_runtime(&mut self, _: &str, access: Access) -> Result<(), Problem> {
        self.init();
        let (device, origin) = {
            let mut s = self.data.lock().unwrap();
            (s.device.take().unwrap(), s.origin)
        };
        let desc = runtime_support::description(self.epoch);
        let (client, server) = tokio::io::duplex(4096);
        let data = self.data.clone();
        self.task = Some(tokio::spawn(async move {
            let server =
                Server::accept(StreamRecords::new(server), desc, device, origin, access).await;
            server::serve(server, data).await;
        }));
        let mode = self.data.lock().unwrap().send_fault.clone();
        let channel = Channel::prepare_runtime(
            support::faults::FaultIo {
                inner: StreamRecords::new(client),
                mode,
            },
            desc,
            &support::config(Role::Controller),
            access,
        )
        .await
        .map_err(|e| problem(&e))?;
        self.client = Some(RuntimeClient::new(channel).map_err(|e| problem(&e))?);
        Ok(())
    }
    fn runtime_peer(&self) -> Option<Ready> {
        self.client.as_ref().and_then(Client::peer)
    }
    fn runtime_pending(&self) -> Option<Command> {
        self.client.as_ref().and_then(Client::pending)
    }
    async fn send_runtime(&mut self, intent: RuntimeIntent) -> Result<Command, Problem> {
        self.client
            .as_mut()
            .unwrap()
            .send(intent.operation, intent.expected_revision)
            .await
            .map_err(|e| problem(&e))
    }
    fn try_runtime_response(&mut self) -> Result<Option<Response>, Problem> {
        self.client
            .as_mut()
            .unwrap()
            .receive()
            .map_err(|e| problem(&e))
    }
    async fn write(&mut self, bytes: &[u8; 20]) -> Result<(), Problem> {
        if let Some(client) = &mut self.client {
            client.heartbeat().await.map_err(|e| problem(&e))?;
        }
        self.reply = self
            .diagnostic
            .as_mut()
            .unwrap()
            .receive(bytes, 0)
            .encode()
            .to_vec();
        self.data.lock().unwrap().beats += 1;
        Ok(())
    }
    async fn reply(&mut self) -> Result<Vec<u8>, Problem> {
        Ok(self.reply.clone())
    }
    async fn diagnostics(&mut self) -> Result<Vec<u8>, Problem> {
        let mut bytes = vec![0; 20];
        bytes[0] = 1;
        bytes[1] = 3;
        bytes[16..20].copy_from_slice(&131_072u32.to_le_bytes());
        Ok(bytes)
    }
    async fn description(&mut self) -> Result<Option<Vec<u8>>, Problem> {
        let mut desc = runtime_support::description(self.epoch);
        if self.data.lock().unwrap().wrong_description {
            desc.boot = [7; 16];
        }
        Ok(Some(desc.encode().unwrap().to_vec()))
    }
    async fn disconnect(&mut self) -> Result<(), Problem> {
        self.client = None;
        if let Some(task) = self.task.take() {
            task.await.unwrap();
        }
        self.diagnostic = None;
        self.data.lock().unwrap().disconnects += 1;
        Ok(())
    }
}
fn problem(error: &stagemaster_device_channel::Error) -> Problem {
    Problem::new(
        if matches!(error, stagemaster_device_channel::Error::Denied) {
            C::Runtime
        } else {
            C::Lost
        },
    )
    .detail(error.to_string())
}
pub fn status(host: &Host) -> Snapshot {
    host.request(Request::Status).unwrap()
}
pub async fn settle() {
    for _ in 0..50 {
        tokio::task::yield_now().await;
    }
}
pub async fn connect(host: &Host, access: Access) -> u32 {
    if status(host).candidates.is_empty() {
        host.request(Request::Scan {
            epoch: status(host).epoch,
        })
        .unwrap();
        settle().await;
        tokio::time::advance(Duration::from_secs(8)).await;
        settle().await;
    }
    host.connect_runtime(status(host).epoch, "runtime".into(), access)
        .unwrap();
    wait_for_settled(host).await;
    status(host).epoch
}
pub async fn disconnect(host: &Host) {
    host.request(Request::Cancel {
        epoch: status(host).epoch,
    })
    .unwrap();
    wait_for_settled(host).await;
    assert_eq!(status(host).phase, Phase::Idle);
}
pub async fn command(host: &Host, operation: stagemaster_runtime_protocol::Operation) -> Response {
    let epoch = status(host).epoch;
    let snapshot = host.runtime_snapshot(epoch).unwrap();
    let expected_revision = snapshot.last_response.map_or(0, |r| r.observed.revision);
    host.exchange_runtime(
        epoch,
        RuntimeIntent {
            operation,
            expected_revision,
        },
    )
    .await
    .unwrap()
}

pub async fn wait_for_settled(host: &Host) {
    for _ in 0..1000 {
        settle().await;
        if !matches!(status(host).phase, Phase::Connecting | Phase::Stopping) {
            return;
        }
        tokio::time::advance(Duration::from_millis(5)).await;
    }
    panic!("connection did not settle: {:?}", status(host).phase);
}
