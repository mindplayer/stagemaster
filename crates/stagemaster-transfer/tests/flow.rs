use stagemaster_install::{Commit, Identity, Installer, Phase, Record, Slot, Storage};
use stagemaster_install_store::{FileSnapshot, FileStore};
use stagemaster_package::{Archive, ReadAt};
use stagemaster_playback::Player;
use stagemaster_project::{Document, PackageSelection};
use stagemaster_transfer::{
    Action, Assembler, AuthorizedLink, Command, Error, Frame, Outcome, RemoteError, Request,
    Response, Service, Upload, UploadError,
};
use std::{
    cell::RefCell,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

fn id(number: u64) -> [u8; 16] {
    let mut id = [0; 16];
    id[8..].copy_from_slice(&number.to_be_bytes());
    id
}
fn principal() -> [u8; 16] {
    [9; 16]
}
fn dir() -> tempfile::TempDir {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    std::fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}
fn package(level: u16) -> Vec<u8> {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    json["lighting"]["scenes"][0]["assignments"][0]["source"]["value"]["value"] = level.into();
    let first = json["lighting"]["scenes"][0].clone();
    for n in 0..20 {
        let mut scene = first.clone();
        scene["id"] = format!("00000000-0000-4000-8000-{:012}", 100 + n).into();
        scene["name"] = format!("传输验证场景 {n}").into();
        json["lighting"]["scenes"]
            .as_array_mut()
            .unwrap()
            .push(scene);
    }
    let doc = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    let view = doc.view();
    let items: Vec<_> = view
        .scenes
        .iter()
        .map(|s| PackageSelection::Scene { id: s.id.clone() })
        .chain(
            view.sequences
                .iter()
                .map(|s| PackageSelection::Sequence { id: s.id.clone() }),
        )
        .collect();
    let bytes = doc.build_package(&items).unwrap().bytes;
    assert!(bytes.len() > 2048);
    bytes
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum CommitFailure {
    Before,
    After,
}
#[derive(Default)]
struct Fault {
    writes: usize,
    commits: usize,
    partial_write: bool,
    commit_failure: Option<CommitFailure>,
    settle: bool,
}
struct Store {
    inner: FileStore,
    fault: Rc<RefCell<Fault>>,
}
impl Storage for Store {
    type Error = io::Error;
    type Snapshot = FileSnapshot;
    fn capacity(&self, slot: Slot) -> usize {
        self.inner.capacity(slot)
    }
    fn record(&self, slot: Slot) -> io::Result<Record> {
        self.inner.record(slot)
    }
    fn slot_len(&self, slot: Slot) -> io::Result<usize> {
        self.inner.slot_len(slot)
    }
    fn read(&self, slot: Slot, offset: usize, bytes: &mut [u8]) -> io::Result<()> {
        self.inner.read(slot, offset, bytes)
    }
    fn prepare(&mut self, slot: Slot, bytes: usize) -> io::Result<()> {
        self.inner.prepare(slot, bytes)
    }
    fn write(&mut self, slot: Slot, offset: usize, bytes: &[u8]) -> io::Result<()> {
        self.fault.borrow_mut().writes += 1;
        if self.fault.borrow().partial_write {
            self.inner
                .write(slot, offset, &bytes[..bytes.len().div_ceil(2)])?;
            return Err(io::Error::other("注入部分写入失败"));
        }
        self.inner.write(slot, offset, bytes)
    }
    fn sync_payload(&mut self, slot: Slot) -> io::Result<()> {
        self.inner.sync_payload(slot)
    }
    fn commit_record(&mut self, commit: Commit) -> io::Result<()> {
        self.fault.borrow_mut().commits += 1;
        if self.fault.borrow().commit_failure == Some(CommitFailure::Before) {
            return Err(io::Error::other("注入提交前失败"));
        }
        self.inner.commit_record(commit)?;
        if self.fault.borrow().commit_failure == Some(CommitFailure::After) {
            return Err(io::Error::other("注入提交回执丢失"));
        }
        Ok(())
    }
    fn settle(&mut self) -> io::Result<()> {
        if self.fault.borrow().settle {
            return Err(io::Error::other("注入对账失败"));
        }
        self.inner.settle()
    }
    fn release(&mut self) {
        self.inner.release();
    }
    fn snapshot(&self, slot: Slot) -> io::Result<FileSnapshot> {
        self.inner.snapshot(slot)
    }
}
type Server = Service<Store>;
fn open(path: &Path, boot: u8) -> (Server, Rc<RefCell<Fault>>) {
    let fault = Rc::new(RefCell::new(Fault::default()));
    let store = Store {
        inner: FileStore::open(path).unwrap(),
        fault: fault.clone(),
    };
    (
        Service::new(Installer::open(store, [boot; 16]).unwrap().0).unwrap(),
        fault,
    )
}
fn connect<R: ReadAt>(server: &mut Server, upload: &mut Upload<R>, session: u64) {
    server.detach();
    server
        .attach(AuthorizedLink {
            principal: principal(),
            session: id(session),
        })
        .unwrap();
    upload.connect(id(session)).unwrap();
}
fn fragments(bytes: &[u8], mtu: usize) -> Frame {
    let mut assembler = Assembler::new();
    for piece in bytes.chunks(mtu) {
        assembler.push(piece).unwrap();
    }
    assembler.take().unwrap()
}
fn exchange<R: ReadAt>(
    server: &mut Server,
    upload: &mut Upload<R>,
    mtu: usize,
) -> Result<Option<Command>, UploadError> {
    let Some(frame) = upload.outbound()?.cloned() else {
        return Ok(None);
    };
    let command = Request::decode(frame.bytes()).unwrap().action.command();
    let response = server
        .process(fragments(frame.bytes(), mtu).bytes())
        .unwrap();
    upload.accept(fragments(response.bytes(), mtu).bytes())?;
    Ok(Some(command))
}
fn finish<R: ReadAt>(server: &mut Server, upload: &mut Upload<R>, mtu: usize) -> usize {
    for count in 0..100 {
        if exchange(server, upload, mtu).unwrap().is_none() {
            return count;
        }
    }
    panic!("上传未收敛")
}
fn same_frames(server: &Server, bytes: &[u8]) {
    let snapshot = server.snapshot().unwrap();
    let archive = Archive::open(bytes).unwrap();
    assert_eq!(snapshot.archive().entries(), archive.entries());
    for index in 0..archive.entries().len() {
        let expected = archive.load(bytes, index).unwrap();
        let actual = snapshot.load(index).unwrap();
        let mut a = Player::new(actual.plan, 0);
        let mut b = Player::new(expected.plan, 0);
        assert_eq!(a.values(), b.values());
        a.execute(0, 0).unwrap();
        b.execute(0, 0).unwrap();
        for time in (0..10_000).step_by(25) {
            a.advance(time).unwrap();
            b.advance(time).unwrap();
            let mut left = [0; 512];
            let mut right = [0; 512];
            actual.output.render(a.values(), &mut left).unwrap();
            expected.output.render(b.values(), &mut right).unwrap();
            assert_eq!(left, right);
        }
    }
}

#[test]
fn small_mtu_transfer_replays_identically_and_duplicate_install_never_writes() {
    let bytes = package(10000);
    for mtu in [20, 185, 244, 512] {
        let dir = dir();
        let (mut server, fault) = open(dir.path(), 1);
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        connect(&mut server, &mut upload, 1);
        let requests = finish(&mut server, &mut upload, mtu);
        assert_eq!(requests, 4 + bytes.len().div_ceil(1024));
        assert!(matches!(upload.outcome(), Some(Outcome::Installed(c)) if c.generation == 1));
        same_frames(&server, &bytes);
        let writes = fault.borrow().writes;
        let mut duplicate = Upload::new(bytes.as_slice()).unwrap();
        connect(&mut server, &mut duplicate, 2);
        assert_eq!(finish(&mut server, &mut duplicate, mtu), 1);
        assert_eq!(fault.borrow().writes, writes);
        assert_eq!(fault.borrow().commits, 1);
    }
}

#[test]
fn each_lost_application_response_retries_once_or_reconnects_without_rewriting() {
    let bytes = package(20000);
    let total = 4 + bytes.len().div_ceil(1024);
    for reconnect in [false, true] {
        for lost in 0..total {
            let dir = dir();
            let (mut server, fault) = open(dir.path(), 1);
            let mut upload = Upload::new(bytes.as_slice()).unwrap();
            connect(&mut server, &mut upload, 1);
            for _ in 0..lost {
                exchange(&mut server, &mut upload, 20).unwrap();
            }
            let request = upload.outbound().unwrap().unwrap().clone();
            let original = server.process(request.bytes()).unwrap(); // Entire application reply is lost.
            if reconnect {
                connect(&mut server, &mut upload, 2);
            } else {
                assert_eq!(upload.outbound().unwrap().unwrap(), &request);
                let before = (fault.borrow().writes, fault.borrow().commits);
                let reply = server.process(request.bytes()).unwrap();
                assert_eq!(reply, original);
                assert_eq!((fault.borrow().writes, fault.borrow().commits), before);
                upload.accept(reply.bytes()).unwrap();
            }
            finish(&mut server, &mut upload, 20);
            assert_eq!(fault.borrow().writes, bytes.len().div_ceil(1024));
            assert_eq!(fault.borrow().commits, 1);
            same_frames(&server, &bytes);
        }
    }
}

#[test]
fn partial_fragments_and_late_old_responses_are_discarded_on_reconnect() {
    let bytes = package(10000);
    for cut in [1, 7, 8, 19, 20, 600] {
        let dir = dir();
        let (mut server, fault) = open(dir.path(), 1);
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        connect(&mut server, &mut upload, 1);
        exchange(&mut server, &mut upload, 20).unwrap();
        exchange(&mut server, &mut upload, 20).unwrap();
        let old = upload.outbound().unwrap().unwrap().clone();
        let mut partial = Assembler::new();
        assert!(!partial.push(&old.bytes()[..cut]).unwrap());
        assert!(partial.take().is_none());
        connect(&mut server, &mut upload, 2); // Transport drops partial assembler and old notification queue.
        let query = upload.outbound().unwrap().unwrap().clone();
        let reply = server.process(query.bytes()).unwrap();
        let mut stale = Response::decode(reply.bytes()).unwrap();
        stale.link = id(1);
        assert!(upload.accept(stale.encode().unwrap().bytes()).is_err());
        assert_eq!(upload.outbound().unwrap().unwrap(), &query);
        upload.accept(reply.bytes()).unwrap();
        finish(&mut server, &mut upload, 20);
        assert_eq!(fault.borrow().writes, bytes.len().div_ceil(1024));
        same_frames(&server, &bytes);
    }
}

#[test]
fn device_restart_at_every_request_boundary_resumes_only_committed_data() {
    let bytes = package(30000);
    let total = 4 + bytes.len().div_ceil(1024);
    for boundary in 0..=total {
        let dir = dir();
        let (mut server, _) = open(dir.path(), 1);
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        connect(&mut server, &mut upload, 1);
        for _ in 0..boundary {
            exchange(&mut server, &mut upload, 20).unwrap();
        }
        drop(server);
        let (mut server, _) = open(dir.path(), 2);
        connect(&mut server, &mut upload, 2);
        finish(&mut server, &mut upload, 20);
        assert!(matches!(upload.outcome(),Some(Outcome::Installed(c)) if c.generation==1));
        same_frames(&server, &bytes);
    }
}

#[test]
fn cancelled_during_each_pending_operation_and_commit_wins_are_reported_truthfully() {
    let bytes = package(10000);
    let total = 4 + bytes.len().div_ceil(1024);
    for pending in 0..total {
        let dir = dir();
        let (mut server, _) = open(dir.path(), 1);
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        connect(&mut server, &mut upload, 1);
        for _ in 0..pending {
            exchange(&mut server, &mut upload, 20).unwrap();
        }
        let frame = upload.outbound().unwrap().unwrap().clone();
        let command = Request::decode(frame.bytes()).unwrap().action.command();
        upload.request_cancel();
        upload
            .accept(server.process(frame.bytes()).unwrap().bytes())
            .unwrap();
        finish(&mut server, &mut upload, 20);
        if command == Command::Commit {
            assert!(matches!(upload.outcome(), Some(Outcome::Installed(_))));
        } else {
            assert!(matches!(
                upload.outcome(),
                Some(Outcome::Cancelled | Outcome::NotStarted)
            ));
            assert!(server.snapshot().is_err());
        }
    }
}

#[test]
fn denied_connections_and_principal_ownership_protect_unfinished_installs() {
    let bytes = package(10000);
    let dir = dir();
    let (mut server, _) = open(dir.path(), 1);
    let request = Request {
        link: id(1),
        id: 1,
        action: Action::Status,
    }
    .encode()
    .unwrap();
    assert_eq!(server.process(request.bytes()), Err(Error::Denied));
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    connect(&mut server, &mut upload, 1);
    assert_eq!(
        server.attach(AuthorizedLink {
            principal: principal(),
            session: id(2)
        }),
        Err(Error::Busy)
    );
    exchange(&mut server, &mut upload, 20).unwrap();
    exchange(&mut server, &mut upload, 20).unwrap();
    let transaction = upload.state().unwrap().progress.unwrap().transaction;
    server.detach();
    server
        .attach(AuthorizedLink {
            principal: [8; 16],
            session: id(2),
        })
        .unwrap();
    for (index, action) in [
        Action::Cancel(transaction),
        Action::Write {
            transaction,
            offset: 0,
            bytes: &bytes[..20],
        },
        Action::Begin {
            transaction,
            identity: upload.identity(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let frame = Request {
            link: id(2),
            id: index as u64 + 1,
            action,
        }
        .encode()
        .unwrap();
        let response = Response::decode(server.process(frame.bytes()).unwrap().bytes()).unwrap();
        assert_eq!(response.result, Err(RemoteError::Ownership));
        assert!(!response.state.owned);
    }
    connect(&mut server, &mut upload, 3);
    finish(&mut server, &mut upload, 20);
    same_frames(&server, &bytes);
}

#[test]
fn full_request_ids_protect_header_counter_wrap_and_conflicting_duplicates() {
    let bytes = package(10000);
    let dir = dir();
    let (mut server, _) = open(dir.path(), 1);
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    connect(&mut server, &mut upload, 1);
    finish(&mut server, &mut upload, 20);
    let transaction = upload.state().unwrap().progress.unwrap().transaction;
    server.detach();
    server
        .attach(AuthorizedLink {
            principal: principal(),
            session: id(4),
        })
        .unwrap();
    for n in 1..=257 {
        let frame = Request {
            link: id(4),
            id: n,
            action: Action::Status,
        }
        .encode()
        .unwrap();
        let response = Response::decode(server.process(frame.bytes()).unwrap().bytes()).unwrap();
        assert_eq!(response.id, n);
    }
    let stale = Request {
        link: id(4),
        id: 1,
        action: Action::Status,
    }
    .encode()
    .unwrap();
    assert_eq!(server.process(stale.bytes()), Err(Error::Sequence));
    assert_eq!(server.process(stale.bytes()), Err(Error::Denied));
    server
        .attach(AuthorizedLink {
            principal: principal(),
            session: id(5),
        })
        .unwrap();
    let original = Request {
        link: id(5),
        id: 1,
        action: Action::Status,
    }
    .encode()
    .unwrap();
    server.process(original.bytes()).unwrap();
    let conflict = Request {
        link: id(5),
        id: 1,
        action: Action::Cancel(transaction),
    }
    .encode()
    .unwrap();
    assert_eq!(server.process(conflict.bytes()), Err(Error::Sequence));
    same_frames(&server, &bytes);
}

#[test]
fn uncertain_commit_is_reconciled_and_failed_storage_requires_explicit_cancel() {
    let bytes = package(10000);
    for after in [false, true] {
        let dir = dir();
        let (mut server, fault) = open(dir.path(), 1);
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        connect(&mut server, &mut upload, 1);
        loop {
            let frame = upload.outbound().unwrap().unwrap().clone();
            if Request::decode(frame.bytes()).unwrap().action.command() == Command::Commit {
                break;
            }
            upload
                .accept(server.process(frame.bytes()).unwrap().bytes())
                .unwrap();
        }
        fault.borrow_mut().commit_failure = Some(if after {
            CommitFailure::After
        } else {
            CommitFailure::Before
        });
        exchange(&mut server, &mut upload, 20).unwrap();
        assert_eq!(
            upload.state().unwrap().progress.unwrap().phase,
            Phase::Uncertain
        );
        fault.borrow_mut().settle = true;
        assert!(matches!(
            exchange(&mut server, &mut upload, 20),
            Err(UploadError::Remote(RemoteError::Storage))
        ));
        assert_eq!(
            upload.state().unwrap().progress.unwrap().phase,
            Phase::Uncertain
        );
        fault.borrow_mut().settle = false;
        upload.retry().unwrap();
        exchange(&mut server, &mut upload, 20).unwrap();
        exchange(&mut server, &mut upload, 20).unwrap();
        if after {
            finish(&mut server, &mut upload, 20);
            same_frames(&server, &bytes);
        } else {
            assert!(matches!(
                upload.outbound(),
                Err(UploadError::Remote(RemoteError::State))
            ));
            upload.request_cancel();
            finish(&mut server, &mut upload, 20);
            assert_eq!(upload.outcome(), Some(Outcome::Cancelled));
            assert!(server.snapshot().is_err());
        }
        assert_eq!(fault.borrow().commits, 1);
    }
    let dir = dir();
    let (mut server, fault) = open(dir.path(), 1);
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    connect(&mut server, &mut upload, 1);
    exchange(&mut server, &mut upload, 20).unwrap();
    exchange(&mut server, &mut upload, 20).unwrap();
    fault.borrow_mut().partial_write = true;
    assert!(matches!(
        exchange(&mut server, &mut upload, 20),
        Err(UploadError::Remote(RemoteError::Storage))
    ));
    assert_eq!(upload.state().unwrap().progress.unwrap().received, 0);
    assert!(upload.outbound().is_err());
    upload.request_cancel();
    finish(&mut server, &mut upload, 20);
    fault.borrow_mut().partial_write = false;
    let mut retry = Upload::new(bytes.as_slice()).unwrap();
    connect(&mut server, &mut retry, 2);
    finish(&mut server, &mut retry, 20);
    same_frames(&server, &bytes);
}

#[test]
fn incompatible_progress_and_forged_success_do_not_advance_the_host() {
    let a = package(10000);
    let b = package(20000);
    let dir = dir();
    let (mut server, _) = open(dir.path(), 1);
    let mut upload = Upload::new(a.as_slice()).unwrap();
    connect(&mut server, &mut upload, 1);
    exchange(&mut server, &mut upload, 20).unwrap();
    exchange(&mut server, &mut upload, 20).unwrap();
    let frame = upload.outbound().unwrap().unwrap().clone();
    let request = Request::decode(frame.bytes()).unwrap();
    let reply = Response {
        link: request.link,
        id: request.id,
        command: request.action.command(),
        result: Ok(()),
        state: upload.state().unwrap(),
    }
    .encode()
    .unwrap();
    assert!(upload.accept(reply.bytes()).is_err()); // WRITE success but no acknowledged bytes.
    assert_eq!(upload.outbound().unwrap().unwrap(), &frame);
    let mut other = Upload::new(b.as_slice()).unwrap();
    connect(&mut server, &mut other, 2);
    exchange(&mut server, &mut other, 20).unwrap();
    assert!(matches!(
        other.outbound(),
        Err(UploadError::Remote(RemoteError::Conflict))
    ));
    other.request_cancel();
    exchange(&mut server, &mut other, 20).unwrap();
    assert!(matches!(
        other.outbound(),
        Err(UploadError::Remote(RemoteError::Conflict))
    ));
    connect(&mut server, &mut upload, 3);
    finish(&mut server, &mut upload, 20);
    same_frames(&server, &a);
}

#[test]
fn corrupted_source_after_validation_cannot_replace_an_installed_package() {
    #[derive(Clone)]
    struct Mutable(Rc<RefCell<Vec<u8>>>);
    impl ReadAt for Mutable {
        fn len(&self) -> usize {
            self.0.borrow().len()
        }
        fn read_exact(
            &self,
            offset: usize,
            target: &mut [u8],
        ) -> Result<(), stagemaster_package::Error> {
            target.copy_from_slice(&self.0.borrow()[offset..offset + target.len()]);
            Ok(())
        }
    }
    let a = package(10000);
    let b = package(20000);
    let dir = dir();
    let (mut server, _) = open(dir.path(), 1);
    let mut first = Upload::new(a.as_slice()).unwrap();
    connect(&mut server, &mut first, 1);
    finish(&mut server, &mut first, 20);
    let source = Mutable(Rc::new(RefCell::new(b)));
    let mut upload = Upload::new(source.clone()).unwrap();
    let last = source.len() - 1;
    source.0.borrow_mut()[last] ^= 1;
    connect(&mut server, &mut upload, 2);
    loop {
        match exchange(&mut server, &mut upload, 20) {
            Ok(Some(_)) => {}
            Err(UploadError::Remote(RemoteError::Package)) => break,
            result => panic!("意外结果：{result:?}"),
        }
    }
    assert_eq!(
        upload.state().unwrap().progress.unwrap().phase,
        Phase::Failed
    );
    same_frames(&server, &a);
    upload.request_cancel();
    finish(&mut server, &mut upload, 20);
    assert_eq!(upload.outcome(), Some(Outcome::Cancelled));
    assert_eq!(
        server.snapshot().unwrap().commit().identity,
        Identity::from_archive(&Archive::open(a.as_slice()).unwrap())
    );
}

#[test]
fn a_finished_upload_never_reinstalls_old_content_after_another_install_and_reconnect() {
    let a = package(10000);
    let b = package(20000);
    let dir = dir();
    let (mut server, fault) = open(dir.path(), 1);
    let mut first = Upload::new(a.as_slice()).unwrap();
    connect(&mut server, &mut first, 1);
    finish(&mut server, &mut first, 20);
    let completed = first.outcome();
    let mut second = Upload::new(b.as_slice()).unwrap();
    connect(&mut server, &mut second, 2);
    finish(&mut server, &mut second, 20);
    assert_eq!(fault.borrow().commits, 2);
    first.disconnect();
    assert_eq!(first.connect(id(1)), Err(Error::Connection));
    connect(&mut server, &mut first, 3);
    assert!(first.outbound().unwrap().is_none());
    assert_eq!(first.outcome(), completed); // Historical intent result, not current device state.
    assert_eq!(fault.borrow().commits, 2);
    same_frames(&server, &b);
}
