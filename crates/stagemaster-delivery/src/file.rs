use crate::{Error, Incoming, Package};
use stagemaster_install::Identity;
use stagemaster_package::MAX_PACKAGE_BYTES;
use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

/// Freeze a user-selected local/removable file before installation. Use a blocking work queue.
/// Cancellation is checked between bounded reads and validation; it cannot interrupt an OS read.
/// # Errors
/// Reject non-ordinary files, read failure, cancellation, excess length or invalid content.
pub fn from_file(
    path: &Path,
    expected: Option<Identity>,
    cancel: &AtomicBool,
) -> Result<Package, Error> {
    check_cancel(cancel)?;
    let mut incoming = Incoming::new(expected)?;
    if !fs::symlink_metadata(path)?.is_file() {
        return Err(Error::Source);
    }
    let mut file = File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(Error::Source);
    }
    if meta.len() > MAX_PACKAGE_BYTES as u64 {
        return Err(Error::Bounds);
    }
    if expected.is_some_and(|id| meta.len() != id.bytes as u64) {
        return Err(Error::Identity);
    }
    let mut block = [0; 8192];
    loop {
        check_cancel(cancel)?;
        let size = match file.read(&mut block) {
            Ok(size) => size,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error.into()),
        };
        if size == 0 {
            break;
        }
        incoming.push(&block[..size])?;
    }
    check_cancel(cancel)?;
    let package = incoming.finish()?;
    check_cancel(cancel)?;
    Ok(package)
}
fn check_cancel(cancel: &AtomicBool) -> Result<(), Error> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}
