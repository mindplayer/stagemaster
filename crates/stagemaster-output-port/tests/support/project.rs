use stagemaster_install::{Identity, Installer};
use stagemaster_install_store::{FileSnapshot, FileStore};
use stagemaster_package::{Archive, MAX_LOADER_BYTES};
use stagemaster_project::{Document, PackageSelection};
use stagemaster_runtime::*;
use std::path::PathBuf;

pub struct SoftwareOnly;
impl PlaybackPolicy for SoftwareOnly {
    fn authorize(&mut self, _: Permission) -> Result<(), Denial> {
        Ok(())
    }
}
pub type Device = Runtime<FileSnapshot, SoftwareOnly>;
pub struct Project {
    pub _dir: tempfile::TempDir,
    pub installer: Installer<FileStore>,
}
impl Project {
    pub fn new() -> Self {
        let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        json["entryPoints"] = serde_json::json!([]);
        json["lighting"]["profiles"][0]["attributes"][0]["default"]["value"] = 12000.into();
        json["lighting"]["sequences"][0]["repeat"] = "loop".into();
        let doc = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
        let bytes = doc
            .build_package(&[PackageSelection::Sequence {
                id: doc.view().sequences[0].id.clone(),
            }])
            .unwrap()
            .bytes;
        let archive = Archive::open(bytes.as_slice()).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
        std::fs::create_dir_all(&root).unwrap();
        let dir = tempfile::tempdir_in(root).unwrap();
        let (mut installer, _) =
            Installer::open(FileStore::open(dir.path()).unwrap(), [7; 16]).unwrap();
        let transaction = installer.transaction(1);
        installer
            .begin(transaction, Identity::from_archive(&archive))
            .unwrap();
        for (i, block) in bytes.chunks(1024).enumerate() {
            installer.write(transaction, i * 1024, block).unwrap();
        }
        installer.verify(transaction).unwrap();
        installer.commit(transaction).unwrap();
        Self {
            _dir: dir,
            installer,
        }
    }
    pub fn runtime() -> Device {
        Runtime::new([8; 16], 0, MAX_LOADER_BYTES, SoftwareOnly).unwrap()
    }
}
pub fn control(runtime: &mut Device, lease: Lease, action: Action, now: u64) {
    let state = runtime.state();
    runtime
        .submit(
            Request {
                lease,
                serial: state.owner.unwrap().serial + 1,
                expected_revision: state.revision,
                action,
            },
            now,
        )
        .unwrap()
        .result
        .unwrap();
}
pub fn acquire(runtime: &mut Device, now: u64) -> Lease {
    runtime
        .acquire(
            Grant {
                principal: [9; 16],
                origin: Origin::Remote,
                duration_ms: 60_000,
            },
            false,
            now,
        )
        .unwrap()
}
