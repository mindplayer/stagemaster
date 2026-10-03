use super::slots::{Activation, Entry, Inbox, Reclaimed, Staging, Update};
use stagemaster_live::media::{GroupInfo, GroupKey, Prepared, Sample};
use stagemaster_runtime::Code;
use stagemaster_time::Mapping;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

/// Trusted provider/preparation endpoint for exactly one fixed local group.
/// This is not an operator lease. Controlled activation/completion requires a current authorized ticket.
pub struct MediaPort {
    pub(super) group: GroupInfo,
    pub(super) staging: Arc<Mutex<Staging>>,
    pub(super) inbox: Arc<Mutex<Inbox>>,
    pub(super) alive: Arc<AtomicBool>,
    pub(super) control: Option<Arc<Mutex<super::control::Mailbox>>>,
}
impl MediaPort {
    /// Initial group identity. Obtain current playback keys from authoritative host state/receipts.
    #[must_use]
    pub const fn initial(&self) -> GroupInfo {
        self.group
    }
    /// Stage one prepared replacement off the scheduler; submit its ticket through the original Client.
    /// # Errors
    /// Reject other groups, contention, an occupied slot or exhausted ticket space.
    pub fn stage(
        &self,
        prepared: Prepared,
        sample: Sample,
        mapping: Mapping,
    ) -> Result<Activation, Code> {
        if self.control.is_some() {
            return Err(Code::State);
        }
        self.stage_inner(prepared, sample, mapping, None)
    }
    pub(super) fn stage_inner(
        &self,
        prepared: Prepared,
        sample: Sample,
        mapping: Mapping,
        request: Option<super::ControlTicket>,
    ) -> Result<Activation, Code> {
        if !self.alive.load(Ordering::Acquire) {
            return Err(Code::State);
        }
        if !prepared.key().same_group(self.group.key) {
            return Err(Code::Selection);
        }
        let mut slot = self.staging.try_lock().map_err(|_| Code::Busy)?;
        if slot.entry.is_some() {
            return Err(Code::Busy);
        }
        let serial = slot.serial.checked_add(1).ok_or(Code::Exhausted)?;
        let ticket = Activation {
            group: prepared.key(),
            serial,
        };
        slot.entry = Some(Entry {
            ticket,
            prepared,
            sample,
            mapping,
            result: None,
            request,
        });
        slot.serial = serial;
        Ok(ticket)
    }
    /// Take back the staged/replaced plans on this caller's thread.
    /// With withdraw=false, pending commands retain their slot. With true, a pending ticket is cancelled.
    /// A wait timeout alone is never cancellation; inspect result to distinguish applied from withdrawn.
    /// # Errors
    /// Reject busy slots, stale tickets or pending preparations without explicit withdrawal.
    pub fn reclaim(&self, ticket: Activation, withdraw: bool) -> Result<Reclaimed, Code> {
        let mut slot = self.staging.try_lock().map_err(|_| Code::Busy)?;
        let entry = slot
            .entry
            .as_ref()
            .filter(|e| e.ticket == ticket)
            .ok_or(Code::State)?;
        if entry.result.is_none() && !withdraw {
            return Err(Code::Busy);
        }
        let entry = slot.entry.take().ok_or(Code::State)?;
        Ok(Reclaimed {
            prepared: entry.prepared,
            result: entry.result,
        })
    }
    /// Replace this group's pending observation. Intermediate observations may be superseded.
    /// Only the original sample time/sequence may be published; repeat reads do not become new samples.
    /// # Errors
    /// Refuse another group/provider, contention or counter exhaustion. Host validates clock quality and generation.
    pub fn publish(&self, key: GroupKey, sample: Sample, mapping: Mapping) -> Result<u64, Code> {
        if !self.alive.load(Ordering::Acquire) {
            return Err(Code::State);
        }
        if !key.same_group(self.group.key) || sample.at.clock != self.group.provider {
            return Err(Code::Selection);
        }
        let mut inbox = self.inbox.try_lock().map_err(|_| Code::Busy)?;
        let serial = inbox.serial.checked_add(1).ok_or(Code::Exhausted)?;
        inbox.latest = Some(Update {
            serial,
            key,
            sample,
            mapping,
        });
        inbox.serial = serial;
        Ok(serial)
    }
}
