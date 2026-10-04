//! Immutable observation buffers, refreshed only from the original manual contribution.
use stagemaster_live::{Key, Session};
use std::sync::Arc;

pub type ManualValues = [Option<Arc<[Option<u16>]>>; 64];
pub(crate) struct ManualSnapshots(ManualValues);
impl ManualSnapshots {
    pub fn new(session: &Session) -> Self {
        let mut values = Self(std::array::from_fn(|_| None));
        for source in session.sources() {
            values.refresh(session, source.key);
        }
        values
    }
    pub fn refresh(&mut self, session: &Session, key: Key) {
        let Some(values) = session.manual_values(key) else {
            return;
        };
        // Slot order is the prepared Session source order and never changes.
        let Some(index) = session.sources().position(|s| s.key == key) else {
            return;
        };
        if self.0[index].as_deref() != Some(values) {
            self.0[index] = Some(Arc::from(values));
        }
    }
    pub fn snapshot(&self) -> ManualValues {
        self.0.clone()
    }
}
