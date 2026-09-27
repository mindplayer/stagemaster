use super::*;
use stagemaster_install::{Code, Error, Identity, Installed, Installer, Phase, SlotHealth};
use stagemaster_package::Archive;
use stagemaster_playback::Player;
use stagemaster_project::{Document, EditCommand, PackageSelection, ValueMode};
use std::{cell::RefCell, rc::Rc};

type Engine = Installer<FileStore>;
fn dir() -> tempfile::TempDir {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}
fn package(level: u16) -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    value["entryPoints"] = serde_json::json!([]);
    let mut doc = Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    let view = doc.view();
    doc.edit(EditCommand::SetSceneValue {
        scene_id: view.scenes[0].id.clone(),
        fixture_id: view.fixtures[0].id.clone(),
        attribute: "dimmer".into(),
        mode: ValueMode::Literal,
        value: level,
    })
    .unwrap();
    let selection: Vec<_> = view
        .scenes
        .iter()
        .map(|s| PackageSelection::Scene { id: s.id.clone() })
        .chain(
            view.sequences
                .iter()
                .map(|s| PackageSelection::Sequence { id: s.id.clone() }),
        )
        .collect();
    doc.build_package(&selection).unwrap().bytes
}
fn identity(bytes: &[u8]) -> Identity {
    Identity::from_archive(&Archive::open(bytes).unwrap())
}
fn open(path: &Path, boot: u8) -> (Engine, Rc<RefCell<Fault>>) {
    let store = FileStore::open(path).unwrap();
    let fault = store.fault.clone();
    let (engine, _) = Engine::open(store, [boot; 16]).unwrap();
    (engine, fault)
}
fn transfer(engine: &mut Engine, counter: u64, bytes: &[u8]) -> Result<Commit, Error<io::Error>> {
    let t = engine.transaction(counter);
    let progress = engine.begin(t, identity(bytes))?;
    if progress.phase != Phase::Committed {
        for (index, chunk) in bytes.chunks(128).enumerate() {
            engine.write(t, index * 128, chunk)?;
        }
    }
    engine.verify(t)?;
    engine.commit(t)
}
fn same_frames(snapshot: &Installed<FileSnapshot>, bytes: &[u8]) {
    let expected = Archive::open(bytes).unwrap();
    assert_eq!(snapshot.archive().entries(), expected.entries());
    for index in 0..expected.entries().len() {
        let actual = snapshot.load(index).unwrap();
        let wanted = expected.load(bytes, index).unwrap();
        let mut a = Player::new(actual.plan, 0);
        let mut b = Player::new(wanted.plan, 0);
        // Loading/installing alone has not executed either plan.
        assert_eq!(a.values(), b.values());
        a.execute(0, 0).unwrap();
        b.execute(0, 0).unwrap();
        for time in (0..10_000).step_by(17) {
            a.advance(time).unwrap();
            b.advance(time).unwrap();
            let mut left = [0; 512];
            let mut right = [0; 512];
            actual.output.render(a.values(), &mut left).unwrap();
            wanted.output.render(b.values(), &mut right).unwrap();
            assert_eq!(left, right);
        }
    }
}
fn code<T>(result: Result<T, Error<io::Error>>, expected: Code) {
    assert!(matches!(result.err(),Some(Error::Code(value)) if value==expected));
}

