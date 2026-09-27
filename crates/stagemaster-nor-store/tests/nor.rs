mod support;
use sha2::{Digest, Sha256};
use stagemaster_install::{Installer, Phase, Record, Slot, Storage};
use stagemaster_nor_store::{Code, Error, Layout, NorDevice};
use stagemaster_package::{Archive, ReadAt};
use stagemaster_playback::Player;
use stagemaster_transfer::{Assembler, AuthorizedLink, Frame, Outcome, Service, Upload};
use std::{cell::RefCell, rc::Rc};
use support::*;

fn blank() -> (Rc<RefCell<State>>, Layout) {
    let layout = Layout::new(4096).unwrap();
    (
        Rc::new(RefCell::new(State::blank(layout.total_bytes()))),
        layout,
    )
}
fn same_frames<R: ReadAt>(actual: &stagemaster_install::Installed<R>, bytes: &[u8]) {
    let expected = Archive::open(bytes).unwrap();
    assert_eq!(actual.commit().identity, identity(bytes));
    assert_eq!(actual.archive().entries().len(), expected.entries().len());
    for index in 0..expected.entries().len() {
        let a = actual.load(index).unwrap();
        let b = expected.load(bytes, index).unwrap();
        let mut left = Player::new(a.plan, 0);
        let mut right = Player::new(b.plan, 0);
        left.execute(0, 0).unwrap();
        right.execute(0, 0).unwrap();
        for time in (0..10_000).step_by(25) {
            left.advance(time).unwrap();
            right.advance(time).unwrap();
            let mut l = [0; 512];
            let mut r = [0; 512];
            a.output.render(left.values(), &mut l).unwrap();
            b.output.render(right.values(), &mut r).unwrap();
            assert_eq!(l, r);
        }
    }
}
#[test]
fn physical_words_are_programmed_once_for_odd_chunks_retries_and_padding() {
    let bytes = package(12000);
    for chunk in [1, 3, 17, 255, 257, 1024] {
        let (state, layout) = blank();
        let device = NorDevice::new(Model::<16, 256, 256>(state.clone()), layout).unwrap();
        let mut installer = Installer::open(device.open_for_installation().unwrap(), [1; 16])
            .unwrap()
            .0;
        let t = installer.transaction(1);
        installer.begin(t, identity(&bytes)).unwrap();
        for (i, b) in bytes.chunks(chunk).enumerate() {
            installer.write(t, i * chunk, b).unwrap();
            let mutations = state.borrow().mutations();
            installer.write(t, i * chunk, b).unwrap();
            assert_eq!(state.borrow().mutations(), mutations);
        }
        installer.verify(t).unwrap();
        installer.commit(t).unwrap();
        same_frames(&installer.snapshot().unwrap(), &bytes);
        drop(installer);
        let mut read = Installer::open(device.open_read_only().unwrap(), [2; 16])
            .unwrap()
            .0;
        same_frames(&read.snapshot().unwrap(), &bytes);
        let mutations = state.borrow().mutations();
        assert!(install(&mut read, &package(24000), 1, 17).is_err());
        assert_eq!(state.borrow().mutations(), mutations);
        let payload = layout.payload_offset(Slot::A);
        assert!(
            state.borrow().bytes[payload + bytes.len()..payload + bytes.len().div_ceil(256) * 256]
                .iter()
                .all(|b| *b == 0xff)
        );
    }
}
#[test]
fn every_driver_boundary_recovers_only_a_whole_old_or_new_package() {
    let (state, layout) = blank();
    let first = package(11000);
    let old = package(22000);
    let new = package(33000);
    {
        let device = device(&state, layout);
        let mut i = open(&device);
        install(&mut i, &first, 1, 17).unwrap();
        install(&mut i, &old, 2, 17).unwrap();
    }
    let seed = state.borrow().clone();
    let operations = {
        let device = device(&state, layout);
        let mut i = open(&device);
        state.borrow_mut().reset();
        install(&mut i, &new, 1, 17).unwrap();
        state.borrow().operations.clone()
    };
    assert!(
        operations
            .iter()
            .any(|o| o.kind == Kind::Erase && o.length == 4096)
    );
    assert!(
        operations
            .iter()
            .any(|o| o.kind == Kind::Write && o.offset == 256)
    );
    for mode in [Mode::Before, Mode::Partial, Mode::After, Mode::Silent] {
        for point in 1..=operations.len() {
            *state.borrow_mut() = seed.clone();
            {
                let device = device(&state, layout);
                let mut i = open(&device);
                state.borrow_mut().reset();
                state.borrow_mut().fault = Some((point, mode));
                let result = install(&mut i, &new, 1, 17);
                if result.is_ok() {
                    assert_eq!(i.head().unwrap().identity, identity(&new));
                }
                // Loss of all volatile state immediately after the failing operation returns.
                // No recovery/cancel/cleanup performs a physical mutation.
            }
            state.borrow_mut().reset();
            {
                let saved_state = state.borrow();
                let bytes = &saved_state.bytes;
                assert_eq!(
                    &bytes[4096..8192],
                    &seed.bytes[4096..8192],
                    "other metadata {point} {mode:?}"
                );
                assert_eq!(
                    &bytes[layout.payload_offset(Slot::B)..],
                    &seed.bytes[layout.payload_offset(Slot::B)..]
                );
            }
            let device = device(&state, layout);
            let mut i = open(&device);
            let head = i.head().unwrap().identity;
            assert!(
                head == identity(&old) || head == identity(&new),
                "{point} {mode:?}"
            );
            assert!(i.snapshot().is_ok());
            install(&mut i, &new, 1, 17).unwrap();
            assert_eq!(i.head().unwrap().identity, identity(&new));
        }
    }
    eprintln!(
        "NOR fault matrix: {} driver boundaries × 4 modes",
        operations.len()
    );
}
#[test]
fn interrupted_first_install_has_no_phantom_commit_and_cancel_reclaims_scratch() {
    let bytes = package(12345);
    for chunks in [0, 1, 7, bytes.len().div_ceil(17)] {
        let (state, layout) = blank();
        {
            let d = device(&state, layout);
            let mut i = open(&d);
            let t = i.transaction(1);
            i.begin(t, identity(&bytes)).unwrap();
            for (index, b) in bytes.chunks(17).take(chunks).enumerate() {
                i.write(t, index * 17, b).unwrap();
            }
            if chunks == bytes.len().div_ceil(17) {
                i.verify(t).unwrap();
            }
        }
        let d = device(&state, layout);
        let mut i = open(&d);
        assert!(i.head().is_none());
        let t = i.transaction(1);
        i.begin(t, identity(&bytes)).unwrap();
        i.cancel(t).unwrap();
        install(&mut i, &bytes, 2, 3).unwrap();
        same_frames(&i.snapshot().unwrap(), &bytes);
    }
}
#[test]
fn leased_old_slot_survives_store_reopen_and_prevents_any_reuse_erase() {
    let (state, layout) = blank();
    let device = device(&state, layout);
    let first = package(10000);
    let second = package(20000);
    let third = package(30000);
    let mut installer = open(&device);
    install(&mut installer, &first, 1, 17).unwrap();
    let lease = installer.snapshot().unwrap();
    assert!(matches!(
        device.open_for_installation(),
        Err(Error::Code(Code::Busy))
    ));
    drop(installer);
    let mut installer = open(&device);
    install(&mut installer, &second, 1, 17).unwrap();
    state.borrow_mut().reset();
    let transaction = installer.transaction(2);
    assert!(installer.begin(transaction, identity(&third)).is_err());
    assert_eq!(state.borrow().mutations(), 0);
    same_frames(&lease, &first);
    same_frames(&installer.snapshot().unwrap(), &second);
    installer.cancel(transaction).unwrap();
    drop(lease);
    install(&mut installer, &third, 3, 17).unwrap();
    same_frames(&installer.snapshot().unwrap(), &third);
}
#[test]
fn torn_metadata_bytes_and_latest_package_damage_fall_back_without_formatting() {
    let (state, layout) = blank();
    let a = package(10000);
    let b = package(20000);
    {
        let d = device(&state, layout);
        let mut i = open(&d);
        install(&mut i, &a, 1, 17).unwrap();
        install(&mut i, &b, 2, 17).unwrap();
    }
    let seed = state.borrow().clone();
    // Include selective version-byte erasure (not just a prefix power-loss model).
    for byte in 0..512 {
        *state.borrow_mut() = seed.clone();
        state.borrow_mut().bytes[4096 + byte] ^= 0x80;
        let d = device(&state, layout);
        let i = open(&d);
        assert_eq!(
            i.head().unwrap().identity,
            identity(&a),
            "metadata byte {byte}"
        );
    }
    *state.borrow_mut() = seed.clone();
    state.borrow_mut().bytes[layout.payload_offset(Slot::B) + b.len() - 1] ^= 1;
    let d = device(&state, layout);
    state.borrow_mut().reset();
    let i = open(&d);
    assert_eq!(i.head().unwrap().identity, identity(&a));
    assert_eq!(state.borrow().mutations(), 0);
}
#[test]
fn valid_unsupported_envelopes_geometry_and_capacity_fail_closed() {
    let (state, layout) = blank();
    assert!(Layout::new(0).is_err());
    assert!(Layout::new(4097).is_err());
    assert!(Layout::new(usize::MAX).is_err());
    assert!(matches!(
        NorDevice::new(Model::<32, 4, 4096>(state.clone()), layout),
        Err(Error::Code(Code::Geometry))
    ));
    assert!(matches!(
        NorDevice::new(Model::<4, 512, 4096>(state.clone()), layout),
        Err(Error::Code(Code::Geometry))
    ));
    assert!(matches!(
        NorDevice::new(Model::<4, 4, 8192>(state.clone()), layout),
        Err(Error::Code(Code::Geometry))
    ));
    assert!(matches!(
        NorDevice::new(
            Model::<4, 4, 4096>(state.clone()),
            Layout::new(8192).unwrap()
        ),
        Err(Error::Code(Code::Capacity))
    ));
    {
        let d = device(&state, layout);
        install(&mut open(&d), &package(10000), 1, 17).unwrap();
    }
    let seed = state.borrow().clone();
    for version in [false, true] {
        *state.borrow_mut() = seed.clone();
        {
            let mut s = state.borrow_mut();
            if version {
                s.bytes[7] = 2;
            } else {
                s.bytes[8..12].copy_from_slice(&8192_u32.to_le_bytes());
            }
            let hash = Sha256::digest(&s.bytes[..128]);
            s.bytes[128..160].copy_from_slice(&hash);
            s.reset();
        }
        let result = NorDevice::new(Model::<4, 4, 4096>(state.clone()), layout);
        assert!(
            matches!(result, Err(Error::Code(c)) if c == if version {Code::Format} else {Code::Layout})
        );
        assert_eq!(state.borrow().mutations(), 0);
    }
}
#[test]
fn raw_storage_bounds_poison_and_active_snapshot_rules() {
    let (state, layout) = blank();
    let d = device(&state, layout);
    let mut s = d.open_for_installation().unwrap();
    assert!(matches!(s.record(Slot::A).unwrap(), Record::Absent));
    assert!(s.prepare(Slot::A, usize::MAX).is_err());
    assert!(s.prepare(Slot::A, 63).is_err());
    s.prepare(Slot::A, 65).unwrap();
    assert!(s.snapshot(Slot::A).is_err());
    assert!(s.write(Slot::A, 0, &[]).is_err());
    assert!(s.write(Slot::A, 1, &[1]).is_err());
    assert!(s.write(Slot::A, usize::MAX, &[1]).is_err());
    assert!(s.read(Slot::A, 0, &mut [0]).is_err());
    state.borrow_mut().reset();
    state.borrow_mut().fault = Some((1, Mode::Partial));
    assert!(s.write(Slot::A, 0, &[1; 17]).is_err());
    state.borrow_mut().reset();
    assert!(s.write(Slot::A, 0, &[1; 17]).is_err());
    assert!(s.sync_payload(Slot::A).is_err());
    assert!(s.prepare(Slot::A, 65).is_err());
    assert_eq!(state.borrow().mutations(), 0);
    s.release();
    s.prepare(Slot::A, 65).unwrap();
    s.write(Slot::A, 0, &[1; 65]).unwrap();
    s.sync_payload(Slot::A).unwrap();
    s.sync_payload(Slot::A).unwrap();
    assert!(s.read(Slot::A, 65, &mut [0]).is_err());
    assert!(s.read(Slot::A, usize::MAX, &mut [0]).is_err());
    s.read(Slot::A, 65, &mut []).unwrap();
}
#[test]
fn complete_but_unacknowledged_seal_is_reconciled_without_reprogramming() {
    let (state, layout) = blank();
    let d = device(&state, layout);
    let mut i = open(&d);
    let bytes = package(12000);
    let t = i.transaction(1);
    i.begin(t, identity(&bytes)).unwrap();
    for (n, b) in bytes.chunks(17).enumerate() {
        i.write(t, n * 17, b).unwrap();
    }
    i.verify(t).unwrap();
    // Record the exact final seal write in an independent run, without guessing call counts.
    let saved = state.borrow().clone();
    state.borrow_mut().reset();
    i.commit(t).unwrap();
    let seal = state
        .borrow()
        .operations
        .iter()
        .position(|o| o.kind == Kind::Write && o.offset == 256)
        .unwrap()
        + 1;
    drop(i);
    drop(d);
    *state.borrow_mut() = saved;
    state.borrow_mut().reset();
    // Recreate staged state through a fresh transaction; the scratch is deliberately erased again.
    let d = device(&state, layout);
    let mut i = open(&d);
    let t = i.transaction(1);
    i.begin(t, identity(&bytes)).unwrap();
    for (n, b) in bytes.chunks(17).enumerate() {
        i.write(t, n * 17, b).unwrap();
    }
    i.verify(t).unwrap();
    state.borrow_mut().reset();
    state.borrow_mut().fault = Some((seal, Mode::After));
    assert!(i.commit(t).is_err());
    assert_eq!(i.progress().unwrap().phase, Phase::Uncertain);
    let writes = state.borrow().mutations();
    assert!(i.commit(t).is_err());
    state.borrow_mut().fault = None;
    i.reconcile().unwrap();
    assert_eq!(i.progress().unwrap().phase, Phase::Committed);
    assert_eq!(state.borrow().mutations(), writes);
    same_frames(&i.snapshot().unwrap(), &bytes);
}
fn fragment(bytes: &[u8]) -> Frame {
    let mut a = Assembler::new();
    for block in bytes.chunks(20) {
        a.push(block).unwrap();
    }
    a.take().unwrap()
}
#[test]
fn bounded_transfer_runs_on_the_same_nor_backend_and_replays_all_programs() {
    let (state, layout) = blank();
    let d = device(&state, layout);
    let mut service = Service::new(open(&d)).unwrap();
    let bytes = package(45000);
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    service
        .attach(AuthorizedLink {
            principal: [3; 16],
            session: [2; 16],
        })
        .unwrap();
    upload.connect([2; 16]).unwrap();
    for _ in 0..100 {
        let Some(request) = upload.outbound().unwrap().cloned() else {
            break;
        };
        let response = service.process(fragment(request.bytes()).bytes()).unwrap();
        // Loss of the application receipt must replay the response without programming a word twice.
        let duplicate = service.process(fragment(request.bytes()).bytes()).unwrap();
        assert_eq!(response, duplicate);
        upload.accept(fragment(response.bytes()).bytes()).unwrap();
    }
    assert!(matches!(upload.outcome(), Some(Outcome::Installed(_))));
    same_frames(&service.snapshot().unwrap(), &bytes);
}
#[test]
fn large_odd_blocks_cross_sectors_and_short_replacements_hide_old_tails() {
    use stagemaster_install::{Commit, Identity};
    let layout = Layout::new(16384).unwrap();
    let state = Rc::new(RefCell::new(State::blank(layout.total_bytes())));
    let d = NorDevice::new(Model::<16, 256, 4096>(state.clone()), layout).unwrap();
    let mut s = d.open_for_installation().unwrap();
    for (generation, length) in [(1, 8191), (2, 123), (3, 16384)] {
        let bytes: Vec<_> = (0..length)
            .map(|i| u8::try_from((i * 31) % 256).unwrap())
            .collect();
        s.prepare(Slot::A, length).unwrap();
        let mut offset = 0;
        for chunk in [17, 1024].into_iter().cycle() {
            if offset == length {
                break;
            }
            let end = (offset + chunk).min(length);
            s.write(Slot::A, offset, &bytes[offset..end]).unwrap();
            let start = offset.saturating_sub(19);
            let mut read = vec![0; end - start];
            s.read(Slot::A, start, &mut read).unwrap();
            assert_eq!(read, bytes[start..end]);
            offset = end;
        }
        s.sync_payload(Slot::A).unwrap();
        s.commit_record(Commit {
            slot: Slot::A,
            generation,
            identity: Identity {
                bytes: length,
                digest: [7; 32],
            },
        })
        .unwrap();
        s.release();
        let snapshot = s.snapshot(Slot::A).unwrap();
        let mut all = vec![0; length];
        snapshot.read_exact(0, &mut all).unwrap();
        assert_eq!(all, bytes);
        assert!(snapshot.read_exact(length, &mut [0]).is_err());
        drop(snapshot);
    }
}
#[test]
fn opening_a_driver_read_failure_never_reports_an_empty_device() {
    let (state, layout) = blank();
    {
        let d = device(&state, layout);
        install(&mut open(&d), &package(10000), 1, 17).unwrap();
    }
    state.borrow_mut().reset();
    state.borrow_mut().fault = Some((1, Mode::Partial));
    assert!(matches!(
        NorDevice::new(Model::<4, 4, 4096>(state.clone()), layout),
        Err(Error::Flash(Failure::Injected))
    ));
    assert_eq!(state.borrow().mutations(), 0);
}
