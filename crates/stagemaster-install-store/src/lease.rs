//! The store owns lock lifetime; incidental duplicated descriptors must not extend it.
use std::{fs::File, io};
pub(super) struct Lease(File);
impl Lease {
    pub(super) fn exclusive(file: File) -> io::Result<Self> {
        file.try_lock().map_err(io::Error::other)?;
        Ok(Self(file))
    }
    pub(super) fn shared(file: File) -> io::Result<Self> {
        file.try_lock_shared().map_err(io::Error::other)?;
        Ok(Self(file))
    }
    #[cfg(test)]
    pub(super) fn duplicate(&self) -> io::Result<File> {
        self.0.try_clone()
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        // No retry or forced takeover: if explicit unlock fails, closing the file remains
        // the platform fallback. The guarded resource is already out of use by its owner.
        let _ = self.0.unlock();
    }
}