#[test]
fn transfer_resume_duplicate_conflicts_cancellation_and_stale_transactions() {
    let dir = dir();
    let (mut engine, fault) = open(dir.path(), 1);
    let bytes = package(10000);
    let t = engine.transaction(1);
    code(
        engine.begin(engine.transaction(0), identity(&bytes)),
        Code::Stale,
    );
    code(
        engine.begin(engine.transaction(2), identity(&bytes)),
        Code::Order,
    );
    engine.begin(t, identity(&bytes)).unwrap();
    let first = engine.write(t, 0, &bytes[..64]).unwrap();
    assert_eq!(engine.begin(t, identity(&bytes)).unwrap(), first);
    assert_eq!(engine.write(t, 0, &bytes[..64]).unwrap(), first);
    let mut wrong = bytes[..64].to_vec();
    wrong[4] ^= 1;
    code(engine.write(t, 0, &wrong), Code::Conflict);
    code(engine.write(t, 63, &bytes[63..65]), Code::Order);
    code(engine.write(t, 65, &bytes[65..66]), Code::Order);
    code(engine.write(t, usize::MAX, &[1]), Code::Bounds);
    code(engine.write(t, 64, &[]), Code::Bounds);
    code(engine.write(t, 64, &[0; 1025]), Code::Bounds);
    code(engine.verify(t), Code::Incomplete);
    code(
        engine.begin(engine.transaction(2), identity(&bytes)),
        Code::Busy,
    );
    let mut other = identity(&bytes);
    other.digest[0] ^= 1;
    code(engine.begin(t, other), Code::Conflict);
    assert_eq!(engine.progress(), Some(first));
    // A reconnect queries this same installer and resumes its acknowledged prefix.
    for offset in (64..bytes.len()).step_by(37) {
        engine
            .write(t, offset, &bytes[offset..(offset + 37).min(bytes.len())])
            .unwrap();
    }
    engine.verify(t).unwrap();
    code(engine.write(t, 0, &bytes[..64]), Code::State);
    let commit = engine.commit(t).unwrap();
    let writes = fault.borrow().seen.len();
    assert_eq!(engine.commit(t).unwrap(), commit);
    assert_eq!(
        engine
            .begin(engine.transaction(2), identity(&bytes))
            .unwrap()
            .phase,
        Phase::Committed
    );
    assert_eq!(fault.borrow().seen.len(), writes);
    code(engine.commit(t), Code::Stale);
    code(engine.cancel(engine.transaction(2)), Code::State);
    let next = package(20000);
    let t = engine.transaction(3);
    engine.begin(t, identity(&next)).unwrap();
    engine.write(t, 0, &next[..64]).unwrap();
    engine.cancel(t).unwrap();
    assert_eq!(engine.cancel(t).unwrap().phase, Phase::Cancelled);
    assert_eq!(
        engine.begin(t, identity(&next)).unwrap().phase,
        Phase::Cancelled
    );
    same_frames(&engine.snapshot().unwrap(), &bytes);
    transfer(&mut engine, 4, &next).unwrap();
    code(engine.cancel(t), Code::Stale);
    same_frames(&engine.snapshot().unwrap(), &next);
}

#[test]
fn corrupt_unknown_version_wrong_identity_and_partial_write_never_publish() {
    let dir = dir();
    let (mut engine, fault) = open(dir.path(), 1);
    let old = package(10000);
    let commit = transfer(&mut engine, 1, &old).unwrap();
    let new = package(20000);
    for (index, offset) in [new.len() - 1, 8].into_iter().enumerate() {
        let t = engine.transaction(index as u64 + 2);
        engine.begin(t, identity(&new)).unwrap();
        let mut bytes = new.clone();
        bytes[offset] ^= 1;
        for (n, chunk) in bytes.chunks(1024).enumerate() {
            engine.write(t, n * 1024, chunk).unwrap();
        }
        assert!(matches!(engine.verify(t), Err(Error::Package(_))));
        assert_eq!(engine.progress().unwrap().phase, Phase::Failed);
        assert_eq!(engine.head(), Some(commit));
        code(engine.commit(t), Code::State);
        engine.cancel(t).unwrap();
    }
    let t = engine.transaction(4);
    let mut wrong = identity(&new);
    wrong.digest[0] ^= 1;
    engine.begin(t, wrong).unwrap();
    for (n, chunk) in new.chunks(1024).enumerate() {
        engine.write(t, n * 1024, chunk).unwrap();
    }
    assert!(matches!(engine.verify(t), Err(Error::Package(_))));
    engine.cancel(t).unwrap();
    let t = engine.transaction(5);
    engine.begin(t, identity(&new)).unwrap();
    fault.borrow_mut().partial_write = true;
    assert!(matches!(
        engine.write(t, 0, &new[..64]),
        Err(Error::Storage(_))
    ));
    assert_eq!(engine.progress().unwrap().received, 0);
    code(engine.write(t, 0, &new[..64]), Code::State);
    engine.cancel(t).unwrap();
    fault.borrow_mut().partial_write = false;
    same_frames(&engine.snapshot().unwrap(), &old);
    transfer(&mut engine, 6, &new).unwrap();
}

