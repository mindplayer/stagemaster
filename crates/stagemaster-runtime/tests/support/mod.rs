use stagemaster_install::{Commit, Identity, Installed, Installer, Record, Slot, Storage};
use stagemaster_install_store::{FileSnapshot, FileStore};
use stagemaster_package::{Archive, ReadAt};
use stagemaster_project::{Document, PackageSelection};
use stagemaster_runtime::*;
use std::{
    cell::{Cell, RefCell},
    io,
    path::PathBuf,
    rc::Rc,
};

#[derive(Default)]
pub struct Metrics {
    pub reads: Cell<usize>,
    pub readers: Cell<usize>,
    pub fail: Cell<bool>,
}
pub struct Reader {
    inner: FileSnapshot,
    metrics: Rc<Metrics>,
}
impl Drop for Reader {
    fn drop(&mut self) {
        self.metrics.readers.set(self.metrics.readers.get() - 1);
    }
}
impl ReadAt for Reader {
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn read_exact(
        &self,
        offset: usize,
        target: &mut [u8],
    ) -> Result<(), stagemaster_package::Error> {
        self.metrics.reads.set(self.metrics.reads.get() + 1);
        if self.metrics.fail.get() {
            return Err(stagemaster_package::Error::Read);
        }
        self.inner.read_exact(offset, target)
    }
}
pub struct ObservedStore {
    pub inner: FileStore,
    pub metrics: Rc<Metrics>,
}
impl Storage for ObservedStore {
    type Error = io::Error;
    type Snapshot = Reader;
    fn capacity(&self, slot: Slot) -> usize {
        self.inner.capacity(slot)
    }
    fn record(&self, slot: Slot) -> io::Result<Record> {
        self.inner.record(slot)
    }
    fn slot_len(&self, slot: Slot) -> io::Result<usize> {
        self.inner.slot_len(slot)
    }
    fn read(&self, slot: Slot, offset: usize, bytes: &mut [u8]) -> io::Result<()> {
        self.inner.read(slot, offset, bytes)
    }
    fn prepare(&mut self, slot: Slot, bytes: usize) -> io::Result<()> {
        self.inner.prepare(slot, bytes)
    }
    fn write(&mut self, slot: Slot, offset: usize, bytes: &[u8]) -> io::Result<()> {
        self.inner.write(slot, offset, bytes)
    }
    fn sync_payload(&mut self, slot: Slot) -> io::Result<()> {
        self.inner.sync_payload(slot)
    }
    fn commit_record(&mut self, commit: Commit) -> io::Result<()> {
        self.inner.commit_record(commit)
    }
    fn settle(&mut self) -> io::Result<()> {
        self.inner.settle()
    }
    fn release(&mut self) {
        self.inner.release();
    }
    fn snapshot(&self, slot: Slot) -> io::Result<Reader> {
        let inner = self.inner.snapshot(slot)?;
        self.metrics.readers.set(self.metrics.readers.get() + 1);
        Ok(Reader {
            inner,
            metrics: self.metrics.clone(),
        })
    }
}
#[derive(Default)]
pub struct Policy {
    pub permissions: Vec<Permission>,
    pub deny: Option<Denial>,
}
#[derive(Clone, Default)]
pub struct PolicyRef(pub Rc<RefCell<Policy>>);
impl PlaybackPolicy for PolicyRef {
    fn authorize(&mut self, permission: Permission) -> Result<(), Denial> {
        let mut p = self.0.borrow_mut();
        p.permissions.push(permission);
        p.deny.map_or(Ok(()), Err)
    }
}
pub type Device = Runtime<Reader, PolicyRef>;
pub fn package(repeat: bool) -> Vec<u8> {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    json["lighting"]["profiles"][0]["attributes"][0]["default"]["value"] = 12000.into();
    json["lighting"]["sequences"][0]["repeat"] = if repeat { "loop" } else { "once" }.into();
    let doc = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    let view = doc.view();
    let selections: Vec<_> = view
        .scenes
        .iter()
        .map(|s| PackageSelection::Scene { id: s.id.clone() })
        .chain(
            view.sequences
                .iter()
                .map(|s| PackageSelection::Sequence { id: s.id.clone() }),
        )
        .collect();
    doc.build_package(&selections).unwrap().bytes
}
pub fn install(installer: &mut Installer<ObservedStore>, bytes: &[u8], counter: u64) {
    let identity = Identity::from_archive(&Archive::open(bytes).unwrap());
    let t = installer.transaction(counter);
    installer.begin(t, identity).unwrap();
    for (i, block) in bytes.chunks(1024).enumerate() {
        installer.write(t, i * 1024, block).unwrap();
    }
    installer.verify(t).unwrap();
    installer.commit(t).unwrap();
}
pub struct Fixture {
    pub _dir: tempfile::TempDir,
    pub installer: Installer<ObservedStore>,
    pub metrics: Rc<Metrics>,
    pub bytes: Vec<u8>,
    pub policy: PolicyRef,
}
impl Fixture {
    pub fn new(repeat: bool) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
        std::fs::create_dir_all(&root).unwrap();
        let dir = tempfile::tempdir_in(root).unwrap();
        let metrics = Rc::new(Metrics::default());
        let store = ObservedStore {
            inner: FileStore::open(dir.path()).unwrap(),
            metrics: metrics.clone(),
        };
        let mut installer = Installer::open(store, [1; 16]).unwrap().0;
        let bytes = package(repeat);
        install(&mut installer, &bytes, 1);
        Self {
            _dir: dir,
            installer,
            metrics,
            bytes,
            policy: PolicyRef::default(),
        }
    }
    pub fn runtime(&self, budget: usize) -> Device {
        let mut runtime = Runtime::new([2; 16], 0, budget, self.policy.clone()).unwrap();
        let permit = runtime
            .confirm_quiescent(runtime.quiescence_request().unwrap(), 0)
            .unwrap();
        runtime
            .finish_maintenance(permit, 0, || self.installer.snapshot().map(Some))
            .unwrap();
        runtime
    }
    pub fn snapshot(&self) -> Installed<Reader> {
        self.installer.snapshot().unwrap()
    }
}
pub fn grant(origin: Origin, duration_ms: u64) -> Grant {
    Grant {
        principal: [3; 16],
        origin,
        duration_ms,
    }
}
pub fn acquire(runtime: &mut Device, origin: Origin, now: u64) -> Lease {
    runtime.acquire(grant(origin, 60000), false, now).unwrap()
}
pub fn request(runtime: &Device, lease: Lease, action: Action) -> Request {
    Request {
        lease,
        serial: runtime.state().owner.unwrap().serial + 1,
        expected_revision: runtime.state().revision,
        action,
    }
}
pub fn submit(runtime: &mut Device, lease: Lease, action: Action, now: u64) -> Receipt {
    runtime
        .submit(request(runtime, lease, action), now)
        .unwrap()
}
pub fn apply(runtime: &mut Device, lease: Lease, action: Action, now: u64) {
    assert_eq!(submit(runtime, lease, action, now).result, Ok(()));
}
pub fn load(runtime: &mut Device, lease: Lease, index: usize, now: u64) -> ProgramKey {
    let e = &runtime.catalog()[index];
    let key = ProgramKey {
        kind: e.kind,
        id: e.id,
    };
    apply(runtime, lease, Action::Select(key), now);
    apply(runtime, lease, Action::Load, now);
    key
}
pub fn start(runtime: &mut Device, lease: Lease, now: u64) {
    apply(
        runtime,
        lease,
        Action::Start {
            step: runtime.steps()[0].id,
        },
        now,
    );
}
