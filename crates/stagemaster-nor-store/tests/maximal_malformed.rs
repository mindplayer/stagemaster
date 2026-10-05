#[path = "../../../tools/package-acceptance/bounded_corpus.rs"]
mod corpus;
#[allow(dead_code)]
mod support;
use stagemaster_install::{Code, Error, Identity, Installer, Phase, Slot, SlotHealth};
use stagemaster_nor_store::Layout;
use stagemaster_package::MAX_PACKAGE_BYTES;
use std::{cell::RefCell, rc::Rc};
use support::{State, device, install, open, package};

fn verify_rejection_and_recovery(chunk: usize) {
    for case in corpus::cases() {
        let layout = Layout::new(MAX_PACKAGE_BYTES).unwrap();
        let state = Rc::new(RefCell::new(State::blank(layout.total_bytes())));
        let device = device(&state, layout);
        let mut installer = open(&device);
        let old = package(19000);
        let original = install(&mut installer, &old, 1, 333).unwrap();
        let snapshot = installer.snapshot().unwrap();
        let plan = snapshot.load(0).unwrap();
        let protected = state.borrow().bytes[..layout.payload_offset(Slot::B)].to_vec();
        let transaction = installer.transaction(2);
        let identity = Identity {
            bytes: case.bytes.len(),
            digest: case.bytes[32..64].try_into().unwrap(),
        };
        assert_eq!(
            installer.begin(transaction, identity).unwrap().phase,
            Phase::Receiving
        );
        let mut offset = 0;
        for block in case.bytes.chunks(chunk) {
            let p = installer.write(transaction, offset, block).unwrap();
            offset += block.len();
            assert_eq!(p.received, offset);
            assert_eq!(p.phase, Phase::Receiving);
        }
        assert!(
            matches!(installer.verify(transaction),Err(Error::Package(e)) if e==case.expected),
            "{}",
            case.name
        );
        assert_eq!(installer.progress().unwrap().phase, Phase::Failed);
        assert!(matches!(
            installer.commit(transaction),
            Err(Error::Code(Code::State))
        ));
        assert_eq!(installer.head(), Some(original));
        // B metadata is part of the prefix, so compare only A's metadata and complete payload.
        let a = layout.payload_offset(Slot::A);
        let b = layout.payload_offset(Slot::B);
        assert_eq!(&state.borrow().bytes[..4096], &protected[..4096]);
        assert_eq!(&state.borrow().bytes[a..b], &protected[a..b]);
        assert_eq!(snapshot.load(0).unwrap(), plan);
        assert_eq!(
            installer.cancel(transaction).unwrap().phase,
            Phase::Cancelled
        );
        drop(snapshot);
        drop(installer);
        drop(device);
        // Recreate the complete adapter as well as the Installer: no active-slot or length cache.
        let rebooted_device = support::device(&state, layout);
        let mutations = state.borrow().mutations();
        let (mut rebooted, report) =
            Installer::open(rebooted_device.open_for_installation().unwrap(), [2; 16]).unwrap();
        assert_eq!(state.borrow().mutations(), mutations);
        assert_eq!(report.selected, Some(original));
        assert!(matches!(report.slots[1], SlotHealth::Empty));
        assert_eq!(report.high_water, original.generation);
        let recovered_old = rebooted.snapshot().unwrap();
        assert_eq!(recovered_old.load(0).unwrap(), plan);
        let replacement = package(24000);
        let next = install(&mut rebooted, &replacement, 1, 333).unwrap();
        assert_eq!(next.slot, Slot::B);
        assert_eq!(next.generation, original.generation + 1);
        assert_eq!(recovered_old.load(0).unwrap(), plan);
        drop(recovered_old);
        drop(rebooted);
        drop(rebooted_device);
        let restored_device = support::device(&state, layout);
        let mutations = state.borrow().mutations();
        let (restored, report) =
            Installer::open(restored_device.open_read_only().unwrap(), [3; 16]).unwrap();
        assert_eq!(state.borrow().mutations(), mutations);
        assert_eq!(report.selected, Some(next));
        assert_eq!(restored.snapshot().unwrap().commit(), next);
    }
}
#[test]
fn complete_maximum_bad_packages_with_full_chunks_preserve_old_flash_and_recover() {
    verify_rejection_and_recovery(1024);
}
#[test]
fn complete_maximum_bad_packages_with_odd_chunks_preserve_old_flash_and_recover() {
    verify_rejection_and_recovery(997);
}
