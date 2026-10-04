use stagemaster_live::Key;
use stagemaster_runtime::Code;
use std::sync::Arc;

/// Bounded immutable targets allocated before queue admission; worker clones only share them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch(Arc<[Key]>);
impl Batch {
    /// # Errors
    /// Reject empty/oversized or duplicate keys; the backend checks current boot and program kind.
    pub fn new(keys: &[Key]) -> Result<Self, Code> {
        if keys.is_empty() || keys.len() > 64 {
            return Err(Code::Budget);
        }
        for (i, key) in keys.iter().enumerate() {
            if keys[..i].contains(key) {
                return Err(Code::Selection);
            }
        }
        Ok(Self(keys.into()))
    }
    pub(crate) fn keys(&self) -> &[Key] {
        &self.0
    }
}