#[test]
fn valid_hash_unknown_version_and_change_after_verification_are_rejected() {
    use sha2::{Digest, Sha256};
    let dir = dir();
    let (mut engine, _) = open(dir.path(), 1);
    let old = package(10000);
    let current = transfer(&mut engine, 1, &old).unwrap();
    let new = package(20000);
    let mut unknown = new.clone();
    unknown[8] = 99;
    let mut hash = Sha256::new();
    hash.update(&unknown[..32]);
    hash.update(&unknown[64..]);
    let digest: [u8; 32] = hash.finalize().into();
    unknown[32..64].copy_from_slice(&digest);
    let t = engine.transaction(2);
    engine
        .begin(
            t,
            Identity {
                bytes: unknown.len(),
                digest,
            },
        )
        .unwrap();
    for (n, chunk) in unknown.chunks(128).enumerate() {
        engine.write(t, n * 128, chunk).unwrap();
    }
    assert!(matches!(
        engine.verify(t),
        Err(Error::Package(stagemaster_package::Error::Version))
    ));
    engine.cancel(t).unwrap();
    let t = engine.transaction(3);
    engine.begin(t, identity(&new)).unwrap();
    for (n, chunk) in new.chunks(128).enumerate() {
        engine.write(t, n * 128, chunk).unwrap();
    }
    engine.verify(t).unwrap();
    let mut changed = new;
    *changed.last_mut().unwrap() ^= 1;
    fs::write(dir.path().join("slot-1.smpkg"), changed).unwrap();
    assert!(matches!(engine.commit(t), Err(Error::Package(_))));
    assert_eq!(engine.head(), Some(current));
    same_frames(&engine.snapshot().unwrap(), &old);
}

#[test]
fn lost_commit_receipt_with_unreadable_metadata_remains_uncertain_until_reconciled() {
    let dir = dir();
    let (mut engine, fault) = open(dir.path(), 1);
    let old = package(10000);
    let current = transfer(&mut engine, 1, &old).unwrap();
    let new = package(20000);
    let t = engine.transaction(2);
    engine.begin(t, identity(&new)).unwrap();
    for (n, chunk) in new.chunks(128).enumerate() {
        engine.write(t, n * 128, chunk).unwrap();
    }
    engine.verify(t).unwrap();
    *fault.borrow_mut() = Fault {
        fail_at: Some(5),
        ..Fault::default()
    };
    assert!(matches!(engine.commit(t), Err(Error::CommitUncertain(_))));
    assert_eq!(fault.borrow().seen.last(), Some(&"record:renamed"));
    let metadata = dir.path().join("slot-1.commit");
    let held = dir.path().join("held.commit");
    fs::rename(&metadata, &held).unwrap();
    fs::create_dir(&metadata).unwrap();
    assert!(matches!(engine.reconcile(), Err(Error::Storage(_))));
    assert_eq!(engine.head(), Some(current));
    assert_eq!(engine.progress().unwrap().phase, Phase::Uncertain);
    code(engine.cancel(t), Code::Uncertain);
    fs::remove_dir(&metadata).unwrap();
    fs::rename(held, metadata).unwrap();
    let recovered = engine.reconcile().unwrap();
    assert_eq!(recovered.selected.unwrap().identity, identity(&new));
    assert_eq!(engine.commit(t).unwrap(), recovered.selected.unwrap());
    same_frames(&engine.snapshot().unwrap(), &new);
}

#[test]
fn torn_target_record_during_uncertain_commit_falls_back_and_can_be_replaced() {
    let dir = dir();
    let (mut engine, fault) = open(dir.path(), 1);
    let old = package(10000);
    let current = transfer(&mut engine, 1, &old).unwrap();
    let new = package(20000);
    let t = engine.transaction(2);
    engine.begin(t, identity(&new)).unwrap();
    for (n, chunk) in new.chunks(128).enumerate() {
        engine.write(t, n * 128, chunk).unwrap();
    }
    engine.verify(t).unwrap();
    *fault.borrow_mut() = Fault {
        fail_at: Some(5),
        ..Fault::default()
    };
    assert!(matches!(engine.commit(t), Err(Error::CommitUncertain(_))));
    // Model a medium that detects a torn target record rather than promising a 96-byte atomic write.
    fs::write(dir.path().join("slot-1.commit"), b"torn-target-record").unwrap();
    let report = engine.reconcile().unwrap();
    assert!(matches!(report.slots[1], SlotHealth::InvalidRecord));
    assert_eq!(report.selected, Some(current));
    assert_eq!(engine.progress().unwrap().phase, Phase::Failed);
    same_frames(&engine.snapshot().unwrap(), &old);
    engine.cancel(t).unwrap();
    fault.borrow_mut().fail_at = None;
    transfer(&mut engine, 3, &new).unwrap();
    same_frames(&engine.snapshot().unwrap(), &new);
}

