use crate::{
    Code, Commit, Error, Identity, MAX_CHUNK_BYTES, Phase, Progress, Record, Slot, Storage,
    Transaction,
};
use stagemaster_package::{Archive, Program, ReadAt};

#[derive(Debug)]
pub enum SlotHealth {
    Empty,
    InvalidRecord,
    InvalidPackage(stagemaster_package::Error),
    Ready(Commit),
}
#[derive(Debug)]
pub struct RecoveryReport {
    pub slots: [SlotHealth; 2],
    pub selected: Option<Commit>,
    pub high_water: u64,
}

pub struct Installer<S> {
    storage: S,
    boot: [u8; 16],
    head: Option<Commit>,
    high_water: u64,
    progress: Option<Progress>,
}
impl<S: Storage> Installer<S> {
    /// # Errors
    /// Refuse zero boot identities, unreadable metadata and ambiguous committed generations.
    pub fn open(mut storage: S, boot: [u8; 16]) -> Result<(Self, RecoveryReport), Error<S::Error>> {
        if boot == [0; 16] {
            return Err(Code::Identity.into());
        }
        storage.settle().map_err(Error::Storage)?;
        let report = inspect(&storage)?;
        let installer = Self {
            storage,
            boot,
            head: report.selected,
            high_water: report.high_water,
            progress: None,
        };
        Ok((installer, report))
    }
    #[must_use]
    pub const fn transaction(&self, counter: u64) -> Transaction {
        Transaction {
            boot: self.boot,
            counter,
        }
    }
    #[must_use]
    pub const fn head(&self) -> Option<Commit> {
        self.head
    }
    #[must_use]
    pub const fn progress(&self) -> Option<Progress> {
        self.progress
    }

