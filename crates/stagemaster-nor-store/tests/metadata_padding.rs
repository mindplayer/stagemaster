//! Keep every metadata padding byte mandatory, even with a recomputed envelope hash.
#[allow(dead_code)]
mod support;
use sha2::{Digest, Sha256};
use stagemaster_install::{Record, Slot, Storage};
use stagemaster_nor_store::Layout;
use std::{cell::RefCell, rc::Rc};
use support::{State, device, install, open, package};

fn reject_each_padding_byte(indices: &(impl Iterator<Item = usize> + Clone)) {
    let layout = Layout::new(4096).unwrap();
    let state = Rc::new(RefCell::new(State::blank(layout.total_bytes())));
    let device = device(&state, layout);
    {
        let mut installer = open(&device);
        install(&mut installer, &package(10_000), 1, 1024).unwrap();
        install(&mut installer, &package(20_000), 2, 1024).unwrap();
    }
    let seed = state.borrow().bytes.clone();
    let storage = device.open_read_only().unwrap();
    for slot in [Slot::A, Slot::B] {
        assert!(matches!(storage.record(slot).unwrap(), Record::Bytes(_)));
        assert!(storage.snapshot(slot).is_ok());
        for index in indices.clone() {
            {
                let mut s = state.borrow_mut();
                s.bytes.copy_from_slice(&seed);
                let start = layout.metadata_offset(slot);
                s.bytes[start + index] ^= 1;
                // Matching hash must not excuse a noncanonical reserved field.
                let hash = Sha256::digest(&s.bytes[start..start + 128]);
                s.bytes[start + 128..start + 160].copy_from_slice(&hash);
            }
            assert!(
                matches!(storage.record(slot).unwrap(), Record::Invalid),
                "slot={slot:?} byte={index}"
            );
            assert!(storage.snapshot(slot).is_err());
            assert!(matches!(
                storage.record(slot.other()).unwrap(),
                Record::Bytes(_)
            ));
        }
        state.borrow_mut().bytes.copy_from_slice(&seed);
    }
}

#[test]
fn reserved_zeros_remain_required_with_an_authentic_envelope_digest() {
    reject_each_padding_byte(&(12..32));
}

#[test]
fn erased_padding_and_zero_seal_tail_remain_strict() {
    reject_each_padding_byte(&(160..256).chain(264..512));
}
