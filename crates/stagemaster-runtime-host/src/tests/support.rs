use crate::{Action, Client, Configuration, Host, Observer, Snapshot};
use stagemaster_install::{Commit, Identity, Installer, Record, Slot, Storage};
use stagemaster_install_store::{FileSnapshot, FileStore};
use stagemaster_package::{Archive, MAX_LOADER_BYTES, ReadAt};
use stagemaster_project::{Document, PackageSelection};
use stagemaster_runtime::{
    self as runtime, Grant, Origin, PlaybackPolicy, ProgramKey, Request, Runtime,
};
use std::{
    io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

pub const TTL: Duration = Duration::from_secs(2);
pub const WAIT: Duration = Duration::from_secs(4);

#[derive(Default)]
pub struct Reads {
    pub count: AtomicUsize,
    pub inaccessible: AtomicBool,
}
pub struct Reader {
    inner: FileSnapshot,
    reads: Arc<Reads>,
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
        self.reads.count.fetch_add(1, Ordering::SeqCst);
        if self.reads.inaccessible.load(Ordering::SeqCst) {
            return Err(stagemaster_package::Error::Read);
        }
        self.inner.read_exact(offset, target)
    }
}
struct Store {
    inner: FileStore,
    reads: Arc<Reads>,
}
impl Storage for Store {
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
        Ok(Reader {
            inner: self.inner.snapshot(slot)?,
            reads: self.reads.clone(),
        })
    }
}
pub struct Fixture {
    _dir: tempfile::TempDir,
    installer: Installer<Store>,
    pub reads: Arc<Reads>,
    pub bytes: Vec<u8>,
}
pub struct SoftwareAcceptance;
impl PlaybackPolicy for SoftwareAcceptance {
    fn authorize(&mut self, _: runtime::Permission) -> Result<(), runtime::Denial> {
        Ok(())
    }
}
impl Fixture {
    pub fn new() -> Self {
        let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        value["entryPoints"] = serde_json::json!([]);
        value["lighting"]["sequences"][0]["repeat"] = "loop".into();
        let document = Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
        let selection = PackageSelection::Sequence {
            id: document.view().sequences[0].id.clone(),
        };
        let bytes = document.build_package(&[selection]).unwrap().bytes;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
        std::fs::create_dir_all(&root).unwrap();
        let dir = tempfile::tempdir_in(root).unwrap();
        let reads = Arc::new(Reads::default());
        let store = Store {
            inner: FileStore::open(dir.path()).unwrap(),
            reads: reads.clone(),
        };
        let mut installer = Installer::open(store, [1; 16]).unwrap().0;
        let transaction = installer.transaction(1);
        let identity = Identity::from_archive(&Archive::open(bytes.as_slice()).unwrap());
        installer.begin(transaction, identity).unwrap();
        for (i, chunk) in bytes.chunks(1024).enumerate() {
            installer.write(transaction, i * 1024, chunk).unwrap();
        }
        installer.verify(transaction).unwrap();
        installer.commit(transaction).unwrap();
        Self {
            _dir: dir,
            installer,
            reads,
            bytes,
        }
    }
    pub fn prepared<P: PlaybackPolicy>(&self, policy: P) -> Runtime<Reader, P> {
        self.reads.inaccessible.store(false, Ordering::SeqCst);
        let mut runtime = Runtime::new([2; 16], 0, MAX_LOADER_BYTES, policy).unwrap();
        let permit = runtime
            .confirm_quiescent(runtime.quiescence_request().unwrap(), 0)
            .unwrap();
        runtime
            .finish_maintenance(permit, 0, || self.installer.snapshot().map(Some))
            .unwrap();
        let entry = &runtime.catalog()[0];
        let key = ProgramKey {
            kind: entry.kind,
            id: entry.id,
        };
        let lease = runtime.acquire(grant(9, 60_000), false, 0).unwrap();
        for (serial, action) in [
            (1, runtime::Action::Select(key)),
            (2, runtime::Action::Load),
        ] {
            runtime
                .submit(
                    Request {
                        lease,
                        serial,
                        expected_revision: runtime.state().revision,
                        action,
                    },
                    0,
                )
                .unwrap()
                .result
                .unwrap();
        }
        runtime.release(lease, 0).unwrap();
        self.reads.count.store(0, Ordering::SeqCst);
        self.reads.inaccessible.store(true, Ordering::SeqCst);
        runtime
    }
    pub fn host(&self) -> (Host, [u8; 16]) {
        let runtime = self.prepared(SoftwareAcceptance);
        let step = runtime.steps()[0].id;
        (
            Host::start(
                runtime,
                Configuration {
                    period: Duration::from_millis(5),
                },
            )
            .unwrap(),
            step,
        )
    }
}
pub fn grant(id: u8, duration_ms: u64) -> Grant {
    Grant {
        principal: [id; 16],
        origin: Origin::Remote,
        duration_ms,
    }
}
pub fn until(observer: &Observer, test: impl Fn(Snapshot) -> bool) -> Snapshot {
    let deadline = Instant::now() + WAIT;
    loop {
        if let Ok(view) = observer.read()
            && let Some(snapshot) = view.snapshot
            && test(snapshot)
        {
            return snapshot;
        }
        assert!(Instant::now() < deadline, "宿主状态未在期限内满足验收条件");
        std::thread::sleep(Duration::from_millis(2));
    }
}
pub fn connect(host: &Host, id: u8, duration_ms: u64, takeover: bool) -> Client {
    let client = host
        .connect(grant(id, duration_ms), takeover, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    until(&host.observer(), |s| {
        s.state.owner.is_some_and(|o| o.principal == [id; 16])
    });
    client
}
pub fn start(client: &Client, step: [u8; 16]) -> runtime::Receipt {
    let state = client.acquired_state();
    client
        .submit(1, state.revision, Action::Start { step }, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap()
}
