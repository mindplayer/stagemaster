use stagemaster_device_host::{
    Candidate, InstallationPeer, Phase, Problem, ProblemCode as C, Request, Service, Snapshot,
    Transport,
};
use stagemaster_device_info::{Description, Firmware, Limits, capability as cap};
use stagemaster_device_link::Session;
use stagemaster_install::Installer;
use stagemaster_install_store::FileStore;
use stagemaster_transfer::{
    Assembler, AuthorizedLink, Command, Frame, Request as WireRequest, Response, Service as Server,
};
use std::{
    collections::VecDeque,
    future::pending,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::{Instant, sleep};

#[derive(Clone, Copy, Default)]
pub enum Fault {
    #[default]
    None,
    NoReply,
    Partial,
    WrongBoot,
    WrongSession,
    WrongId,
    WrongCommand,
    Extra,
    Empty,
    FailNotify,
    FailWrite,
    DropCommit,
    EarlyReply,
}
pub struct State {
    pub server: Server<FileStore>,
    pub directory: Arc<tempfile::TempDir>,
    pub grant: Option<InstallationPeer>,
    pub description: Option<Description>,
    pub fault: Fault,
    pub fragment_delay: Duration,
    pub notification_delay: Duration,
    pub diagnostic_delay: Duration,
    pub fragments: usize,
    pub connects: u8,
    pub disconnects: usize,
    pub beats: Vec<Instant>,
    pub notifications: VecDeque<Vec<u8>>,
    discovered: bool,
    diagnostic: Option<Session>,
    reply: Vec<u8>,
    assembler: Assembler,
    available: Instant,
    started: Instant,
}
#[derive(Clone)]
pub struct Fake(pub Arc<Mutex<State>>);
pub type Host = Service<Fake>;

pub fn setup(payload: u16) -> (Host, Arc<Mutex<State>>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    std::fs::create_dir_all(&root).unwrap();
    let directory = tempfile::tempdir_in(root).unwrap();
    let server = Server::new(
        Installer::open(FileStore::open(directory.path()).unwrap(), [2; 16])
            .unwrap()
            .0,
    )
    .unwrap();
    let description = Description {
        device: [1; 16],
        boot: [2; 16],
        session: 45,
        model: 1,
        firmware: Firmware::default(),
        capabilities: cap::DIAGNOSTICS | cap::CATALOG | cap::INSTALLATION,
        authentication: 1,
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
    };
    let state = Arc::new(Mutex::new(State {
        server,
        directory: Arc::new(directory),
        description: Some(description),
        grant: Some(InstallationPeer {
            device: [1; 16],
            boot: [2; 16],
            session: [1; 16],
            authentication: 1,
            fragment_bytes: payload,
            message_bytes: 1280,
        }),
        fault: Fault::None,
        fragment_delay: Duration::ZERO,
        notification_delay: Duration::ZERO,
        diagnostic_delay: Duration::ZERO,
        fragments: 0,
        connects: 0,
        disconnects: 0,
        beats: vec![],
        notifications: VecDeque::new(),
        discovered: false,
        diagnostic: None,
        reply: vec![],
        assembler: Assembler::new(),
        available: Instant::now(),
        started: Instant::now(),
    }));
    (Service::new(Fake(state.clone())), state)
}
impl Transport for Fake {
    async fn start_scan(&mut self) -> Result<(), Problem> {
        self.0.lock().unwrap().discovered = false;
        Ok(())
    }
    async fn discover(&mut self) -> Result<Candidate, Problem> {
        let found = {
            let mut s = self.0.lock().unwrap();
            std::mem::replace(&mut s.discovered, true)
        };
        if found {
            pending().await
        } else {
            Ok(Candidate {
                id: "test".into(),
                name: "软件安装夹具".into(),
                rssi: None,
            })
        }
    }
    async fn stop_scan(&mut self) -> Result<(), Problem> {
        Ok(())
    }
    async fn connect(&mut self, _: &str) -> Result<(), Problem> {
        let mut s = self.0.lock().unwrap();
        s.connects += 1;
        let session = [s.connects; 16];
        s.diagnostic = Some(Session::new(45, 0).unwrap());
        s.started = Instant::now();
        s.assembler = Assembler::new();
        s.notifications.clear();
        s.server.detach();
        if let Some(peer) = s.grant.as_mut() {
            peer.session = session;
            s.server
                .attach(AuthorizedLink {
                    principal: [9; 16],
                    session,
                })
                .unwrap();
        }
        Ok(())
    }
    async fn write(&mut self, bytes: &[u8; 20]) -> Result<(), Problem> {
        let mut s = self.0.lock().unwrap();
        let now = u64::try_from(s.started.elapsed().as_millis()).unwrap();
        s.reply = s
            .diagnostic
            .as_mut()
            .unwrap()
            .receive(bytes, now)
            .encode()
            .to_vec();
        s.beats.push(Instant::now());
        Ok(())
    }
    async fn reply(&mut self) -> Result<Vec<u8>, Problem> {
        Ok(self.0.lock().unwrap().reply.clone())
    }
    async fn diagnostics(&mut self) -> Result<Vec<u8>, Problem> {
        let delay = self.0.lock().unwrap().diagnostic_delay;
        sleep(delay).await;
        let mut bytes = vec![0; 20];
        bytes[0] = 1;
        bytes[1] = 3;
        bytes[16..20].copy_from_slice(&131_072u32.to_le_bytes());
        Ok(bytes)
    }
    async fn description(&mut self) -> Result<Option<Vec<u8>>, Problem> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .description
            .map(|d| d.encode().unwrap().to_vec()))
    }
    fn installation_peer(&self) -> Option<InstallationPeer> {
        self.0.lock().unwrap().grant
    }
    async fn write_installation(&mut self, bytes: &[u8]) -> Result<(), Problem> {
        let delay = self.0.lock().unwrap().fragment_delay;
        sleep(delay).await;
        let mut s = self.0.lock().unwrap();
        s.fragments += 1;
        if matches!(s.fault, Fault::FailWrite) {
            return Err(Problem::new(C::Lost));
        }
        if s.assembler.push(bytes).unwrap() {
            let frame = s.assembler.take().unwrap();
            let command = WireRequest::decode(frame.bytes()).unwrap().action.command();
            let response = s.server.process(frame.bytes()).unwrap();
            let mut response = Response::decode(response.bytes()).unwrap();
            match s.fault {
                Fault::WrongBoot => response.state.boot = [3; 16],
                Fault::WrongSession => response.link = [8; 16],
                Fault::WrongId => response.id += 256,
                Fault::WrongCommand => response.command = Command::Reconcile,
                _ => {}
            }
            if matches!(s.fault, Fault::NoReply)
                || matches!(s.fault, Fault::DropCommit) && command == Command::Commit
            {
                return Ok(());
            }
            let response = response.encode().unwrap();
            let payload = usize::from(s.grant.unwrap().fragment_bytes);
            s.notifications = response
                .bytes()
                .chunks(payload)
                .map(<[u8]>::to_vec)
                .collect();
            match s.fault {
                Fault::Partial => {
                    s.notifications = VecDeque::from([response.bytes()[..8].to_vec()]);
                }
                Fault::Extra => s.notifications.push_back(vec![1]),
                Fault::Empty => s.notifications = VecDeque::from([vec![]]),
                _ => {}
            }
            s.available = Instant::now() + s.notification_delay;
        } else if matches!(s.fault, Fault::EarlyReply) {
            s.notifications.push_back(vec![9]);
        }
        Ok(())
    }
    fn try_installation_notification(&mut self) -> Result<Option<Vec<u8>>, Problem> {
        let mut s = self.0.lock().unwrap();
        if matches!(s.fault, Fault::FailNotify) && s.fragments > 0 {
            return Err(Problem::new(C::Lost));
        }
        if Instant::now() < s.available {
            return Ok(None);
        }
        Ok(s.notifications.pop_front())
    }
    async fn disconnect(&mut self) -> Result<(), Problem> {
        let mut s = self.0.lock().unwrap();
        s.disconnects += 1;
        s.diagnostic = None;
        s.notifications.clear();
        s.assembler = Assembler::new();
        s.server.detach();
        Ok(())
    }
}
pub fn status(host: &Host) -> Snapshot {
    host.request(Request::Status).unwrap()
}
pub async fn settle() {
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
}
pub async fn connect(host: &Host) -> Snapshot {
    let initial = status(host);
    if initial.candidates.is_empty() {
        host.request(Request::Scan {
            epoch: initial.epoch,
        })
        .unwrap();
        settle().await;
        tokio::time::advance(Duration::from_secs(8)).await;
        settle().await;
    }
    host.request(Request::Connect {
        epoch: status(host).epoch,
        id: "test".into(),
    })
    .unwrap();
    settle().await;
    status(host)
}
pub async fn disconnect(host: &Host) {
    host.request(Request::Cancel {
        epoch: status(host).epoch,
    })
    .unwrap();
    settle().await;
    assert_eq!(status(host).phase, Phase::Idle);
}
pub fn query(host: &Host) -> Frame {
    let peer = host.installation_peer(status(host).epoch).unwrap().unwrap();
    WireRequest {
        link: peer.session,
        id: 1,
        action: stagemaster_transfer::Action::Status,
    }
    .encode()
    .unwrap()
}