#[test]
fn pinned_old_reader_survives_new_commit_and_blocks_reuse_even_across_reopen() {
    let dir = dir();
    let (mut engine, _) = open(dir.path(), 1);
    let a = package(10000);
    let b = package(20000);
    let c = package(30000);
    transfer(&mut engine, 1, &a).unwrap();
    let old = engine.snapshot().unwrap();
    transfer(&mut engine, 2, &b).unwrap();
    same_frames(&old, &a);
    let t = engine.transaction(3);
    assert!(matches!(
        engine.begin(t, identity(&c)),
        Err(Error::Storage(_))
    ));
    engine.cancel(t).unwrap();
    drop(engine);
    let (mut engine, _) = open(dir.path(), 2);
    let t = engine.transaction(1);
    assert!(engine.begin(t, identity(&c)).is_err());
    engine.cancel(t).unwrap();
    same_frames(&old, &a);
    drop(old);
    transfer(&mut engine, 2, &c).unwrap();
    same_frames(&engine.snapshot().unwrap(), &c);
}

#[test]
fn every_write_failure_reconciles_and_reopens_to_a_complete_version_then_retries() {
    let a = package(10000);
    let b = package(20000);
    let dir0 = dir();
    let (mut baseline, fault) = open(dir0.path(), 1);
    transfer(&mut baseline, 1, &a).unwrap();
    fault.borrow_mut().seen.clear();
    transfer(&mut baseline, 2, &b).unwrap();
    let points = fault.borrow().seen.clone();
    assert!(points.len() > 20);
    for (index, point) in points.iter().enumerate() {
        let dir = dir();
        let (mut engine, fault) = open(dir.path(), 1);
        let old = transfer(&mut engine, 1, &a).unwrap();
        let original = fs::read(dir.path().join("slot-0.smpkg")).unwrap();
        *fault.borrow_mut() = Fault {
            fail_at: Some(index + 1),
            ..Fault::default()
        };
        assert!(transfer(&mut engine, 2, &b).is_err(), "{point}");
        same_frames(&engine.snapshot().unwrap(), &a);
        let committed = matches!(*point, "record:renamed" | "record:directory-synced");
        if engine.progress().unwrap().phase == Phase::Uncertain {
            code(engine.cancel(engine.transaction(2)), Code::Uncertain);
            code(
                engine.begin(engine.transaction(3), identity(&b)),
                Code::Uncertain,
            );
            let next = fault.borrow().seen.len() + 1;
            fault.borrow_mut().fail_at = Some(next);
            assert!(engine.reconcile().is_err());
            assert_eq!(engine.progress().unwrap().phase, Phase::Uncertain);
            fault.borrow_mut().fail_at = None;
            let report = engine.reconcile().unwrap();
            assert_eq!(
                report.selected.unwrap().identity,
                if committed {
                    identity(&b)
                } else {
                    identity(&a)
                }
            );
        }
        assert_eq!(fs::read(dir.path().join("slot-0.smpkg")).unwrap(), original);
        let stale = engine.transaction(2);
        drop(engine);
        let (mut reopened, _) = open(dir.path(), 2);
        assert_eq!(
            reopened.head().unwrap().identity,
            if committed {
                identity(&b)
            } else {
                old.identity
            },
            "{point}"
        );
        code(reopened.begin(stale, identity(&b)), Code::Stale);
        transfer(&mut reopened, 1, &b).unwrap();
        same_frames(&reopened.snapshot().unwrap(), &b);
    }
}

