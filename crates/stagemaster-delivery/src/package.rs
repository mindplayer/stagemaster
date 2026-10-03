use crate::Error;
use stagemaster_install::Identity;
use stagemaster_package::{Archive, ReadAt};
use std::sync::Arc;

/// Immutable, fully checked content. This type is neither a signature nor a playback permit.
#[derive(Clone)]
pub struct Package {
    bytes: Arc<[u8]>,
    archive: Arc<Archive>,
}
impl Package {
    /// Run on the host's non-realtime work queue; validation scans every contained program.
    /// # Errors
    /// Reject malformed, unsupported or corrupt content and any supplied identity mismatch.
    pub fn from_bytes(bytes: Arc<[u8]>, expected: Option<Identity>) -> Result<Self, Error> {
        let archive = Archive::open(bytes.as_ref())?;
        if expected.is_some_and(|value| value != Identity::from_archive(&archive)) {
            return Err(Error::Identity);
        }
        Ok(Self {
            bytes,
            archive: Arc::new(archive),
        })
    }
    #[must_use]
    pub fn identity(&self) -> Identity {
        Identity::from_archive(&self.archive)
    }
    #[must_use]
    pub fn archive(&self) -> &Archive {
        &self.archive
    }
}
impl ReadAt for Package {
    fn len(&self) -> usize {
        self.bytes.len()
    }
    fn read_exact(
        &self,
        offset: usize,
        target: &mut [u8],
    ) -> Result<(), stagemaster_package::Error> {
        self.bytes.as_ref().read_exact(offset, target)
    }
}
