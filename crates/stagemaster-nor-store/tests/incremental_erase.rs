//! Bound physical work per request, including the largest declared slot and final tail.
#[allow(dead_code)]
mod support;
use stagemaster_install::{Installer, Slot, Storage};
use stagemaster_nor_store::{Layout, NorDevice};
use std::{cell::RefCell, rc::Rc};
use support::{Kind, Mode, Model, State, device, identity, install, open, package};

fn max_slot<const W: usize, const E: usize>() {
    let length = 2 * 1024 * 1024;
    let layout = Layout::new(length).unwrap();
    let state = Rc::new(RefCell::new(State::blank(layout.total_bytes())));
    let base = layout.payload_offset(Slot::A);
    state.borrow_mut().bytes[base..base + length].fill(0);
    let device = NorDevice::new(Model::<4, W, E>(state.clone()), layout).unwrap();
    let mut storage = device.open_for_installation().unwrap();
    state.borrow_mut().reset();
    storage.prepare(Slot::A, length).unwrap();
    assert!(
        state
            .borrow()
            .operations
            .iter()
            .filter(|o| o.kind == Kind::Erase)
            .all(|o| o.offset < 4096)
    );
    assert!(
        state.borrow().bytes[base..base + length]
            .iter()
            .all(|b| *b == 0)
    );
    let mut cursor = 0;
    let mut erased = 0;
    for size in [1, 257, 1024, 17].into_iter().cycle() {
        if cursor == length {
            break;
        }
        let count = size.min(length - cursor);
        state.borrow_mut().reset();
        storage.write(Slot::A, cursor, &vec![0x69; count]).unwrap();
        cursor += count;
        let state = state.borrow();
        let sectors: Vec<_> = state
            .operations
            .iter()
            .filter(|o| o.kind == Kind::Erase)
            .collect();
        // Input plus at most one carried word; work never depends on whole package size.
        assert!(sectors.len() <= (1024 + W).div_ceil(E) + 1);
        for op in sectors {
            assert_eq!(op.offset, base + erased);
            assert_eq!(op.length, E);
            erased += E;
        }
        assert!(erased <= cursor.div_ceil(E) * E);
        assert!(
            state.bytes[base + erased..base + (erased + E).min(length)]
                .iter()
                .all(|b| *b == 0)
        );
    }
    storage.sync_payload(Slot::A).unwrap();
    assert_eq!(erased, length);
    assert!(
        state.borrow().bytes[base..base + length]
            .iter()
            .all(|b| *b == 0x69)
    );
}

#[test]
fn two_mib_preparation_never_erases_payload_and_streaming_erases_each_sector_once() {
    max_slot::<4, 4096>();
    max_slot::<256, 256>();
}

#[test]
fn buffered_tail_erases_only_when_sealed_and_cancel_does_not_erase_unwritten_sectors() {
    let layout = Layout::new(16384).unwrap();
    let state = Rc::new(RefCell::new(State::blank(layout.total_bytes())));
    let base = layout.payload_offset(Slot::A);
    state.borrow_mut().bytes[base..base + 16384].fill(0x12);
    let d = NorDevice::new(Model::<4, 256, 4096>(state.clone()), layout).unwrap();
    let mut s = d.open_for_installation().unwrap();
    s.prepare(Slot::A, 4097).unwrap();
    for offset in (0..4096).step_by(1024) {
        s.write(Slot::A, offset, &[0x69; 1024]).unwrap();
    }
    state.borrow_mut().reset();
    s.write(Slot::A, 4096, &[0x88]).unwrap();
    let mut tail = [0];
    s.read(Slot::A, 4096, &mut tail).unwrap();
    assert_eq!(tail, [0x88]);
    assert_eq!(state.borrow().mutations(), 0);
    s.sync_payload(Slot::A).unwrap();
    let mutations = state.borrow().mutations();
    s.sync_payload(Slot::A).unwrap();
    assert_eq!(state.borrow().mutations(), mutations);
    assert_eq!(state.borrow().bytes[base + 4096], 0x88);
    assert!(
        state.borrow().bytes[base + 4097..base + 8192]
            .iter()
            .all(|b| *b == 0xff)
    );
    s.release();
    assert!(
        state.borrow().bytes[base + 8192..base + 16384]
            .iter()
            .all(|b| *b == 0x12)
    );
}

#[test]
fn later_sector_faults_poison_the_stage_and_recovery_keeps_the_other_committed_slot() {
    let layout = Layout::new(8192).unwrap();
    let state = Rc::new(RefCell::new(State::blank(layout.total_bytes())));
    let current = package(20_000);
    {
        let d = device(&state, layout);
        let mut i = open(&d);
        install(&mut i, &package(10_000), 1, 1024).unwrap();
        install(&mut i, &current, 2, 1024).unwrap();
    }
    let base = layout.payload_offset(Slot::A);
    state.borrow_mut().bytes[base + 4096..base + 8192].fill(0);
    let seed = state.borrow().clone();
    let start = || {
        let d = device(&state, layout);
        let mut s = d.open_for_installation().unwrap();
        s.prepare(Slot::A, 4099).unwrap();
        for offset in (0..4096).step_by(1024) {
            s.write(Slot::A, offset, &[0x69; 1024]).unwrap();
        }
        s.write(Slot::A, 4096, &[0x88; 3]).unwrap();
        state.borrow_mut().reset();
        (d, s)
    };
    let boundaries = {
        let (_d, mut s) = start();
        s.sync_payload(Slot::A).unwrap();
        state.borrow().operations.len()
    };
    for mode in [Mode::Before, Mode::Partial, Mode::After, Mode::Silent] {
        for point in 1..=boundaries {
            *state.borrow_mut() = seed.clone();
            {
                let (_d, mut s) = start();
                state.borrow_mut().fault = Some((point, mode));
                assert!(
                    s.sync_payload(Slot::A).is_err(),
                    "point={point} mode={mode:?}"
                );
                state.borrow_mut().fault = None;
                assert!(s.sync_payload(Slot::A).is_err());
                assert!(s.read(Slot::A, 0, &mut [0]).is_err());
                assert!(s.write(Slot::A, 4099, &[1]).is_err());
            }
            state.borrow_mut().reset();
            assert_eq!(
                &state.borrow().bytes[layout.payload_offset(Slot::B)..],
                &seed.bytes[layout.payload_offset(Slot::B)..]
            );
            let d = device(&state, layout);
            let i = Installer::open(d.open_read_only().unwrap(), [2; 16])
                .unwrap()
                .0;
            assert_eq!(i.head().unwrap().identity, identity(&current));
            assert!(i.snapshot().is_ok());
        }
    }
}