#[test]
fn abrupt_process_exit_at_every_storage_point_retains_a_complete_committed_version() {
    let a = package(10000);
    let b = package(20000);
    let control = dir();
    let (mut engine, fault) = open(control.path(), 1);
    transfer(&mut engine, 1, &a).unwrap();
    fault.borrow_mut().seen.clear();
    transfer(&mut engine, 2, &b).unwrap();
    let points = fault.borrow().seen.clone();
    for (index, point) in points.iter().enumerate() {
        let dir = dir();
        let (mut initial, _) = open(dir.path(), 1);
        transfer(&mut initial, 1, &a).unwrap();
        drop(initial);
        fs::write(dir.path().join("input.smpkg"), &b).unwrap();
        fs::write(dir.path().join("keep.txt"), b"keep").unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tests::abrupt_exit_worker",
                "--ignored",
                "--nocapture",
            ])
            .env("STAGEMASTER_INSTALL_TEST_DIRECTORY", dir.path())
            .env("STAGEMASTER_INSTALL_TEST_EXIT_AT", (index + 1).to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(91), "{point}");
        let (mut recovered, _) = open(dir.path(), 3);
        let committed = matches!(*point, "record:renamed" | "record:directory-synced");
        assert_eq!(
            recovered.head().unwrap().identity,
            if committed {
                identity(&b)
            } else {
                identity(&a)
            },
            "{point}"
        );
        same_frames(
            &recovered.snapshot().unwrap(),
            if committed { &b } else { &a },
        );
        assert_eq!(fs::read(dir.path().join("keep.txt")).unwrap(), b"keep");
        assert!(fs::read_dir(dir.path()).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".install-")
        }));
        transfer(&mut recovered, 1, &b).unwrap();
    }
}
#[test]
#[ignore = "由逐点中断父测试作为独立子进程调用"]
fn abrupt_exit_worker() {
    let Some(path) = std::env::var_os("STAGEMASTER_INSTALL_TEST_DIRECTORY") else {
        return;
    };
    let path = PathBuf::from(path);
    let (mut engine, fault) = open(&path, 2);
    let at = std::env::var("STAGEMASTER_INSTALL_TEST_EXIT_AT")
        .unwrap()
        .parse()
        .unwrap();
    *fault.borrow_mut() = Fault {
        exit_at: Some(at),
        ..Fault::default()
    };
    let bytes = fs::read(path.join("input.smpkg")).unwrap();
    transfer(&mut engine, 1, &bytes).unwrap();
    panic!("requested crash point was not reached");
}

#[test]
fn startup_reports_damage_and_ignores_uncommitted_payloads_without_erasing_evidence() {
    let dir = dir();
    let a = package(10000);
    let b = package(20000);
    let c = package(30000);
    let (mut engine, _) = open(dir.path(), 1);
    let a_commit = transfer(&mut engine, 1, &a).unwrap();
    drop(engine);
    fs::write(dir.path().join("slot-1.smpkg"), &b).unwrap();
    let (mut engine, report) = Engine::open(FileStore::open(dir.path()).unwrap(), [2; 16]).unwrap();
    assert_eq!(report.selected, Some(a_commit));
    transfer(&mut engine, 1, &b).unwrap();
    drop(engine);
    let mut damaged = b.clone();
    damaged[90] ^= 1;
    fs::write(dir.path().join("slot-1.smpkg"), &damaged).unwrap();
    let (mut engine, report) = Engine::open(FileStore::open(dir.path()).unwrap(), [3; 16]).unwrap();
    assert!(matches!(report.slots[1], SlotHealth::InvalidPackage(_)));
    assert_eq!(report.selected, Some(a_commit));
    assert_eq!(report.high_water, 2);
    assert_eq!(fs::read(dir.path().join("slot-1.smpkg")).unwrap(), damaged);
    let c_commit = transfer(&mut engine, 1, &c).unwrap();
    assert_eq!(c_commit.generation, 3);
    drop(engine);
    fs::write(dir.path().join("slot-1.commit"), b"torn").unwrap();
    let (engine, report) = Engine::open(FileStore::open(dir.path()).unwrap(), [4; 16]).unwrap();
    assert!(matches!(report.slots[1], SlotHealth::InvalidRecord));
    assert_eq!(report.selected, Some(a_commit));
    drop(engine);
    fs::write(dir.path().join("slot-0.commit"), b"also torn").unwrap();
    let (engine, report) = Engine::open(FileStore::open(dir.path()).unwrap(), [5; 16]).unwrap();
    assert!(report.selected.is_none());
    code(engine.snapshot(), Code::Empty);
    assert_eq!(fs::read(dir.path().join("slot-0.smpkg")).unwrap(), a);
}

