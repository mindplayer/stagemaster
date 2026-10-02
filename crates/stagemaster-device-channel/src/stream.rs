use crate::{Error, MAX_RECORD_BYTES, QUEUED_RECORDS, RECORD_TIMEOUT, RecordIo};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf},
    sync::mpsc,
    task::JoinHandle,
};

/// Bounded records over an already-connected byte stream. Does not discover/connect endpoints.
/// Framing: two-byte big-endian length followed by 1..=1297 untrusted bytes.
pub struct StreamRecords<S: AsyncRead + AsyncWrite + Unpin + Send + 'static> {
    writer: Option<WriteHalf<S>>,
    queue: mpsc::Receiver<Vec<u8>>,
    failed: Arc<AtomicBool>,
    task: JoinHandle<()>,
    sending: bool,
}
impl<S: AsyncRead + AsyncWrite + Unpin + Send + 'static> StreamRecords<S> {
    /// Must be constructed inside a Tokio runtime. Owns both halves until closed/dropped.
    pub fn new(stream: S) -> Self {
        let (reader, writer) = tokio::io::split(stream);
        let (sender, queue) = mpsc::channel(QUEUED_RECORDS);
        let failed = Arc::new(AtomicBool::new(false));
        let signal = failed.clone();
        let task = tokio::spawn(async move {
            let _ = read_records(reader, sender).await;
            signal.store(true, Ordering::Release);
        });
        Self {
            writer: Some(writer),
            queue,
            failed,
            task,
            sending: false,
        }
    }
}
fn transport(error: impl std::fmt::Display) -> Error {
    Error::Transport(error.to_string())
}
async fn read_records<S: AsyncRead + AsyncWrite + Unpin>(
    mut reader: ReadHalf<S>,
    sender: mpsc::Sender<Vec<u8>>,
) -> Result<(), Error> {
    let mut payload = [0; MAX_RECORD_BYTES];
    loop {
        let mut header = [0; 2];
        // Idle connection is governed by the application lease; a partial record has its own deadline.
        reader
            .read_exact(&mut header[..1])
            .await
            .map_err(transport)?;
        tokio::time::timeout(RECORD_TIMEOUT, async {
            reader
                .read_exact(&mut header[1..])
                .await
                .map_err(transport)?;
            let length = usize::from(u16::from_be_bytes(header));
            if !(1..=MAX_RECORD_BYTES).contains(&length) {
                return Err(Error::Bounds);
            }
            reader
                .read_exact(&mut payload[..length])
                .await
                .map_err(transport)?;
            sender
                .try_send(payload[..length].to_vec())
                .map_err(|_| Error::Closed)
        })
        .await
        .map_err(|_| Error::Timeout)??;
    }
}
impl<S: AsyncRead + AsyncWrite + Unpin + Send + 'static> RecordIo for StreamRecords<S> {
    async fn send(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if !self.healthy() {
            self.close();
            return Err(Error::Closed);
        }
        if !(1..=MAX_RECORD_BYTES).contains(&bytes.len()) {
            self.close();
            return Err(Error::Bounds);
        }
        self.sending = true; // Retained if this future is dropped after a partial write.
        let result = tokio::time::timeout(RECORD_TIMEOUT, async {
            let writer = self.writer.as_mut().ok_or(Error::Closed)?;
            let length = u16::try_from(bytes.len()).map_err(|_| Error::Bounds)?;
            writer
                .write_all(&length.to_be_bytes())
                .await
                .map_err(transport)?;
            writer.write_all(bytes).await.map_err(transport)?;
            writer.flush().await.map_err(transport)
        })
        .await
        .unwrap_or(Err(Error::Timeout));
        if result.is_err() || self.failed.load(Ordering::Acquire) {
            self.close();
            return result.and(Err(Error::Closed));
        }
        self.sending = false;
        Ok(())
    }
    fn try_receive(&mut self) -> Result<Option<Vec<u8>>, Error> {
        if !self.healthy() {
            self.close();
            return Err(Error::Closed);
        }
        match self.queue.try_recv() {
            Ok(bytes) => Ok(Some(bytes)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                self.close();
                Err(Error::Closed)
            }
        }
    }
    fn healthy(&self) -> bool {
        self.writer.is_some() && !self.sending && !self.failed.load(Ordering::Acquire)
    }
    fn close(&mut self) {
        self.failed.store(true, Ordering::Release);
        self.task.abort();
        self.writer = None;
        self.queue.close();
        while self.queue.try_recv().is_ok() {}
    }
}
impl<S: AsyncRead + AsyncWrite + Unpin + Send + 'static> Drop for StreamRecords<S> {
    fn drop(&mut self) {
        self.close();
    }
}
