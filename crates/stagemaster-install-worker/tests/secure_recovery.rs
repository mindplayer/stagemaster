#![cfg(feature = "application")]
#[allow(dead_code)]
mod maintenance_support;
#[allow(dead_code)]
mod secure_support;
use maintenance_support::{Device, fixture};
use secure_support::Pair;
use stagemaster_device_session::Kind;
use stagemaster_transfer::{Action, Outcome, Request, Upload};

fn open(worker: &mut Device, epoch: u32) -> Pair {
    let (mut pair, command) = Pair::new(epoch, 600_000);
    let completion = worker.process(command, 0, || pair.gateway.live_epoch(0));
    pair.gateway.complete(completion, 0).unwrap();
    pair.ready();
    pair
}

#[test]
fn interrupted_write_and_lost_commit_response_reconcile_with_fresh_key_proof() {
    for at_commit in [false, true] {
        let (_dir, mut worker, metrics, bytes) = fixture();
        worker
            .confirm_quiescent(worker.quiescence_request().unwrap(), 0)
            .unwrap();
        let mut pair = open(&mut worker, 1);
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        upload.connect(pair.session).unwrap();
        loop {
            let request = upload.outbound().unwrap().unwrap().clone();
            let action = Request::decode(request.bytes()).unwrap().action;
            let lose = matches!(
                (at_commit, action),
                (false, Action::Write { .. }) | (true, Action::Commit(_))
            );
            let command = pair.send(Kind::Message, request.bytes(), 0).unwrap();
            let completion = worker.process(command, 0, || pair.gateway.live_epoch(0));
            if lose {
                pair.gateway.close();
                // Late response cannot be emitted or consumed by a later connection.
                assert!(pair.gateway.complete(completion, 0).is_err());
                break;
            }
            pair.gateway.complete(completion, 0).unwrap();
            upload.accept(&pair.receive(Kind::Message, 0)).unwrap();
        }
        let old_session = pair.session;
        worker.observe(pair.gateway.live_epoch(0));
        upload.disconnect();
        let writes = metrics.mutations.get();
        pair = open(&mut worker, 2);
        assert_ne!(pair.session, old_session);
        upload.connect(pair.session).unwrap();
        while let Some(request) = upload.outbound().unwrap().cloned() {
            let command = pair.send(Kind::Message, request.bytes(), 0).unwrap();
            let completion = worker.process(command, 0, || pair.gateway.live_epoch(0));
            pair.gateway.complete(completion, 0).unwrap();
            upload.accept(&pair.receive(Kind::Message, 0)).unwrap();
        }
        assert!(matches!(upload.outcome(), Some(Outcome::Installed(_))));
        if at_commit {
            assert_eq!(
                metrics.mutations.get(),
                writes,
                "committed package must not be rewritten"
            );
        }
    }
}

#[test]
fn authenticated_cancellation_preserves_the_previous_installed_package() {
    let (_dir, mut worker, _metrics, bytes) = fixture();
    maintenance_support::install(&mut worker, &bytes);
    let previous = worker.snapshot().unwrap().commit();
    worker.observe(None);
    // Change a valid package using another supported export selection.
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    let document =
        stagemaster_project::Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    let next = document
        .build_package(&[stagemaster_project::PackageSelection::Scene {
            id: document.view().scenes[0].id.clone(),
        }])
        .unwrap()
        .bytes;
    let mut pair = open(&mut worker, 2);
    let mut upload = Upload::new(next.as_slice()).unwrap();
    upload.connect(pair.session).unwrap();
    let mut requested = false;
    while let Some(request) = upload.outbound().unwrap().cloned() {
        let written = matches!(
            Request::decode(request.bytes()).unwrap().action,
            Action::Write { .. }
        );
        let command = pair.send(Kind::Message, request.bytes(), 0).unwrap();
        let completion = worker.process(command, 0, || pair.gateway.live_epoch(0));
        pair.gateway.complete(completion, 0).unwrap();
        upload.accept(&pair.receive(Kind::Message, 0)).unwrap();
        if written {
            upload.request_cancel();
            requested = true;
        }
    }
    assert!(requested);
    assert!(matches!(upload.outcome(), Some(Outcome::Cancelled)));
    assert_eq!(worker.snapshot().unwrap().commit(), previous);
}
