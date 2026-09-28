//! Drain the SDK stream continuously into a bounded connection-local receiver.
#[cfg(test)]
mod tests;
use crate::{Problem, ProblemCode};
use btleplug::api::ValueNotification;
use futures_util::{StreamExt, stream::BoxStream};
use stagemaster_device_link::management::Serial;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::{sync::mpsc, task::JoinHandle};
use uuid::Uuid;

const CAPACITY: usize = 64;
pub(super) struct Receiver {
    queue: mpsc::Receiver<Vec<u8>>,
    failed: Arc<AtomicBool>,
    task: JoinHandle<()>,
}
impl Receiver {
    pub fn spawn(
        mut stream: BoxStream<'static, ValueNotification>,
        uuid: Uuid,
        limit: u16,
    ) -> Self {
        let (sender, queue) = mpsc::channel(CAPACITY);
        let failed = Arc::new(AtomicBool::new(false));
        let signal = failed.clone();
        let task = tokio::spawn(async move {
            let mut sequence = Serial::default();
            while let Some(notification) = stream.next().await {
                if notification.uuid != uuid
                    || notification.service_uuid != super::installation::SERVICE
                {
                    continue;
                }
                let Ok(bytes) = sequence.receive(&notification.value) else {
                    break;
                };
                if bytes.len() > usize::from(limit) || sender.try_send(bytes.to_vec()).is_err() {
                    break;
                }
            }
            signal.store(true, Ordering::Release);
        });
        Self {
            queue,
            failed,
            task,
        }
    }
    pub fn healthy(&self) -> bool {
        !self.failed.load(Ordering::Acquire)
    }
    pub fn receive(&mut self) -> Result<Option<Vec<u8>>, Problem> {
        if !self.healthy() {
            return Err(Problem::new(ProblemCode::Installation));
        }
        match self.queue.try_recv() {
            Ok(bytes) => Ok(Some(bytes)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => Err(Problem::new(ProblemCode::Lost)),
        }
    }
}
impl Drop for Receiver {
    fn drop(&mut self) {
        self.task.abort();
    }
}
