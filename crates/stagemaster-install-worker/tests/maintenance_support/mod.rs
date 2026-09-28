mod store;
use stagemaster_install::{Installer, Storage};
use stagemaster_install_store::FileStore;
use stagemaster_install_worker::{Command, Epoch, ManagedWorker, Reply};
use stagemaster_project::{Document, PackageSelection};
use stagemaster_runtime::{
    Action, Denial, Grant, Lease, Origin, Permission, PlaybackPolicy, Request,
};
use stagemaster_transfer::{AuthorizedLink, Frame, Upload};
use std::{path::Path, rc::Rc};
pub use store::{Metrics, Store};

pub struct Allowed;
impl PlaybackPolicy for Allowed {
    fn authorize(&mut self, _permission: Permission) -> Result<(), Denial> {
        Ok(())
    }
}
pub type Device = ManagedWorker<Store, Allowed>;
pub fn epoch(n: u32) -> Epoch {
    Epoch::new(n).unwrap()
}
pub fn open_command(n: u8) -> Command {
    Command::Open {
        epoch: epoch(u32::from(n)),
        link: AuthorizedLink {
            principal: [9; 16],
            session: [n; 16],
        },
    }
}
pub fn open(worker: &mut Device, n: u8, now: u64) {
    assert!(matches!(
        worker
            .process(open_command(n), now, || Some(epoch(u32::from(n))))
            .result,
        Ok(Reply::Opened)
    ));
}
pub fn frame<S: Storage, P: PlaybackPolicy>(
    worker: &mut ManagedWorker<S, P>,
    n: u8,
    request: Frame,
    now: u64,
) -> Frame {
    match worker
        .process(
            Command::Frame {
                epoch: epoch(u32::from(n)),
                frame: request,
            },
            now,
            || Some(epoch(u32::from(n))),
        )
        .result
        .unwrap()
    {
        Reply::Frame(value) => value,
        Reply::Opened => panic!("wrong reply"),
    }
}
pub fn transfer(worker: &mut Device, upload: &mut Upload<&[u8]>, n: u8, now: u64) {
    while let Some(request) = upload.outbound().unwrap().cloned() {
        let response = frame(worker, n, request, now);
        upload.accept(response.bytes()).unwrap();
    }
}
pub fn fixture() -> (tempfile::TempDir, Device, Rc<Metrics>, Vec<u8>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    let dir = tempfile::tempdir_in(root).unwrap();
    let metrics = Rc::new(Metrics::default());
    let store = Store {
        inner: FileStore::open(dir.path()).unwrap(),
        metrics: metrics.clone(),
    };
    let worker = ManagedWorker::new(
        Installer::open(store, [7; 16]).unwrap().0,
        0,
        64 * 1024,
        Allowed,
    )
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    value["entryPoints"] = serde_json::json!([]);
    let document = Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    let selected = document
        .view()
        .scenes
        .iter()
        .map(|s| PackageSelection::Scene { id: s.id.clone() })
        .chain(
            document
                .view()
                .sequences
                .iter()
                .map(|s| PackageSelection::Sequence { id: s.id.clone() }),
        )
        .collect::<Vec<_>>();
    let bytes = document.build_package(&selected).unwrap().bytes;
    (dir, worker, metrics, bytes)
}
pub fn install(worker: &mut Device, bytes: &[u8]) {
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), 0)
        .unwrap();
    open(worker, 1, 0);
    let mut upload = Upload::new(bytes).unwrap();
    upload.connect([1; 16]).unwrap();
    transfer(worker, &mut upload, 1, 0);
}
pub fn acquire(worker: &mut Device, now: u64) -> Lease {
    worker
        .acquire(
            Grant {
                principal: [9; 16],
                origin: Origin::Panel,
                duration_ms: 60_000,
            },
            false,
            now,
        )
        .unwrap()
}
pub fn apply(
    worker: &mut Device,
    lease: Lease,
    action: Action,
    now: u64,
) -> Result<(), stagemaster_runtime::Code> {
    let request = Request {
        lease,
        serial: worker.state().owner.unwrap().serial + 1,
        expected_revision: worker.state().revision,
        action,
    };
    worker.submit(request, now).unwrap().result
}
