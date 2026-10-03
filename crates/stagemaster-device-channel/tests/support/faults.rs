use stagemaster_device_channel::{Error, RecordIo};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};
/// Deliberately preserves the carrier's health flag on injected faults: the nonce owner
/// must enforce its own cancellation boundary, not delegate that decision to the carrier.
pub struct FaultIo<R> {
    pub inner: R,
    pub mode: Arc<AtomicU8>,
}
impl<R: RecordIo> RecordIo for FaultIo<R> {
    async fn send(&mut self, bytes: &[u8]) -> Result<(), Error> {
        match self.mode.load(Ordering::Acquire) {
            1 => std::future::pending::<()>().await,
            2 => return Err(Error::Closed),
            3 => tokio::time::sleep(Duration::from_secs(4)).await,
            _ => (),
        }
        self.inner.send(bytes).await
    }
    fn try_receive(&mut self) -> Result<Option<Vec<u8>>, Error> {
        self.inner.try_receive()
    }
    fn healthy(&self) -> bool {
        self.inner.healthy()
    }
    fn close(&mut self) {
        self.inner.close();
    }
}
