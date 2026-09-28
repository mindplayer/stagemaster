use stagemaster_install::{Commit, Record, Slot, Storage};
use stagemaster_install_store::{FileSnapshot, FileStore};
use stagemaster_package::{Error, ReadAt};
use std::{cell::Cell, io, rc::Rc};

#[derive(Default)]
pub struct Metrics {
    pub mutations: Cell<usize>,
    pub readers: Cell<usize>,
    pub fail_prepare: Cell<bool>,
    pub fail_read: Cell<bool>,
    pub lose_commit: Cell<bool>,
}
pub struct Reader {
    source: FileSnapshot,
    metrics: Rc<Metrics>,
}
impl ReadAt for Reader {
    fn len(&self) -> usize {
        self.source.len()
    }
    fn read_exact(&self, offset: usize, target: &mut [u8]) -> Result<(), Error> {
        if self.metrics.fail_read.get() {
            return Err(Error::Read);
        }
        self.source.read_exact(offset, target)
    }
}
impl Drop for Reader {
    fn drop(&mut self) {
        self.metrics.readers.set(self.metrics.readers.get() - 1);
    }
}
pub struct Store {
    pub inner: FileStore,
    pub metrics: Rc<Metrics>,
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
        assert_eq!(
            self.metrics.readers.get(),
            0,
            "runtime must release its package reader before installation"
        );
        self.metrics.mutations.set(self.metrics.mutations.get() + 1);
        if self.metrics.fail_prepare.replace(false) {
            return Err(io::Error::other("prepare fault"));
        }
        self.inner.prepare(slot, bytes)
    }
    fn write(&mut self, slot: Slot, offset: usize, bytes: &[u8]) -> io::Result<()> {
        self.metrics.mutations.set(self.metrics.mutations.get() + 1);
        self.inner.write(slot, offset, bytes)
    }
    fn sync_payload(&mut self, slot: Slot) -> io::Result<()> {
        self.inner.sync_payload(slot)
    }
    fn commit_record(&mut self, commit: Commit) -> io::Result<()> {
        self.metrics.mutations.set(self.metrics.mutations.get() + 1);
        self.inner.commit_record(commit)?;
        if self.metrics.lose_commit.replace(false) {
            return Err(io::Error::other("lost durable commit"));
        }
        Ok(())
    }
    fn settle(&mut self) -> io::Result<()> {
        self.inner.settle()
    }
    fn release(&mut self) {
        self.inner.release();
    }
    fn snapshot(&self, slot: Slot) -> io::Result<Reader> {
        let source = self.inner.snapshot(slot)?;
        self.metrics.readers.set(self.metrics.readers.get() + 1);
        Ok(Reader {
            source,
            metrics: self.metrics.clone(),
        })
    }
}