#[test]
fn record_conflicts_generation_exhaustion_and_competing_writers_are_explicit() {
    let dir = dir();
    let a = package(10000);
    let b = package(20000);
    let (mut engine, _) = open(dir.path(), 1);
    assert!(FileStore::open(dir.path()).is_err());
    let a_commit = transfer(&mut engine, 1, &a).unwrap();
    let b_commit = transfer(&mut engine, 2, &b).unwrap();
    drop(engine);
    let same = Commit {
        generation: a_commit.generation,
        ..b_commit
    };
    fs::write(dir.path().join("slot-1.commit"), same.encode().unwrap()).unwrap();
    code(
        Engine::open(FileStore::open(dir.path()).unwrap(), [2; 16]),
        Code::Metadata,
    );
    let full = Commit {
        generation: u64::MAX,
        ..b_commit
    };
    fs::write(dir.path().join("slot-1.commit"), full.encode().unwrap()).unwrap();
    let (mut engine, _) = open(dir.path(), 3);
    code(
        engine.begin(engine.transaction(1), identity(&a)),
        Code::Exhausted,
    );
    same_frames(&engine.snapshot().unwrap(), &b);
}

#[test]
fn cancelled_first_upload_never_becomes_installed_and_new_boot_restarts_from_zero() {
    let dir = dir();
    code(
        Engine::open(FileStore::open(dir.path()).unwrap(), [0; 16]),
        Code::Identity,
    );
    let (mut engine, _) = open(dir.path(), 1);
    let bytes = package(10000);
    let t = engine.transaction(1);
    let too_large = Identity {
        bytes: MAX_PACKAGE_BYTES + 1,
        ..identity(&bytes)
    };
    code(engine.begin(t, too_large), Code::Bounds);
    assert!(engine.progress().is_none());
    engine.begin(t, identity(&bytes)).unwrap();
    for (n, chunk) in bytes.chunks(128).enumerate() {
        engine.write(t, n * 128, chunk).unwrap();
    }
    engine.verify(t).unwrap();
    engine.cancel(t).unwrap();
    assert!(engine.head().is_none());
    code(engine.snapshot(), Code::Empty);
    drop(engine);
    let (mut reopened, _) = open(dir.path(), 2);
    assert!(reopened.head().is_none());
    assert!(reopened.progress().is_none());
    code(reopened.commit(t), Code::Stale);
    let committed = transfer(&mut reopened, 1, &bytes).unwrap();
    assert_eq!(committed.generation, 1);
    same_frames(&reopened.snapshot().unwrap(), &bytes);
}

#[cfg(unix)]
#[test]
fn managed_links_and_non_regular_paths_are_never_followed_or_replaced() {
    use std::os::unix::fs::symlink;
    let dir = dir();
    let root = dir.path().join("store");
    fs::create_dir(&root).unwrap();
    let target = dir.path().join("do-not-touch");
    fs::write(&target, b"external").unwrap();
    let link = dir.path().join("alias");
    symlink(&root, &link).unwrap();
    assert!(FileStore::open(&link).is_err());
    for name in [
        "installer.lock",
        "slot-0.commit",
        "slot-0.lease",
        "slot-0.smpkg",
    ] {
        let path = root.join(name);
        symlink(&target, &path).unwrap();
        let opened = FileStore::open(&root).and_then(|store| {
            Engine::open(store, [1; 16])
                .map(|(engine, _)| engine)
                .map_err(|e| io::Error::other(e.to_string()))
        });
        if let Ok(mut engine) = opened {
            assert!(
                engine
                    .begin(engine.transaction(1), identity(&package(10000)))
                    .is_err()
            );
        }
        assert_eq!(fs::read(&target).unwrap(), b"external");
        fs::remove_file(path).unwrap();
    }
    fs::hard_link(&target, root.join("slot-0.smpkg")).unwrap();
    let (mut engine, _) = open(&root, 1);
    assert!(
        engine
            .begin(engine.transaction(1), identity(&package(10000)))
            .is_err()
    );
    assert_eq!(fs::read(target).unwrap(), b"external");
}
