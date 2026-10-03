use crate::Package;
use stagemaster_install::{Commit, Installer, Phase};
use stagemaster_install_store::{FileSnapshot, FileStore};
use stagemaster_package::{MAX_LOADER_BYTES, ReadAt};
use stagemaster_runtime::{
    Action, Denial, Grant, Lease, Origin, Permission, PlaybackPolicy, ProgramKey, Request, Runtime,
};

#[derive(Default)]
pub struct Policy {
    pub allowed: bool,
    pub calls: Vec<Permission>,
}
impl PlaybackPolicy for Policy {
    fn authorize(&mut self, permission: Permission) -> Result<(), Denial> {
        self.calls.push(permission);
        if self.allowed {
            Ok(())
        } else {
            Err(Denial::Missing)
        }
    }
}
pub type Device = Runtime<FileSnapshot, Policy>;
pub fn install(installer: &mut Installer<FileStore>, package: &Package, counter: u64) -> Commit {
    let transaction = installer.transaction(counter);
    let progress = installer.begin(transaction, package.identity()).unwrap();
    if progress.phase != Phase::Committed {
        let mut block = [0; 1024];
        let mut offset = 0;
        while offset < package.len() {
            let len = block.len().min(package.len() - offset);
            package.read_exact(offset, &mut block[..len]).unwrap();
            installer.write(transaction, offset, &block[..len]).unwrap();
            offset += len;
        }
        installer.verify(transaction).unwrap();
    }
    installer.commit(transaction).unwrap()
}
pub fn device(installer: &Installer<FileStore>) -> (Device, Lease) {
    let mut device = Runtime::new([2; 16], 0, MAX_LOADER_BYTES, Policy::default()).unwrap();
    bind(&mut device, installer, 0);
    let lease = device
        .acquire(
            Grant {
                principal: [3; 16],
                origin: Origin::Panel,
                duration_ms: 60000,
            },
            false,
            0,
        )
        .unwrap();
    load(&mut device, lease, 0);
    (device, lease)
}
pub fn bind(device: &mut Device, installer: &Installer<FileStore>, now: u64) {
    let permit = device
        .confirm_quiescent(device.quiescence_request().unwrap(), now)
        .unwrap();
    device
        .finish_maintenance(permit, now, || installer.snapshot().map(Some))
        .unwrap();
}
pub fn load(device: &mut Device, lease: Lease, now: u64) {
    let entry = &device.catalog()[0];
    let key = ProgramKey {
        kind: entry.kind,
        id: entry.id,
    };
    apply(device, lease, Action::Select(key), now).unwrap();
    apply(device, lease, Action::Load, now).unwrap();
}
pub fn apply(
    device: &mut Device,
    lease: Lease,
    action: Action,
    now: u64,
) -> Result<(), stagemaster_runtime::Code> {
    device
        .submit(
            Request {
                lease,
                serial: device.state().owner.unwrap().serial + 1,
                expected_revision: device.state().revision,
                action,
            },
            now,
        )
        .unwrap()
        .result
}
