//! File-only acceptance tool. It has no output driver, radio or production authorization.
use stagemaster_install::{Identity, Installer};
use stagemaster_install_store::FileStore;
use stagemaster_package::{Archive, MAX_LOADER_BYTES, MAX_PACKAGE_BYTES, ReadAt};
use stagemaster_playback::Player;
use stagemaster_runtime::{
    Action, Code, Denial, Grant, Lease, Origin, Permission, PlaybackPolicy, ProgramKey, Request,
    Runtime,
};
use std::{error::Error, io::Read, path::PathBuf};

struct SoftwareAcceptance;
impl PlaybackPolicy for SoftwareAcceptance {
    fn authorize(&mut self, _: Permission) -> Result<(), Denial> {
        Ok(())
    }
}
fn control<R: ReadAt>(
    runtime: &mut Runtime<R, SoftwareAcceptance>,
    lease: Lease,
    action: Action,
    now: u64,
) -> Result<(), Code> {
    let state = runtime.state();
    runtime
        .submit(
            Request {
                lease,
                serial: state.owner.ok_or(Code::Lease)?.serial + 1,
                expected_revision: state.revision,
                action,
            },
            now,
        )?
        .result
}
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("用法：replay 播放包.smpkg（仅软件输出核验）")?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(u64::try_from(MAX_PACKAGE_BYTES)? + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_PACKAGE_BYTES {
        return Err("播放包超出 2 MiB".into());
    }
    let archive = Archive::open(bytes.as_slice())?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    std::fs::create_dir_all(&root)?;
    let directory = tempfile::tempdir_in(root)?;
    let (mut installer, _) = Installer::open(FileStore::open(directory.path())?, [1; 16])?;
    let transaction = installer.transaction(1);
    installer.begin(transaction, Identity::from_archive(&archive))?;
    for (index, chunk) in bytes.chunks(1024).enumerate() {
        installer.write(transaction, index * 1024, chunk)?;
    }
    installer.verify(transaction)?;
    installer.commit(transaction)?;
    for (index, entry) in archive.entries().iter().enumerate() {
        let mut boot = [2; 16];
        boot[0] = u8::try_from(index + 1)?;
        let mut runtime = Runtime::new(boot, 0, MAX_LOADER_BYTES, SoftwareAcceptance)?;
        // No physical driver exists in this program. There are no queued frames to drain.
        let permit =
            runtime.confirm_quiescent(runtime.quiescence_request().ok_or(Code::Mode)?, 0)?;
        runtime.finish_maintenance(permit, 0, || installer.snapshot().map(Some))?;
        let grant = Grant {
            principal: [3; 16],
            origin: Origin::Panel,
            duration_ms: 60_000,
        };
        let lease = runtime.acquire(grant, false, 0)?;
        control(
            &mut runtime,
            lease,
            Action::Select(ProgramKey {
                kind: entry.kind,
                id: entry.id,
            }),
            0,
        )?;
        control(&mut runtime, lease, Action::Load, 0)?;
        let program = archive.load(bytes.as_slice(), index)?;
        let mut reference = Player::new(program.plan, 0);
        control(
            &mut runtime,
            lease,
            Action::Start {
                step: program.labels[0].id,
            },
            0,
        )?;
        reference.execute(0, 0)?;
        let mut compared = 0;
        for now in (0..10_000).step_by(25) {
            match now {
                2500 => {
                    control(&mut runtime, lease, Action::Pause, now)?;
                    reference.pause(now)?;
                }
                3500 => {
                    control(&mut runtime, lease, Action::Resume, now)?;
                    reference.resume(now)?;
                }
                4500 => runtime.release(lease, now)?,
                _ => {}
            }
            runtime.tick(now)?;
            reference.advance(now)?;
            let mut actual = [0; 512];
            let mut expected = [0; 512];
            runtime.render(&mut actual)?.ok_or(Code::NotLoaded)?;
            program.output.render(reference.values(), &mut expected)?;
            if actual != expected {
                return Err(format!("节目 {} 在 {now} ms 输出不一致", entry.name).into());
            }
            compared += 1;
        }
        let lease = runtime.acquire(grant, false, 10_000)?;
        control(&mut runtime, lease, Action::Stop, 10_000)?;
        println!(
            "{}",
            serde_json::json!({"program":entry.name,"frames_compared":compared,"pause_resume":true,"continued_without_connection":true,"stopped":runtime.state().instance.is_none(),"physical_output":false})
        );
    }
    Ok(())
}