    /// # Errors
    /// Reject stale transactions, conflicting retries, another live transfer and slot limits.
    pub fn begin(
        &mut self,
        transaction: Transaction,
        identity: Identity,
    ) -> Result<Progress, Error<S::Error>> {
        self.check_boot(transaction)?;
        identity.validate()?;
        if let Some(p) = self.progress {
            if p.phase == Phase::Uncertain {
                return Err(Code::Uncertain.into());
            }
            if transaction == p.transaction {
                return if identity == p.identity {
                    Ok(p)
                } else {
                    Err(Code::Conflict.into())
                };
            }
            if transaction.counter <= p.transaction.counter {
                return Err(Code::Stale.into());
            }
            if matches!(p.phase, Phase::Receiving | Phase::Verified | Phase::Failed) {
                return Err(Code::Busy.into());
            }
        }
        let last = self.progress.map_or(0, |p| p.transaction.counter);
        if transaction.counter != last.checked_add(1).ok_or(Code::Exhausted)? {
            return Err(Code::Order.into());
        }
        // A lost receipt does not cause another installation of the same immutable package.
        if let Some(head) = self.head.filter(|head| head.identity == identity) {
            verify_slot(&self.storage, head)?;
            let p = Progress {
                transaction,
                identity,
                received: identity.bytes,
                phase: Phase::Committed,
                commit: head,
            };
            self.progress = Some(p);
            return Ok(p);
        }
        let slot = self.head.map_or(Slot::A, |head| head.slot.other());
        if identity.bytes > self.storage.capacity(slot) {
            return Err(Code::Bounds.into());
        }
        let commit = Commit {
            slot,
            identity,
            generation: self.high_water.checked_add(1).ok_or(Code::Exhausted)?,
        };
        self.progress = Some(Progress {
            transaction,
            identity,
            received: 0,
            phase: Phase::Failed,
            commit,
        });
        if let Err(error) = self.storage.prepare(slot, identity.bytes) {
            return Err(Error::Storage(error));
        }
        self.phase(Phase::Receiving);
        self.progress.ok_or_else(|| Code::State.into())
    }
    /// # Errors
    /// Require a contiguous prefix. Complete duplicates must match bytes already accepted.
    pub fn write(
        &mut self,
        transaction: Transaction,
        offset: usize,
        bytes: &[u8],
    ) -> Result<Progress, Error<S::Error>> {
        let p = self.current(transaction)?;
        if p.phase != Phase::Receiving {
            return Err(Code::State.into());
        }
        let end = offset.checked_add(bytes.len()).ok_or(Code::Bounds)?;
        if bytes.is_empty() || bytes.len() > MAX_CHUNK_BYTES || end > p.identity.bytes {
            return Err(Code::Bounds.into());
        }
        if offset < p.received {
            if end > p.received {
                return Err(Code::Order.into());
            }
            let mut previous = [0; MAX_CHUNK_BYTES];
            if let Err(e) = self
                .storage
                .read(p.commit.slot, offset, &mut previous[..bytes.len()])
            {
                self.phase(Phase::Failed);
                return Err(Error::Storage(e));
            }
            if previous[..bytes.len()] != *bytes {
                return Err(Code::Conflict.into());
            }
            return Ok(p);
        }
        if offset != p.received {
            return Err(Code::Order.into());
        }
        if let Err(error) = self.storage.write(p.commit.slot, offset, bytes) {
            self.phase(Phase::Failed);
            return Err(Error::Storage(error));
        }
        let next = Progress { received: end, ..p };
        self.progress = Some(next);
        Ok(next)
    }
    /// # Errors
    /// All bytes, successful persistence and complete independent archive validation are required.
    pub fn verify(&mut self, transaction: Transaction) -> Result<Progress, Error<S::Error>> {
        let p = self.current(transaction)?;
        if matches!(p.phase, Phase::Verified | Phase::Committed) {
            return Ok(p);
        }
        if p.phase != Phase::Receiving {
            return Err(Code::State.into());
        }
        if p.received != p.identity.bytes {
            return Err(Code::Incomplete.into());
        }
        self.phase(Phase::Failed);
        self.storage
            .sync_payload(p.commit.slot)
            .map_err(Error::Storage)?;
        verify_slot(&self.storage, p.commit)?;
        self.phase(Phase::Verified);
        self.progress.ok_or_else(|| Code::State.into())
    }
    /// # Errors
    /// A failed metadata commit is uncertain until reconcile succeeds; it is not a rollback.
    pub fn commit(&mut self, transaction: Transaction) -> Result<Commit, Error<S::Error>> {
        let p = self.current(transaction)?;
        if p.phase == Phase::Committed {
            return Ok(p.commit);
        }
        if p.phase == Phase::Uncertain {
            return Err(Code::Uncertain.into());
        }
        if p.phase != Phase::Verified {
            return Err(Code::State.into());
        }
        self.phase(Phase::Failed);
        verify_slot(&self.storage, p.commit)?;
        self.phase(Phase::Uncertain);
        self.storage
            .commit_record(p.commit)
            .map_err(Error::CommitUncertain)?;
        self.head = Some(p.commit);
        self.high_water = p.commit.generation;
        self.phase(Phase::Committed);
        self.storage.release();
        Ok(p.commit)
    }
    /// # Errors
    /// Committed/uncertain transactions cannot be silently undone by cancellation.
    pub fn cancel(&mut self, transaction: Transaction) -> Result<Progress, Error<S::Error>> {
        let p = self.current(transaction)?;
        match p.phase {
            Phase::Committed => return Err(Code::State.into()),
            Phase::Uncertain => return Err(Code::Uncertain.into()),
            _ => {}
        }
        self.storage.release();
        self.phase(Phase::Cancelled);
        self.progress.ok_or_else(|| Code::State.into())
    }
    /// # Errors
    /// Keep the uncertain state on synchronization/read failures; never guess commit outcome.
    pub fn reconcile(&mut self) -> Result<RecoveryReport, Error<S::Error>> {
        let p = self.progress.ok_or(Code::State)?;
        if p.phase != Phase::Uncertain {
            return Err(Code::State.into());
        }
        self.storage.settle().map_err(Error::Storage)?;
        let report = inspect(&self.storage)?;
        self.head = report.selected;
        self.high_water = report.high_water;
        self.phase(if report.selected == Some(p.commit) {
            Phase::Committed
        } else {
            Phase::Failed
        });
        self.storage.release();
        Ok(report)
    }
    /// # Errors
    /// Return only an independently verified, immutable, leased installed snapshot.
    pub fn snapshot(&self) -> Result<Installed<S::Snapshot>, Error<S::Error>> {
        let commit = self.head.ok_or(Code::Empty)?;
        let reader = self.storage.snapshot(commit.slot).map_err(Error::Storage)?;
        let archive = Archive::open(&reader)?;
        if Identity::from_archive(&archive) != commit.identity {
            return Err(stagemaster_package::Error::Integrity.into());
        }
        Ok(Installed {
            reader,
            archive,
            commit,
        })
    }
    fn phase(&mut self, phase: Phase) {
        if let Some(p) = &mut self.progress {
            p.phase = phase;
        }
    }
    fn check_boot(&self, transaction: Transaction) -> Result<(), Code> {
        if transaction.boot != self.boot || transaction.counter == 0 {
            return Err(Code::Stale);
        }
        Ok(())
    }
    fn current(&self, transaction: Transaction) -> Result<Progress, Code> {
        self.check_boot(transaction)?;
        self.progress
            .filter(|p| p.transaction == transaction)
            .ok_or(Code::Stale)
    }
}

