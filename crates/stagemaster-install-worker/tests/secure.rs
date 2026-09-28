#![cfg(feature = "application")]
#[allow(dead_code)] // Shared maintenance fixture also supports operation-mode tests.
mod maintenance_support;
mod secure_support;
use secure_support::Pair;
use stagemaster_device_session::Kind;
use stagemaster_install::Installer;
use stagemaster_install_store::FileStore;
use stagemaster_install_worker::{Completion, Epoch, Reply, secure::Error};
use stagemaster_transfer::{Outcome, Upload};

#[test]
fn real_encrypted_installation_preserves_maintenance_and_recovers_from_disk() {
    let (dir, mut worker, metrics, bytes) = maintenance_support::fixture();
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), 0)
        .unwrap();
    let (mut pair, open) = Pair::new(1, 600_000);
    assert!(pair.gateway.outbound(0).unwrap().is_none());
    let completion = worker.process(open, 0, || pair.gateway.live_epoch(0));
    pair.gateway.complete(completion, 0).unwrap();
    pair.ready();
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    upload.connect(pair.session).unwrap();
    while let Some(frame) = upload.outbound().unwrap().cloned() {
        let command = pair.send(Kind::Message, frame.bytes(), 0).unwrap();
        // Heartbeat remains available while actual storage work is queued.
        assert!(pair.send(Kind::Heartbeat, &[], 0).is_none());
        assert!(pair.receive(Kind::HeartbeatReply, 0).is_empty());
        let completion = worker.process(command, 0, || pair.gateway.live_epoch(0));
        pair.gateway.complete(completion, 0).unwrap();
        upload.accept(&pair.receive(Kind::Message, 0)).unwrap();
    }
    let Some(Outcome::Installed(installed)) = upload.outcome() else {
        panic!("not committed")
    };
    assert!(metrics.mutations.get() > 0);
    let digest = installed.identity.digest;
    pair.gateway.close();
    worker.observe(pair.gateway.live_epoch(0));
    let state = worker.finish_maintenance(0).unwrap();
    assert!(state.instance.is_none());
    drop(worker);
    let (recovered, _) = Installer::open(FileStore::open(dir.path()).unwrap(), [8; 16]).unwrap();
    assert_eq!(
        recovered.snapshot().unwrap().commit().identity.digest,
        digest
    );
}

#[test]
fn worker_refusal_and_early_requests_never_publish_readiness() {
    let (_dir, mut worker, metrics, _bytes) = maintenance_support::fixture();
    let (mut pair, open) = Pair::new(1, 600_000);
    let completion = worker.process(open, 0, || pair.gateway.live_epoch(0));
    assert!(pair.gateway.complete(completion, 0).is_err());
    assert_eq!(pair.gateway.live_epoch(0), None);
    assert_eq!(metrics.mutations.get(), 0);
    let (mut pair, _) = Pair::new(2, 600_000);
    let query = pair.query();
    let cipher = pair.cipher(Kind::Message, query.bytes(), 0);
    assert_eq!(pair.gateway.receive(&cipher, 0).err(), Some(Error::Order));
    assert_eq!(pair.gateway.live_epoch(0), None);
}

#[test]
fn unsent_ready_receipt_and_partial_business_messages_cannot_dispatch() {
    for partial in [false, true] {
        let (mut pair, _) = Pair::new(1, 600_000);
        if partial {
            pair.mark_opened(1);
        } else {
            pair.gateway
                .complete(
                    Completion {
                        epoch: Epoch::new(1).unwrap(),
                        result: Ok(Reply::Opened),
                    },
                    0,
                )
                .unwrap();
            assert!(pair.gateway.outbound(0).unwrap().is_some());
        }
        let query = pair.query();
        let bytes = if partial {
            &query.bytes()[..3]
        } else {
            query.bytes()
        };
        let cipher = pair.cipher(Kind::Message, bytes, 0);
        assert!(pair.gateway.receive(&cipher, 0).is_err());
        assert_eq!(pair.gateway.live_epoch(0), None);
    }
}

#[test]
fn late_completion_and_heartbeat_do_not_revive_expired_or_revoked_gateway() {
    for revoke in [false, true] {
        let (mut pair, _) = Pair::new(1, 100);
        pair.mark_opened(1);
        if revoke {
            pair.gateway.close();
        }
        let now = if revoke { 1 } else { 100 };
        assert_eq!(pair.gateway.live_epoch(now), None);
        let cipher = pair.cipher(Kind::Heartbeat, &[], now);
        assert!(pair.gateway.receive(&cipher, now).is_err());
        assert!(
            pair.gateway
                .complete(
                    Completion {
                        epoch: Epoch::new(1).unwrap(),
                        result: Ok(Reply::Opened)
                    },
                    now
                )
                .is_err()
        );
        assert!(pair.gateway.outbound(now).is_err());
    }
}

#[test]
fn obsolete_completions_are_ignored_without_publishing_a_new_receipt() {
    let (mut pair, _) = Pair::new(2, 600_000);
    assert!(
        !pair
            .gateway
            .complete(
                Completion {
                    epoch: Epoch::new(1).unwrap(),
                    result: Ok(Reply::Opened)
                },
                0
            )
            .unwrap()
    );
    assert!(pair.gateway.outbound(0).unwrap().is_none());
    pair.mark_opened(2);
}

#[test]
fn ciphertext_replay_and_unbounded_heartbeat_queue_fail_closed() {
    for replay in [true, false] {
        let (mut pair, _) = Pair::new(1, 600_000);
        pair.mark_opened(1);
        let cipher = pair.cipher(Kind::Heartbeat, &[], 0);
        assert!(pair.gateway.receive(&cipher, 0).unwrap().is_none());
        let second = if replay {
            cipher
        } else {
            pair.cipher(Kind::Heartbeat, &[], 0)
        };
        assert!(pair.gateway.receive(&second, 0).is_err());
        assert_eq!(pair.gateway.live_epoch(0), None);
    }
}

#[test]
fn revoked_queued_work_is_rejected_by_actual_worker_before_storage() {
    let (_dir, mut worker, metrics, _bytes) = maintenance_support::fixture();
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), 0)
        .unwrap();
    let (mut pair, open) = Pair::new(1, 600_000);
    pair.gateway.close();
    let completion = worker.process(open, 0, || pair.gateway.live_epoch(0));
    assert!(matches!(
        completion.result,
        Err(stagemaster_install_worker::Error::Obsolete)
    ));
    assert_eq!(metrics.mutations.get(), 0);
}
