use crate::{Buffer, Code, Error, METADATA_BYTES, Shared, io, metadata, payload::Stage};
use alloc::rc::Rc;
use embedded_storage::nor_flash::NorFlash;
use stagemaster_install::{Commit, Record, Slot, Storage};
use stagemaster_package::ReadAt;

pub struct NorStore<F> {
    shared: Rc<Shared<F>>,
    writable: bool,
    stage: Option<Stage>,
}
impl<F> NorStore<F> {
    pub(crate) fn new(shared: Rc<Shared<F>>, writable: bool) -> Self {
        Self {
            shared,
            writable,
            stage: None,
        }
    }
    fn clear(&mut self) {
        self.stage = None;
        self.shared.active.set(None);
    }
}
impl<F> Drop for NorStore<F> {
    fn drop(&mut self) {
        self.clear();
        self.shared.writer.set(false);
    }
}
impl<F: NorFlash> NorStore<F> {
    fn writable(&self) -> Result<(), Error<F::Error>> {
        if !self.writable {
            return Err(Code::ReadOnly.into());
        }
        Ok(())
    }
    fn committed_len(&self, slot: Slot) -> Result<usize, Error<F::Error>> {
        let Record::Bytes(bytes) = metadata::read(&self.shared, slot)? else {
            return Err(Code::State.into());
        };
        let commit = Commit::decode(slot, &bytes).map_err(|_| Code::Integrity)?;
        if commit.identity.bytes > self.shared.layout.slot_bytes() {
            return Err(Code::Bounds.into());
        }
        Ok(commit.identity.bytes)
    }
}
impl<F: NorFlash> Storage for NorStore<F> {
    type Error = Error<F::Error>;
    type Snapshot = Snapshot<F>;
    fn capacity(&self, _slot: Slot) -> usize {
        self.shared.layout.slot_bytes()
    }
    fn record(&self, slot: Slot) -> Result<Record, Self::Error> {
        metadata::read(&self.shared, slot)
    }
    fn slot_len(&self, slot: Slot) -> Result<usize, Self::Error> {
        if let Some(stage) = self.stage.as_ref().filter(|s| s.slot == slot) {
            if stage.poisoned {
                return Err(Code::State.into());
            }
            return Ok(stage.bytes);
        }
        self.committed_len(slot)
    }
    fn read(&self, slot: Slot, offset: usize, target: &mut [u8]) -> Result<(), Self::Error> {
        let end = offset.checked_add(target.len()).ok_or(Code::Bounds)?;
        let base = self.shared.layout.payload_offset(slot);
        if let Some(stage) = self.stage.as_ref().filter(|s| s.slot == slot) {
            if stage.poisoned {
                return Err(Code::State.into());
            }
            if end > stage.received {
                return Err(Code::Bounds.into());
            }
            let physical = stage.programmed.min(end).saturating_sub(offset);
            io::read(&self.shared, base + offset, &mut target[..physical])?;
            if physical < target.len() {
                let tail_offset = offset + physical - stage.programmed;
                target[physical..].copy_from_slice(
                    &stage.tail[tail_offset..tail_offset + end - offset - physical],
                );
            }
            return Ok(());
        }
        if end > self.committed_len(slot)? {
            return Err(Code::Bounds.into());
        }
        io::read(&self.shared, base + offset, target)
    }
    fn prepare(&mut self, slot: Slot, bytes: usize) -> Result<(), Self::Error> {
        self.writable()?;
        if self.stage.is_some() || self.shared.pins[slot.index()].get() != 0 {
            return Err(Code::Busy.into());
        }
        if !(64..=self.capacity(slot)).contains(&bytes) {
            return Err(Code::Bounds.into());
        }
        self.shared.active.set(Some(slot));
        self.stage = Some(Stage::new(slot, bytes));
        let meta = self.shared.layout.metadata_offset(slot);
        io::erase(&self.shared, meta, meta + METADATA_BYTES)?;
        self.stage.as_mut().ok_or(Code::State)?.poisoned = false;
        Ok(())
    }
    fn write(&mut self, slot: Slot, offset: usize, bytes: &[u8]) -> Result<(), Self::Error> {
        self.writable()?;
        let stage = self
            .stage
            .as_mut()
            .filter(|s| s.slot == slot && !s.poisoned && !s.sealed)
            .ok_or(Code::State)?;
        stage.write(&self.shared, offset, bytes)
    }
    fn sync_payload(&mut self, slot: Slot) -> Result<(), Self::Error> {
        self.writable()?;
        let stage = self
            .stage
            .as_mut()
            .filter(|s| s.slot == slot && !s.poisoned && s.received == s.bytes)
            .ok_or(Code::State)?;
        stage.sync(&self.shared)
    }
    fn commit_record(&mut self, commit: Commit) -> Result<(), Self::Error> {
        self.writable()?;
        let stage = self
            .stage
            .as_mut()
            .filter(|s| {
                s.slot == commit.slot
                    && s.bytes == commit.identity.bytes
                    && !s.poisoned
                    && s.sealed
                    && !s.commit_attempted
            })
            .ok_or(Code::State)?;
        stage.commit_attempted = true;
        metadata::write(&self.shared, commit)
    }
    fn settle(&mut self) -> Result<(), Self::Error> {
        // Reads serialize behind completed synchronous operations, including failed program commands.
        // No persistent progress or cached writes exist in this adapter.
        let mut buffer = Buffer::erased();
        for slot in [Slot::A, Slot::B] {
            io::read(
                &self.shared,
                self.shared.layout.metadata_offset(slot),
                &mut buffer.0[..16],
            )?;
        }
        Ok(())
    }
    fn release(&mut self) {
        self.clear();
    }
    fn snapshot(&self, slot: Slot) -> Result<Self::Snapshot, Self::Error> {
        if self.shared.active.get() == Some(slot) {
            return Err(Code::Busy.into());
        }
        let length = self.committed_len(slot)?;
        let pins = &self.shared.pins[slot.index()];
        pins.set(pins.get().checked_add(1).ok_or(Code::Busy)?);
        Ok(Snapshot {
            shared: self.shared.clone(),
            slot,
            length,
        })
    }
}
pub struct Snapshot<F> {
    shared: Rc<Shared<F>>,
    slot: Slot,
    length: usize,
}
impl<F> Drop for Snapshot<F> {
    fn drop(&mut self) {
        let pins = &self.shared.pins[self.slot.index()];
        pins.set(pins.get() - 1);
    }
}
impl<F: NorFlash> ReadAt for Snapshot<F> {
    fn len(&self) -> usize {
        self.length
    }
    fn read_exact(
        &self,
        offset: usize,
        target: &mut [u8],
    ) -> Result<(), stagemaster_package::Error> {
        if offset
            .checked_add(target.len())
            .is_none_or(|end| end > self.length)
        {
            return Err(stagemaster_package::Error::Read);
        }
        io::read(
            &self.shared,
            self.shared.layout.payload_offset(self.slot) + offset,
            target,
        )
        .map_err(|_| stagemaster_package::Error::Read)
    }
}