pub struct Installed<R> {
    reader: R,
    archive: Archive,
    commit: Commit,
}
impl<R: ReadAt> Installed<R> {
    #[must_use]
    pub const fn commit(&self) -> Commit {
        self.commit
    }
    #[must_use]
    pub const fn archive(&self) -> &Archive {
        &self.archive
    }
    /// # Errors
    /// Preserve the archive's per-block integrity and plan limits.
    pub fn load(&self, index: usize) -> Result<Program, stagemaster_package::Error> {
        self.archive.load(&self.reader, index)
    }
}

struct SlotReader<'a, S> {
    storage: &'a S,
    slot: Slot,
    length: usize,
}
impl<S: Storage> ReadAt for SlotReader<'_, S> {
    fn len(&self) -> usize {
        self.length
    }
    fn read_exact(
        &self,
        offset: usize,
        target: &mut [u8],
    ) -> Result<(), stagemaster_package::Error> {
        self.storage
            .read(self.slot, offset, target)
            .map_err(|_| stagemaster_package::Error::Read)
    }
}
fn verify_slot<S: Storage>(storage: &S, commit: Commit) -> Result<(), stagemaster_package::Error> {
    let length = storage
        .slot_len(commit.slot)
        .map_err(|_| stagemaster_package::Error::Read)?;
    if length != commit.identity.bytes || length > storage.capacity(commit.slot) {
        return Err(stagemaster_package::Error::Integrity);
    }
    let reader = SlotReader {
        storage,
        slot: commit.slot,
        length,
    };
    let archive = Archive::open(&reader)?;
    if Identity::from_archive(&archive) != commit.identity {
        return Err(stagemaster_package::Error::Integrity);
    }
    Ok(())
}
fn inspect<S: Storage>(storage: &S) -> Result<RecoveryReport, Error<S::Error>> {
    let mut report = RecoveryReport {
        slots: [SlotHealth::Empty, SlotHealth::Empty],
        selected: None,
        high_water: 0,
    };
    let mut generations = [None, None];
    for slot in [Slot::A, Slot::B] {
        let record = storage.record(slot).map_err(Error::Storage)?;
        let commit = match record {
            Record::Absent => continue,
            Record::Bytes(bytes) => Commit::decode(slot, &bytes).ok(),
            Record::Invalid => None,
        };
        let Some(commit) = commit else {
            report.slots[slot.index()] = SlotHealth::InvalidRecord;
            continue;
        };
        generations[slot.index()] = Some(commit.generation);
        report.high_water = report.high_water.max(commit.generation);
        report.slots[slot.index()] = match verify_slot(storage, commit) {
            Ok(()) => {
                if report
                    .selected
                    .is_none_or(|head| head.generation < commit.generation)
                {
                    report.selected = Some(commit);
                }
                SlotHealth::Ready(commit)
            }
            Err(e) => SlotHealth::InvalidPackage(e),
        };
    }
    if generations[0].is_some() && generations[0] == generations[1] {
        return Err(Code::Metadata.into());
    }
    Ok(report)
}
