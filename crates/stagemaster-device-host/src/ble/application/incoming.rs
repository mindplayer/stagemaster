//! Continuously drain SDK notifications; overflow and partial-record timeout are terminal.
#[cfg(test)]
mod tests;
use super::{C, Problem, RESPONSE, SERVICE, now};
use btleplug::api::ValueNotification;
use futures_util::{StreamExt, stream::BoxStream};
use stagemaster_device_link::secure::{Receiver, Record};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{sync::mpsc, task::JoinHandle, time::Instant};
pub(super) struct Incoming {
    queue: mpsc::Receiver<Record>,
    failed: Arc<AtomicBool>,
    task: JoinHandle<()>,
}
impl Incoming {
    pub fn spawn(
        mut stream: BoxStream<'static, ValueNotification>,
        origin: Instant,
        budget: usize,
    ) -> Self {
        let (sender, queue) = mpsc::channel(4);
        let failed = Arc::new(AtomicBool::new(false));
        let signal = failed.clone();
        let task = tokio::spawn(async move {
            let Ok(mut receiver) = Receiver::new(budget, now(origin)) else {
                signal.store(true, Ordering::Release);
                return;
            };
            loop {
                if receiver.poll(now(origin)).is_err() {
                    break;
                }
                let notification = tokio::select! {
                    next = stream.next() => match next { Some(n) => n, None => break },
                    () = tokio::time::sleep(Duration::from_millis(25)) => continue,
                };
                if notification.service_uuid != SERVICE || notification.uuid != RESPONSE {
                    continue;
                }
                match receiver.push(&notification.value, now(origin)) {
                    Ok(Some(record)) => {
                        if sender.try_send(record).is_err() {
                            break;
                        }
                    }
                    Ok(None) => (),
                    Err(_) => break,
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
    pub fn next(&mut self) -> Result<Option<Record>, Problem> {
        if !self.healthy() {
            return Err(Problem::new(C::Installation));
        }
        match self.queue.try_recv() {
            Ok(record) => Ok(Some(record)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => Err(Problem::new(C::Lost)),
        }
    }
    pub async fn receive(&mut self) -> Result<Record, Problem> {
        let record = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Some(record) = self.next()? {
                    break Ok::<_, Problem>(record);
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .map_err(|_| Problem::new(C::Timeout))??;
        Ok(record)
    }
}
impl Drop for Incoming {
    fn drop(&mut self) {
        self.task.abort();
    }
}
