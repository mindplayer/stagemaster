use super::{
    Result,
    discovery::{NOTIFY, SERVICE},
};
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

pub struct Incoming {
    task: JoinHandle<()>,
    queue: mpsc::Receiver<Record>,
    failed: Arc<AtomicBool>,
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
            let mut receiver = Receiver::new(budget, 0).unwrap();
            loop {
                let now = u64::try_from(origin.elapsed().as_millis()).unwrap();
                if let Err(error) = receiver.poll(now) {
                    eprintln!("接收期限：{error}");
                    break;
                }
                let notification = tokio::select! {
                    next = stream.next() => match next { Some(n) => n, None => break },
                    () = tokio::time::sleep(Duration::from_millis(25)) => continue,
                };
                if notification.service_uuid != SERVICE || notification.uuid != NOTIFY {
                    continue;
                }
                let now = u64::try_from(origin.elapsed().as_millis()).unwrap();
                match receiver.push(&notification.value, now) {
                    Ok(Some(record)) => {
                        if sender.try_send(record).is_err() {
                            break;
                        }
                    }
                    Ok(None) => (),
                    Err(error) => {
                        eprintln!("接收分片：{error}");
                        break;
                    }
                }
            }
            signal.store(true, Ordering::Release);
        });
        Self {
            task,
            queue,
            failed,
        }
    }
    pub async fn receive(&mut self) -> Result<Record> {
        if self.failed.load(Ordering::Acquire) {
            return Err("通知通道已失败".into());
        }
        let record = tokio::time::timeout(Duration::from_secs(5), self.queue.recv())
            .await?
            .ok_or("通知通道已结束")?;
        if self.failed.load(Ordering::Acquire) {
            return Err("通知通道已失败".into());
        }
        Ok(record)
    }
}
impl Drop for Incoming {
    fn drop(&mut self) {
        self.task.abort();
    }
}
