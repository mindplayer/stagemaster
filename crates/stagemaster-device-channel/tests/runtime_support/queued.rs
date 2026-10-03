use crate::runtime_support::{description, device::Device, rights, scopes};
use crate::support::{now, peer::Peer};
use stagemaster_device_channel::RecordIo;
use stagemaster_install_worker::{
    Epoch,
    runtime_queue::{Command, Completion, Endpoint, Gateway, Live},
};
use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};
use tokio::{task::JoinHandle, time::Instant};

// Real bounded queues and an independent blocking storage thread. Test adaptation only;
// production firmware supplies its own cross-core channel and synchronized latest slot.
struct Worker {
    live: Arc<Mutex<Option<Live>>>,
    requests: Option<mpsc::SyncSender<Command>>,
    completions: mpsc::Receiver<Completion>,
    task: Option<JoinHandle<Device>>,
}
impl Worker {
    fn spawn(mut device: Device, origin: Instant) -> Self {
        let live = Arc::new(Mutex::new(None));
        let latest = live.clone();
        let (requests, input) = mpsc::sync_channel(1);
        let (output, completions) = mpsc::sync_channel(1);
        let task = tokio::task::spawn_blocking(move || {
            let mut endpoint = Endpoint::default();
            let read = || *latest.lock().unwrap();
            let mut pending = None;
            loop {
                endpoint.tick(&mut device, now(origin), read).unwrap();
                if read().is_none() {
                    pending = None;
                }
                if let Some(completion) = pending.take() {
                    match output.try_send(completion) {
                        Ok(()) => {}
                        Err(mpsc::TrySendError::Full(completion)) => {
                            pending = Some(completion);
                            std::thread::sleep(Duration::from_millis(2));
                            continue;
                        }
                        Err(mpsc::TrySendError::Disconnected(_)) => break,
                    }
                }
                match input.recv_timeout(Duration::from_millis(5)) {
                    Ok(command) => {
                        pending =
                            Some(endpoint.process(&mut device, command, || now(origin), read));
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            endpoint.tick(&mut device, now(origin), || None).unwrap();
            device
        });
        Self {
            live,
            requests: Some(requests),
            completions,
            task: Some(task),
        }
    }
    fn publish(&self, gateway: &mut Gateway, now: u64) {
        *self.live.lock().unwrap() = gateway.live(now);
    }
    async fn finish(mut self) -> Device {
        *self.live.lock().unwrap() = None;
        self.requests.take();
        self.task.take().unwrap().await.unwrap()
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        *self.live.lock().unwrap() = None;
        self.requests.take();
    }
}

pub async fn serve<R: RecordIo>(io: R, device: Device, origin: Instant) -> Device {
    let Peer { mut io, secure, .. } =
        Peer::authenticate_with(io, description(7), Some(scopes(rights())), origin)
            .await
            .unwrap();
    let mut gateway = Gateway::new(secure, Epoch::new(1).unwrap(), now(origin)).unwrap();
    let worker = Worker::spawn(device, origin);
    worker.publish(&mut gateway, now(origin));
    loop {
        if gateway.poll(now(origin)).is_err() {
            break;
        }
        match io.try_receive() {
            Ok(Some(bytes)) => {
                let result = gateway.receive(&bytes, now(origin));
                worker.publish(&mut gateway, now(origin));
                match result {
                    Ok(Some(command)) => {
                        worker.requests.as_ref().unwrap().try_send(command).unwrap();
                    }
                    Ok(None) => {}
                    Err(_) => break,
                }
            }
            Ok(None) => {}
            Err(_) => break,
        }
        if let Ok(completion) = worker.completions.try_recv() {
            gateway.complete(completion, now(origin)).unwrap();
        }
        let outgoing = gateway.outbound(now(origin)).unwrap().map(<[u8]>::to_vec);
        worker.publish(&mut gateway, now(origin));
        if let Some(bytes) = outgoing {
            if io.send(&bytes).await.is_err() {
                break;
            }
            gateway.sent(now(origin)).unwrap();
            worker.publish(&mut gateway, now(origin));
        }
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    gateway.close();
    io.close();
    worker.finish().await
}
