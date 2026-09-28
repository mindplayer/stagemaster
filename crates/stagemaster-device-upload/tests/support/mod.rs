use stagemaster_device_host::{DeviceLimits, InstallationPeer};
use stagemaster_device_upload::{Connection, Prepared, Service, Snapshot, Target};
use stagemaster_install::Installer;
use stagemaster_install_store::FileStore;
use stagemaster_project::{Document, PackageSelection};
use stagemaster_transfer::{
    AuthorizedLink, Command, Frame, RemoteError, Request, Response, Service as Server,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Copy, Default)]
pub enum Fault {
    #[default]
    None,
    LostCommit,
    RejectBegin,
    Hang,
    Panic,
}
pub struct State {
    pub target: Target,
    pub allowed: bool,
    pub commands: Vec<Command>,
    pub server: Server<FileStore>,
    pub directory: tempfile::TempDir,
    pub fault: Fault,
    pub delay: Duration,
    pub inflight: usize,
}
pub struct Link(pub Arc<Mutex<State>>);
struct Flight(Arc<Mutex<State>>);
impl Drop for Flight {
    fn drop(&mut self) {
        if let Ok(mut state) = self.0.lock() {
            state.inflight -= 1;
        }
    }
}
impl Connection for Link {
    fn target(&self, epoch: u32) -> Result<Target, String> {
        let s = self.0.lock().unwrap();
        if !s.allowed || s.target.epoch != epoch {
            return Err("测试连接无有效安装权限".into());
        }
        Ok(s.target.clone())
    }
    async fn exchange(&self, epoch: u32, frame: Frame) -> Result<Frame, String> {
        self.target(epoch)?;
        let command = Request::decode(frame.bytes()).unwrap().action.command();
        let (delay, fault) = {
            let mut s = self.0.lock().unwrap();
            s.inflight += 1;
            s.commands.push(command);
            (s.delay, s.fault)
        };
        let _flight = Flight(self.0.clone());
        if matches!(fault, Fault::Panic) {
            panic!("注入适配器异常");
        }
        if matches!(fault, Fault::Hang) {
            std::future::pending::<()>().await;
        }
        tokio::time::sleep(delay).await;
        let mut s = self.0.lock().unwrap();
        let response = s.server.process(frame.bytes()).map_err(|e| e.to_string())?;
        if matches!(fault, Fault::LostCommit) && command == Command::Commit {
            return Err("提交回执丢失".into());
        }
        if matches!(fault, Fault::RejectBegin) && command == Command::Begin {
            s.fault = Fault::None;
            let mut response = Response::decode(response.bytes()).unwrap();
            response.result = Err(RemoteError::Busy);
            return Ok(response.encode().unwrap());
        }
        Ok(response)
    }
}
pub const DEVICE: &str = "01010101010101010101010101010101";
pub fn setup() -> (Service<Link>, Arc<Mutex<State>>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    std::fs::create_dir_all(&root).unwrap();
    let directory = tempfile::tempdir_in(root).unwrap();
    let mut server = Server::new(
        Installer::open(FileStore::open(directory.path()).unwrap(), [2; 16])
            .unwrap()
            .0,
    )
    .unwrap();
    server
        .attach(AuthorizedLink {
            principal: [9; 16],
            session: [3; 16],
        })
        .unwrap();
    let target = Target {
        epoch: 1,
        name: "安装测试设备".into(),
        peer: InstallationPeer {
            device: [1; 16],
            boot: [2; 16],
            session: [3; 16],
            authentication: 1,
            fragment_bytes: 244,
            message_bytes: 1280,
        },
        limits: DeviceLimits {
            package_version: 1,
            transfer_version: 1,
            package_bytes: 2 * 1024 * 1024,
            programs: 64,
            universes: 0,
            message_bytes: 1280,
            chunk_bytes: 1024,
            slot_bytes: 2 * 1024 * 1024,
            loader_bytes: 0,
            frame_ms: 0,
        },
    };
    let s = Arc::new(Mutex::new(State {
        target,
        allowed: true,
        commands: vec![],
        server,
        directory,
        fault: Fault::None,
        delay: Duration::from_millis(1),
        inflight: 0,
    }));
    (Service::new(Arc::new(Link(s.clone()))), s)
}
pub fn package() -> Arc<[u8]> {
    package_at(1000)
}
pub fn package_at(level: u16) -> Arc<[u8]> {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    json["lighting"]["scenes"][0]["assignments"][0]["source"]["value"]["value"] = level.into();
    let first = json["lighting"]["scenes"][0].clone();
    for n in 0..20 {
        let mut scene = first.clone();
        scene["id"] = format!("00000000-0000-4000-8000-{:012}", n + 100).into();
        json["lighting"]["scenes"]
            .as_array_mut()
            .unwrap()
            .push(scene);
    }
    let doc = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    let selected: Vec<_> = doc
        .view()
        .scenes
        .iter()
        .map(|s| PackageSelection::Scene { id: s.id.clone() })
        .collect();
    let bytes = doc.build_package(&selected).unwrap().bytes;
    assert!(bytes.len() > 2048);
    bytes.into()
}
pub fn prepared() -> Prepared {
    Prepared::new(package()).unwrap()
}
pub fn reconnect(s: &Mutex<State>) {
    let mut s = s.lock().unwrap();
    s.target.epoch += 1;
    s.target.peer.session[0] += 1;
    let session = s.target.peer.session;
    s.allowed = true;
    s.fault = Fault::None;
    s.server.detach();
    s.server
        .attach(AuthorizedLink {
            principal: [9; 16],
            session,
        })
        .unwrap();
}
pub async fn finished(host: &Service<Link>) -> Snapshot {
    for _ in 0..1000 {
        let view = host.snapshot().unwrap();
        if !view.task.as_ref().unwrap().running {
            return view;
        }
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    panic!("安装任务未结束")
}
pub async fn settle() {
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
}
