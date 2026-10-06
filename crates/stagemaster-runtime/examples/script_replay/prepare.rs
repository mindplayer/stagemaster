use super::schedule::Schedule;
use super::store::{Reader, Store};
use stagemaster_install::{Identity, Installer};
use stagemaster_install_store::FileStore;
use stagemaster_package::{Archive, Kind, MAX_LOADER_BYTES, MAX_PACKAGE_BYTES};
use stagemaster_playback::Player;
use stagemaster_project::{CompiledOutput, Document, PackageSelection};
use stagemaster_runtime::{
    Action, Code, Denial, Grant, Lease, Origin, Permission, PlaybackPolicy, ProgramKey, Receipt,
    Request, Runtime, Status,
};
use std::{cell::Cell, error::Error, path::Path, rc::Rc};

pub struct SoftwareAcceptance;
impl PlaybackPolicy for SoftwareAcceptance {
    fn authorize(&mut self, _: Permission) -> Result<(), Denial> {
        Ok(())
    }
}
pub type Core = Runtime<Reader, SoftwareAcceptance>;
pub struct Prepared {
    pub runtime: Core,
    pub reference: Player,
    pub output: CompiledOutput,
    pub step_ids: Vec<[u8; 16]>,
    pub schedule: Schedule,
    pub lease: Lease,
    pub name: String,
    pub reads: Rc<Cell<usize>>,
    pub source_digest: String,
    // Dropped last, after the Runtime releases its immutable file reader.
    pub _directory: tempfile::TempDir,
}
pub fn request(runtime: &Core, lease: Lease, action: Action) -> Request {
    let state = runtime.state();
    Request {
        lease,
        serial: state.owner.unwrap().serial + 1,
        expected_revision: state.revision,
        action,
    }
}
pub fn apply(runtime: &mut Core, lease: Lease, action: Action, now: u64) -> Result<Receipt, Code> {
    let receipt = runtime.submit(request(runtime, lease, action), now)?;
    receipt.result?;
    Ok(receipt)
}
pub fn grant() -> Grant {
    Grant {
        principal: [3; 16],
        origin: Origin::Panel,
        duration_ms: 60_000,
    }
}

pub fn prepare(document: &Document, bytes: &[u8], id: &str) -> Result<Prepared, Box<dyn Error>> {
    if bytes.len() > MAX_PACKAGE_BYTES {
        return Err("播放包超出容量".into());
    }
    let compiled = document.compile_sequence(id)?;
    let schedule = Schedule::from(&compiled.plan)?;
    let step_ids = compiled
        .steps
        .iter()
        .map(|s| uuid::Uuid::parse_str(&s.id).map(|id| *id.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;
    let key = ProgramKey {
        kind: Kind::Sequence,
        id: *uuid::Uuid::parse_str(id)?.as_bytes(),
    };
    let archive = Archive::open(bytes)?;
    let selections = archive
        .entries()
        .iter()
        .map(|e| {
            let id = uuid::Uuid::from_bytes(e.id).to_string();
            match e.kind {
                Kind::Scene => PackageSelection::Scene { id },
                Kind::Sequence => PackageSelection::Sequence { id },
            }
        })
        .collect::<Vec<_>>();
    let rebuilt = document.build_package(&selections).map_err(|issues| {
        serde_json::to_string(&issues).unwrap_or_else(|_| "源工程无法生成原选择".into())
    })?;
    if rebuilt.bytes != bytes {
        return Err("源工程快照与原播放包不一致，未开始运行".into());
    }
    let index = archive
        .entries()
        .iter()
        .position(|e| e.kind == key.kind && e.id == key.id)
        .ok_or("包内没有所选列表")?;
    let program = archive.load(bytes, index)?;
    if program.labels.len() != compiled.steps.len()
        || program
            .labels
            .iter()
            .zip(&compiled.steps)
            .zip(&step_ids)
            .any(|((a, b), id)| a.id != *id || a.name != b.name || a.number != b.number)
    {
        return Err("源工程与包内步骤标签不一致".into());
    }
    drop(program);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    let directory = tempfile::tempdir_in(root)?;
    let reads = Rc::new(Cell::new(0));
    let blocked = Rc::new(Cell::new(false));
    let store = Store {
        inner: FileStore::open(directory.path())?,
        reads: reads.clone(),
        blocked: blocked.clone(),
    };
    let mut installer = Installer::open(store, [1; 16])?.0;
    let transaction = installer.transaction(1);
    installer.begin(transaction, Identity::from_archive(&archive))?;
    for (index, chunk) in bytes.chunks(1024).enumerate() {
        installer.write(transaction, index * 1024, chunk)?;
    }
    installer.verify(transaction)?;
    installer.commit(transaction)?;
    let mut runtime = Runtime::new([2; 16], 0, MAX_LOADER_BYTES, SoftwareAcceptance)?;
    // No driver exists here, so there are no queued physical frames to drain.
    let permit = runtime.confirm_quiescent(runtime.quiescence_request().ok_or(Code::Mode)?, 0)?;
    runtime.finish_maintenance(permit, 0, || installer.snapshot().map(Some))?;
    let lease = runtime.acquire(grant(), false, 0)?;
    apply(&mut runtime, lease, Action::Select(key), 0)?;
    apply(&mut runtime, lease, Action::Load, 0)?;
    if runtime.state().instance.is_some() || runtime.state().status != Some(Status::Idle) {
        return Err("安装／载入隐式开始了执行".into());
    }
    reads.set(0);
    blocked.set(true);
    Ok(Prepared {
        _directory: directory,
        runtime,
        reference: Player::new(compiled.plan, 0),
        output: compiled.output,
        step_ids,
        schedule,
        lease,
        name: compiled.name,
        reads,
        source_digest: rebuilt.report.source_digest,
    })
}
