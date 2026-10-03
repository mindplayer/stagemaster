use crate::{Error, Package};
use stagemaster_install::Identity;
use stagemaster_package::MAX_PACKAGE_BYTES;

/// One disposable transfer, with no side effects on installation or the active runtime.
pub struct Incoming {
    bytes: Vec<u8>,
    expected: Option<Identity>,
    limit: usize,
    failed: bool,
}
impl Incoming {
    /// Expected identity comes from the caller's publication/selection boundary, not this stream.
    /// # Errors
    /// Reject impossible lengths before allocating or reading a source.
    pub fn new(expected: Option<Identity>) -> Result<Self, Error> {
        let limit = expected.map_or(MAX_PACKAGE_BYTES, |id| id.bytes);
        if !(64..=MAX_PACKAGE_BYTES).contains(&limit) {
            return Err(Error::Bounds);
        }
        Ok(Self {
            bytes: Vec::new(),
            expected,
            limit,
            failed: false,
        })
    }
    /// # Errors
    /// Refuse oversized or previously failed transfers; a failed transfer cannot later finish.
    pub fn push(&mut self, chunk: &[u8]) -> Result<(), Error> {
        if self.failed || chunk.len() > self.limit - self.bytes.len() {
            self.failed = true;
            return Err(Error::Bounds);
        }
        if self.bytes.try_reserve_exact(chunk.len()).is_err() {
            self.failed = true;
            return Err(Error::Allocation);
        }
        self.bytes.extend_from_slice(chunk);
        Ok(())
    }
    #[must_use]
    pub fn received(&self) -> usize {
        self.bytes.len()
    }
    /// # Errors
    /// Require completion and full structural/semantic/integrity verification before exposing bytes.
    pub fn finish(self) -> Result<Package, Error> {
        if self.failed {
            return Err(Error::Bounds);
        }
        if self.expected.is_some_and(|id| self.bytes.len() != id.bytes) {
            return Err(Error::Incomplete);
        }
        Package::from_bytes(self.bytes.into(), self.expected)
    }
}
