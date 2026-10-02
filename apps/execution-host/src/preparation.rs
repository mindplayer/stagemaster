use crate::{application::Application, directory::Directory};
use serde_json::{Value, json};
use stagemaster_install::{Identity, Installer};
use stagemaster_install_store::FileStore;
use stagemaster_package::{Archive, MAX_LOADER_BYTES};
use stagemaster_project::PackageSelection;
use stagemaster_project_store::DiskFile;
use stagemaster_runtime::{
    self as runtime, Grant, Origin, PlaybackPolicy, ProgramKey, Request, Runtime,
};
use stagemaster_runtime_host::{Configuration, Device, Host};
use std::path::Path;
use uuid::Uuid;

pub(crate) struct Prepared<M: Application = Device> {
    pub host: Host<M>,
    pub adapter: M::Context,
    pub source: Value,
    pub boot: Uuid,
}
struct SoftwareOnly;
impl PlaybackPolicy for SoftwareOnly {
    fn authorize(&mut self, _: runtime::Permission) -> Result<(), runtime::Denial> {
        // Only this process's explicit software-output startup path constructs this policy.
        // No physical output adapter exists in this executable.
        Ok(())
    }
}
pub(crate) fn prepare(
    path: &Path,
    selection: &PackageSelection,
    directory: &Directory,
) -> Result<Prepared, String> {
    let (document, _) = DiskFile::open(path)?;
    let built = document
        .build_package(std::slice::from_ref(selection))
        .map_err(|issues| {
            issues
                .into_iter()
                .map(|i| i.message)
                .collect::<Vec<_>>()
                .join("；")
        })?;
    let archive = Archive::open(built.bytes.as_slice()).map_err(|e| e.to_string())?;
    let store = FileStore::open(&directory.store()).map_err(|e| e.to_string())?;
    let boot = Uuid::new_v4();
    let (mut installer, _) =
        Installer::open(store, *boot.as_bytes()).map_err(|e| format!("安装准备失败：{e:?}"))?;
    let tx = installer.transaction(1);
    installer
        .begin(tx, Identity::from_archive(&archive))
        .map_err(|e| format!("安装接纳失败：{e:?}"))?;
    for (i, chunk) in built.bytes.chunks(1024).enumerate() {
        installer
            .write(tx, i * 1024, chunk)
            .map_err(|e| format!("安装写入失败：{e:?}"))?;
    }
    installer
        .verify(tx)
        .map_err(|e| format!("安装校验失败：{e:?}"))?;
    installer
        .commit(tx)
        .map_err(|e| format!("安装提交失败：{e:?}"))?;
    let mut runtime = Runtime::new(*boot.as_bytes(), 0, MAX_LOADER_BYTES, SoftwareOnly)
        .map_err(|e| e.to_string())?;
    let permit = runtime
        .confirm_quiescent(runtime.quiescence_request().ok_or("运行准备状态错误")?, 0)
        .map_err(|e| e.to_string())?;
    runtime
        .finish_maintenance(permit, 0, || installer.snapshot().map(Some))
        .map_err(|e| format!("载入安装快照失败：{e:?}"))?;
    // The compiler was given exactly one explicit selection. Never select the project's first item.
    let [entry] = runtime.catalog() else {
        return Err("准备内容必须仅包含选定的一个节目".into());
    };
    let key = ProgramKey {
        kind: entry.kind,
        id: entry.id,
    };
    let lease = runtime
        .acquire(
            Grant {
                principal: *Uuid::new_v4().as_bytes(),
                origin: Origin::Panel,
                duration_ms: 60_000,
            },
            false,
            0,
        )
        .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?
            .result
            .map_err(|e| e.to_string())?;
    }
    runtime.release(lease, 0).map_err(|e| e.to_string())?;
    let steps: Vec<_> = runtime
        .steps()
        .iter()
        .map(|s| json!({"id":Uuid::from_bytes(s.id).to_string(),"name":s.name,"number":s.number}))
        .collect();
    let source = json!({"mode":"softwareOutput","physicalOutput":false,"report":built.report,"selection":selection,"steps":steps});
    let host = Host::start(runtime, Configuration::default()).map_err(|e| e.to_string())?;
    Ok(Prepared {
        host,
        source,
        boot,
        adapter: (),
    })
}
