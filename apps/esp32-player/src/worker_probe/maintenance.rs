//! Compile-time local test of the same maintenance gate used by the firmware worker.
use stagemaster_install::Storage;
use stagemaster_install_worker::{Command, Epoch, Error, ManagedWorker};
use stagemaster_runtime::{Action, Code, Grant, Mode, Origin, PlaybackPolicy, Request};
use stagemaster_transfer::AuthorizedLink;

pub(super) fn verify<S: Storage, P: PlaybackPolicy>(worker: &mut ManagedWorker<S, P>)
where
    S::Error: core::fmt::Debug,
{
    let state = worker.finish_maintenance(super::now()).unwrap();
    assert_eq!(state.mode, Mode::Operation);
    assert!(state.bound_package.is_some());
    assert!(state.loaded.is_none() && state.selected.is_none() && state.instance.is_none());
    let programs = worker.catalog().len();
    assert!(programs > 0);
    assert!(worker.render(&mut [0x55; 512]).unwrap().is_none());
    // Even a trusted local test grant cannot bypass the runtime mode gate.
    let epoch = Epoch::new(u32::MAX).unwrap();
    let result = worker.process(
        Command::Open {
            epoch,
            link: AuthorizedLink {
                principal: [0x5a; 16],
                session: [0x63; 16],
            },
        },
        super::now(),
        || Some(epoch),
    );
    assert!(matches!(result.result, Err(Error::Maintenance(Code::Mode))));
    let lease = worker
        .acquire(
            Grant {
                principal: [0x5a; 16],
                origin: Origin::Panel,
                duration_ms: 60_000,
            },
            true,
            super::now(),
        )
        .unwrap();
    let request = Request {
        lease,
        serial: 1,
        expected_revision: worker.state().revision,
        action: Action::BeginMaintenance,
    };
    assert!(worker.submit(request, super::now()).unwrap().result.is_ok());
    let quiescence = worker.quiescence_request().unwrap();
    worker.confirm_quiescent(quiescence, super::now()).unwrap();
    assert_eq!(worker.state().mode, Mode::Maintenance);
    assert!(worker.catalog().is_empty());
    esp_println::println!(
        "INSTALL MAINTENANCE CYCLE PASS programs={} operation-write=denied selected=false output=false heap={}/{}",
        programs,
        esp_alloc::HEAP.used(),
        esp_alloc::HEAP.free()
    );
}
