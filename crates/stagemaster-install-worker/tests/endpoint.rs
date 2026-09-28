use stagemaster_install::Installer;
use stagemaster_install_store::FileStore;
use stagemaster_install_worker::{
    ChannelError as E, Command, Completion, Endpoint, Epoch, Phase, Reply, Worker,
};
use stagemaster_package::Archive;
use stagemaster_project::{Document, PackageSelection};
use stagemaster_transfer::{
    Action, Assembler, AuthorizedLink, Frame, Outcome, Request, Response, Upload,
};
use std::path::Path;

fn epoch(n: u32) -> Epoch {
    Epoch::new(n).unwrap()
}
fn link(n: u8) -> AuthorizedLink {
    AuthorizedLink {
        principal: [9; 16],
        session: [n; 16],
    }
}
fn root() -> tempfile::TempDir {
    tempfile::tempdir_in(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
fn worker(path: &Path) -> Worker<FileStore> {
    Worker::new(
        Installer::open(FileStore::open(path).unwrap(), [7; 16])
            .unwrap()
            .0,
    )
    .unwrap()
}
fn package() -> Vec<u8> {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    let document = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    document
        .build_package(
            &document
                .view()
                .scenes
                .iter()
                .map(|s| PackageSelection::Scene { id: s.id.clone() })
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .bytes
}
fn query(n: u8) -> Frame {
    Request {
        link: [n; 16],
        id: 1,
        action: Action::Status,
    }
    .encode()
    .unwrap()
}
fn opened(n: u8, payload: usize, w: &mut Worker<FileStore>) -> Endpoint {
    let (mut endpoint, command) = Endpoint::open(epoch(u32::from(n)), link(n), payload, 0).unwrap();
    let completion = w.process(command, || endpoint.live_epoch());
    assert!(endpoint.complete(completion, 0).unwrap());
    endpoint
}
fn receive(endpoint: &mut Endpoint, frame: &Frame, payload: usize, now: u64) -> Command {
    let mut command = None;
    for fragment in frame.bytes().chunks(payload) {
        assert!(command.is_none());
        command = endpoint.receive(fragment, now).unwrap();
    }
    command.unwrap()
}
fn reply(endpoint: &mut Endpoint, now: u64) -> Frame {
    let mut assembler = Assembler::new();
    while let Some(bytes) = endpoint.fragment(now).unwrap() {
        let same = bytes.to_vec();
        assert_eq!(endpoint.fragment(now).unwrap().unwrap(), same);
        let complete = assembler.push(&same).unwrap();
        assert_eq!(endpoint.sent(now).unwrap(), complete);
    }
    assert_eq!(endpoint.phase(), Phase::Receiving);
    assembler.take().unwrap()
}
fn roundtrip(
    endpoint: &mut Endpoint,
    w: &mut Worker<FileStore>,
    frame: &Frame,
    payload: usize,
) -> Frame {
    let command = receive(endpoint, frame, payload, 0);
    let completion = w.process(command, || endpoint.live_epoch());
    endpoint.complete(completion, 0).unwrap();
    reply(endpoint, 0)
}

#[test]
fn real_package_installs_and_recovers_through_ordered_fragments() {
    assert!(core::mem::size_of::<Endpoint>() < 3000);
    let bytes = package();
    for payload in [1, 20, 244, 1280] {
        let dir = root();
        let mut w = worker(dir.path());
        let mut endpoint = opened(1, payload, &mut w);
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        upload.connect([1; 16]).unwrap();
        while let Some(frame) = upload.outbound().unwrap().cloned() {
            upload
                .accept(roundtrip(&mut endpoint, &mut w, &frame, payload).bytes())
                .unwrap();
        }
        assert!(matches!(upload.outcome(), Some(Outcome::Installed(_))));
        let expected = w.snapshot().unwrap().commit();
        assert_eq!(
            expected.identity.digest,
            *Archive::open(bytes.as_slice()).unwrap().digest()
        );
        endpoint.close();
        w.observe(endpoint.live_epoch());
        drop(w);
        assert_eq!(worker(dir.path()).snapshot().unwrap().commit(), expected);
    }
}

#[test]
fn missing_open_backpressure_and_wrong_session_revoke_before_work() {
    let frame = query(1);
    let (mut endpoint, _) = Endpoint::open(epoch(1), link(1), 1280, 0).unwrap();
    assert!(matches!(endpoint.receive(frame.bytes(), 0), Err(E::State)));
    assert_eq!(endpoint.live_epoch(), None);
    for phase in [Phase::Working, Phase::Sending] {
        let dir = root();
        let mut w = worker(dir.path());
        let mut endpoint = opened(1, 1280, &mut w);
        let command = receive(&mut endpoint, &frame, 1280, 0);
        if phase == Phase::Sending {
            let completion = w.process(command, || endpoint.live_epoch());
            endpoint.complete(completion, 0).unwrap();
        }
        assert!(matches!(endpoint.receive(frame.bytes(), 0), Err(E::State)));
        assert_eq!(endpoint.live_epoch(), None);
        w.observe(endpoint.live_epoch());
    }
    let dir = root();
    let mut w = worker(dir.path());
    let mut endpoint = opened(1, 1280, &mut w);
    assert!(matches!(
        endpoint.receive(query(2).bytes(), 0),
        Err(E::Protocol(stagemaster_transfer::Error::Connection))
    ));
    assert_eq!(endpoint.live_epoch(), None);
}

#[test]
fn fragments_and_clock_failures_never_reset_a_half_message() {
    let frame = query(1);
    for bad in [vec![], vec![0; 21], vec![0; 20]] {
        let dir = root();
        let mut w = worker(dir.path());
        let mut endpoint = opened(1, 20, &mut w);
        assert!(endpoint.receive(&bad, 0).is_err());
        assert!(matches!(
            endpoint.receive(&frame.bytes()[..20], 1),
            Err(E::Closed)
        ));
    }
    let dir = root();
    let mut w = worker(dir.path());
    let mut endpoint = opened(1, 20, &mut w);
    assert!(
        endpoint
            .receive(&frame.bytes()[..3], 100)
            .unwrap()
            .is_none()
    );
    assert!(
        endpoint
            .receive(&frame.bytes()[3..5], 5099)
            .unwrap()
            .is_none()
    );
    assert_eq!(endpoint.poll(5100), Err(E::Timeout)); // slow fragments do not renew
    assert_eq!(endpoint.live_epoch(), None);
    let mut endpoint = opened(2, 20, &mut w);
    endpoint.poll(100).unwrap();
    assert_eq!(endpoint.poll(99), Err(E::Clock));
    assert!(matches!(
        Endpoint::open(epoch(3), link(3), 20, u64::MAX),
        Err(E::Clock)
    ));
    for payload in [0, 1281] {
        assert!(matches!(
            Endpoint::open(epoch(3), link(3), payload, 0),
            Err(E::Bounds)
        ));
    }
    assert!(matches!(
        Endpoint::open(
            epoch(3),
            AuthorizedLink {
                principal: [0; 16],
                session: [3; 16]
            },
            20,
            0
        ),
        Err(E::Protocol(_))
    ));
}

#[test]
fn opening_work_and_reply_deadlines_close_without_rolling_back_io() {
    let (mut endpoint, _) = Endpoint::open(epoch(1), link(1), 20, 0).unwrap();
    endpoint.poll(4999).unwrap();
    assert_eq!(endpoint.poll(5000), Err(E::Timeout));
    let dir = root();
    let mut w = worker(dir.path());
    let mut endpoint = opened(1, 20, &mut w);
    let command = receive(&mut endpoint, &query(1), 20, 10);
    endpoint.poll(30009).unwrap();
    assert_eq!(endpoint.poll(30010), Err(E::Timeout));
    let completion = w.process(command, || endpoint.live_epoch());
    assert!(matches!(
        completion.result,
        Err(stagemaster_install_worker::Error::Obsolete)
    ));
    let mut endpoint = opened(2, 20, &mut w);
    let command = receive(&mut endpoint, &query(2), 20, 10);
    let completion = w.process(command, || endpoint.live_epoch());
    endpoint.complete(completion, 11).unwrap();
    endpoint.fragment(5010).unwrap().unwrap();
    endpoint.sent(5010).unwrap();
    assert_eq!(endpoint.poll(5011), Err(E::Timeout)); // sends do not renew whole-reply budget
}

#[test]
fn old_epoch_completions_are_drained_but_current_mismatches_revoke() {
    for corruption in 0..4 {
        let dir = root();
        let mut w = worker(dir.path());
        let mut endpoint = opened(2, 1280, &mut w);
        assert!(
            !endpoint
                .complete(
                    Completion {
                        epoch: epoch(1),
                        result: Err(stagemaster_install_worker::Error::Obsolete)
                    },
                    0
                )
                .unwrap()
        );
        assert_eq!(endpoint.phase(), Phase::Receiving);
        let command = receive(&mut endpoint, &query(2), 1280, 0);
        let Completion { epoch, result } = w.process(command, || endpoint.live_epoch());
        let Reply::Frame(frame) = result.unwrap() else {
            panic!("response expected")
        };
        let mut response = Response::decode(frame.bytes()).unwrap();
        match corruption {
            0 => response.link = [1; 16],
            1 => response.id = 2,
            2 => response.command = stagemaster_transfer::Command::Cancel,
            _ => {}
        }
        let result = if corruption == 3 {
            Ok(Reply::Opened)
        } else {
            Ok(Reply::Frame(response.encode().unwrap()))
        };
        assert!(endpoint.complete(Completion { epoch, result }, 0).is_err());
        assert_eq!(endpoint.live_epoch(), None);
    }
}

#[test]
fn a_send_is_not_acknowledged_without_an_outstanding_fragment() {
    let dir = root();
    let mut w = worker(dir.path());
    let mut endpoint = opened(1, 20, &mut w);
    let command = receive(&mut endpoint, &query(1), 20, 0);
    let completion = w.process(command, || endpoint.live_epoch());
    endpoint.complete(completion, 0).unwrap();
    assert_eq!(endpoint.sent(0), Err(E::State));
    assert_eq!(endpoint.live_epoch(), None);
}

#[test]
fn interrupted_commit_reply_then_reconnected_cancel_reports_durable_installation() {
    let dir = root();
    let bytes = package();
    let mut w = worker(dir.path());
    let mut endpoint = opened(1, 20, &mut w);
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    upload.connect([1; 16]).unwrap();
    loop {
        let frame = upload.outbound().unwrap().unwrap().clone();
        if Request::decode(frame.bytes()).unwrap().action.command()
            == stagemaster_transfer::Command::Commit
        {
            let command = receive(&mut endpoint, &frame, 20, 0);
            let completion = w.process(command, || endpoint.live_epoch());
            endpoint.complete(completion, 0).unwrap();
            let mut old_half = Assembler::new();
            assert!(
                !old_half
                    .push(endpoint.fragment(0).unwrap().unwrap())
                    .unwrap()
            );
            assert!(!endpoint.sent(0).unwrap());
            endpoint.close();
            w.observe(endpoint.live_epoch());
            // Both halves of this connection are discarded, not appended to new frames.
            assert!(old_half.take().is_none());
            break;
        }
        upload
            .accept(roundtrip(&mut endpoint, &mut w, &frame, 20).bytes())
            .unwrap();
    }
    let committed = w.snapshot().unwrap().commit();
    upload.disconnect();
    upload.request_cancel();
    upload.connect([2; 16]).unwrap();
    endpoint = opened(2, 244, &mut w);
    while let Some(frame) = upload.outbound().unwrap().cloned() {
        upload
            .accept(roundtrip(&mut endpoint, &mut w, &frame, 244).bytes())
            .unwrap();
    }
    assert_eq!(upload.outcome(), Some(Outcome::Installed(committed)));
}
