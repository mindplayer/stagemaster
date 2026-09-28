use stagemaster_install::{Commit, Installer, Record, Slot, Storage};
use stagemaster_install_store::{FileSnapshot, FileStore};
use stagemaster_install_worker::{Command, Epoch, Error, Reply, Worker};
use stagemaster_package::Archive;
use stagemaster_project::{Document, PackageSelection};
use stagemaster_transfer::{Action, AuthorizedLink, Frame, Outcome, Request, Response, Upload};
use std::{cell::Cell, io, path::Path, rc::Rc};

fn epoch(n: u32) -> Epoch {
    Epoch::new(n).unwrap()
}
fn link(n: u8) -> [u8; 16] {
    [n; 16]
}
fn root() -> tempfile::TempDir {
    tempfile::tempdir_in(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
fn package() -> Vec<u8> {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    // The design example includes future entry points outside the lighting subset.
    json["entryPoints"] = serde_json::json!([]);
    let doc = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    let selection: Vec<_> = doc
        .view()
        .scenes
        .iter()
        .map(|s| PackageSelection::Scene { id: s.id.clone() })
        .collect();
    doc.build_package(&selection).unwrap().bytes
}
fn worker(path: &Path) -> Worker<FileStore> {
    Worker::new(
        Installer::open(FileStore::open(path).unwrap(), [7; 16])
            .unwrap()
            .0,
    )
    .unwrap()
}
fn open<S: Storage>(worker: &mut Worker<S>, n: u32, principal: u8) {
    let result = worker.process(
        Command::Open {
            epoch: epoch(n),
            link: AuthorizedLink {
                principal: link(principal),
                session: link(u8::try_from(n).unwrap()),
            },
        },
        || Some(epoch(n)),
    );
    assert!(matches!(result.result, Ok(Reply::Opened)));
}
fn frame<S: Storage>(worker: &mut Worker<S>, n: u32, value: Frame) -> Frame {
    match worker
        .process(
            Command::Frame {
                epoch: epoch(n),
                frame: value,
            },
            || Some(epoch(n)),
        )
        .result
        .unwrap()
    {
        Reply::Frame(value) => value,
        Reply::Opened => panic!("frame produced an open receipt"),
    }
}
fn status(n: u8, id: u64) -> Frame {
    Request {
        link: link(n),
        id,
        action: Action::Status,
    }
    .encode()
    .unwrap()
}

#[test]
fn queued_old_work_cannot_open_or_invalidate_a_new_connection() {
    let dir = root();
    let mut w = worker(dir.path());
    assert_eq!(
        w.process(
            Command::Frame {
                epoch: epoch(1),
                frame: status(1, 1)
            },
            || Some(epoch(1))
        )
        .result
        .unwrap_err(),
        Error::NotOpen
    );
    let old_open = Command::Open {
        epoch: epoch(1),
        link: AuthorizedLink {
            principal: link(9),
            session: link(1),
        },
    };
    assert_eq!(
        w.process(old_open, || Some(epoch(2))).result.unwrap_err(),
        Error::Obsolete
    );
    open(&mut w, 2, 9);
    assert_eq!(
        w.process(
            Command::Frame {
                epoch: epoch(1),
                frame: status(1, 1)
            },
            || Some(epoch(2))
        )
        .result
        .unwrap_err(),
        Error::Obsolete
    );
    let response = frame(&mut w, 2, status(2, 1));
    assert!(Response::decode(response.bytes()).unwrap().result.is_ok());
    w.observe(None);
    let same_open = Command::Open {
        epoch: epoch(2),
        link: AuthorizedLink {
            principal: link(9),
            session: link(2),
        },
    };
    assert_eq!(
        w.process(same_open, || Some(epoch(2))).result.unwrap_err(),
        Error::Obsolete
    );
    assert!(w.snapshot().is_err());
}

#[test]
fn protocol_failure_requires_a_new_authorized_epoch() {
    let dir = root();
    let mut w = worker(dir.path());
    open(&mut w, 1, 9);
    assert_eq!(
        w.process(
            Command::Frame {
                epoch: epoch(1),
                frame: status(99, 1)
            },
            || Some(epoch(1))
        )
        .result
        .unwrap_err(),
        Error::Protocol(stagemaster_transfer::Error::Connection)
    );
    assert_eq!(
        w.process(
            Command::Frame {
                epoch: epoch(1),
                frame: status(1, 1)
            },
            || Some(epoch(1))
        )
        .result
        .unwrap_err(),
        Error::NotOpen
    );
    open(&mut w, 2, 9);
    assert!(
        Response::decode(frame(&mut w, 2, status(2, 1)).bytes())
            .unwrap()
            .result
            .is_ok()
    );
}

struct InterruptStore {
    inner: FileStore,
    live: Rc<Cell<Option<Epoch>>>,
    prepare_count: Rc<Cell<usize>>,
}
impl Storage for InterruptStore {
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
        self.prepare_count.set(self.prepare_count.get() + 1);
        self.inner.prepare(slot, bytes)?;
        self.live.set(None); // Revocation during physical work; queue cleanup cannot undo it.
        Ok(())
    }
    fn write(&mut self, slot: Slot, offset: usize, bytes: &[u8]) -> io::Result<()> {
        self.inner.write(slot, offset, bytes)
    }
    fn sync_payload(&mut self, slot: Slot) -> io::Result<()> {
        self.inner.sync_payload(slot)
    }
    fn commit_record(&mut self, commit: Commit) -> io::Result<()> {
        self.inner.commit_record(commit)
    }
    fn settle(&mut self) -> io::Result<()> {
        self.inner.settle()
    }
    fn release(&mut self) {
        self.inner.release();
    }
    fn snapshot(&self, slot: Slot) -> io::Result<FileSnapshot> {
        self.inner.snapshot(slot)
    }
}

#[test]
fn revocation_during_prepare_drops_receipt_but_preserves_transaction_ownership() {
    let dir = root();
    let bytes = package();
    let live = Rc::new(Cell::new(Some(epoch(1))));
    let count = Rc::new(Cell::new(0));
    let store = InterruptStore {
        inner: FileStore::open(dir.path()).unwrap(),
        live: live.clone(),
        prepare_count: count.clone(),
    };
    let mut w = Worker::new(Installer::open(store, [7; 16]).unwrap().0).unwrap();
    open(&mut w, 1, 9);
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    upload.connect(link(1)).unwrap();
    let query = upload.outbound().unwrap().unwrap().clone();
    upload.accept(frame(&mut w, 1, query).bytes()).unwrap();
    let begin = upload.outbound().unwrap().unwrap().clone();
    assert_eq!(
        w.process(
            Command::Frame {
                epoch: epoch(1),
                frame: begin.clone()
            },
            || live.get()
        )
        .result
        .unwrap_err(),
        Error::Obsolete
    );
    assert_eq!(count.get(), 1);
    assert_eq!(
        w.process(
            Command::Frame {
                epoch: epoch(1),
                frame: begin
            },
            || live.get()
        )
        .result
        .unwrap_err(),
        Error::Obsolete
    );
    assert_eq!(count.get(), 1);
    live.set(Some(epoch(2)));
    open(&mut w, 2, 8);
    let state = Response::decode(frame(&mut w, 2, status(2, 1)).bytes())
        .unwrap()
        .state;
    assert!(!state.owned);
    let cancel = Request {
        link: link(2),
        id: 2,
        action: Action::Cancel(state.progress.unwrap().transaction),
    }
    .encode()
    .unwrap();
    assert_eq!(
        Response::decode(frame(&mut w, 2, cancel).bytes())
            .unwrap()
            .result,
        Err(stagemaster_transfer::RemoteError::Ownership)
    );
    live.set(Some(epoch(3)));
    open(&mut w, 3, 9);
    upload.disconnect();
    upload.request_cancel();
    upload.connect(link(3)).unwrap();
    while let Some(outbound) = upload.outbound().unwrap().cloned() {
        upload.accept(frame(&mut w, 3, outbound).bytes()).unwrap();
    }
    assert_eq!(upload.outcome(), Some(Outcome::Cancelled));
    assert!(w.snapshot().is_err());
    assert_eq!(count.get(), 1);
}

#[test]
fn lost_completion_at_each_boundary_reconciles_without_misreporting_cancel() {
    use stagemaster_transfer::Command as Op;
    for target in [Op::Status, Op::Begin, Op::Write, Op::Verify, Op::Commit] {
        let dir = root();
        let bytes = package();
        let mut w = worker(dir.path());
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        open(&mut w, 1, 9);
        upload.connect(link(1)).unwrap();
        let mut interrupted = false;
        let mut n = 1;
        while let Some(outbound) = upload.outbound().unwrap().cloned() {
            let op = Request::decode(outbound.bytes()).unwrap().action.command();
            if !interrupted && op == target {
                let mut calls = 0;
                let completion = w.process(
                    Command::Frame {
                        epoch: epoch(n),
                        frame: outbound,
                    },
                    || {
                        calls += 1;
                        (calls == 1).then_some(epoch(n))
                    },
                );
                assert_eq!(completion.result.unwrap_err(), Error::Obsolete);
                n += 1;
                open(&mut w, n, 9);
                upload.disconnect();
                if target == Op::Commit {
                    upload.request_cancel();
                }
                upload.connect(link(u8::try_from(n).unwrap())).unwrap();
                interrupted = true;
            } else {
                upload.accept(frame(&mut w, n, outbound).bytes()).unwrap();
            }
        }
        assert!(interrupted);
        assert!(matches!(upload.outcome(), Some(Outcome::Installed(_))));
        let commit = w.snapshot().unwrap().commit();
        assert_eq!(
            commit.identity.digest,
            *Archive::open(bytes.as_slice()).unwrap().digest()
        );
        drop(w);
        let reopened = worker(dir.path());
        assert_eq!(reopened.snapshot().unwrap().commit(), commit);
    }
}
