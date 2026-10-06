//! Instrument the original `FileStore`; preserve all installation and immutable-reader semantics.
use stagemaster_install::{Commit, Record, Slot, Storage};
use stagemaster_install_store::{FileSnapshot, FileStore};
use stagemaster_package::ReadAt;
use std::{cell::Cell, io, rc::Rc};

pub struct Reader {
    inner: FileSnapshot,
    reads: Rc<Cell<usize>>,
    blocked: Rc<Cell<bool>>,
}
impl ReadAt for Reader {
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn read_exact(
        &self,
        offset: usize,
        target: &mut [u8],
    ) -> Result<(), stagemaster_package::Error> {
        self.reads.set(self.reads.get() + 1);
        if self.blocked.get() {
            return Err(stagemaster_package::Error::Read);
        }
        self.inner.read_exact(offset, target)
    }
}
pub struct Store {
    pub inner: FileStore,
    pub reads: Rc<Cell<usize>>,
    pub blocked: Rc<Cell<bool>>,
}
impl Storage for Store {
    type Error = io::Error;
    type Snapshot = Reader;
    fn capacity(&self, slot: Slot) -> usize {
        self.inner.capacity(slot)
    }
    fn record(&self, slot: Slot) -> io::Result<Record> {
        self.inner.record(slot)
    }
    fn slot_len(&self, slot: Slot) -> io::Result<usize> {
        self.inner.slot_len(slot)
    }
    fn read(&self, slot: Slot, offset: usize, target: &mut [u8]) -> io::Result<()> {
        self.inner.read(slot, offset, target)
    }
    fn prepare(&mut self, slot: Slot, bytes: usize) -> io::Result<()> {
        self.inner.prepare(slot, bytes)
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
    fn snapshot(&self, slot: Slot) -> io::Result<Reader> {
        Ok(Reader {
            inner: self.inner.snapshot(slot)?,
            reads: self.reads.clone(),
            blocked: self.blocked.clone(),
        })
    }
}
