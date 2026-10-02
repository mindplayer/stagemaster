use crate::{
    application::Application,
    commands,
    preparation::Prepared,
    sessions::{Admission, Change, Completion, MAX_SESSIONS, Registry},
    wire::{Failure, Input, RecordView},
};
use serde_json::{Value, json};
use stagemaster_runtime_host::{Host, Observer, WaitError};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{Notify, Semaphore};
use uuid::Uuid;

pub(crate) struct Service<M: Application> {
    pub id: Uuid,
    pub observer: Observer<M>,
    pub adapter: M::Context,
    pub source: Value,
    host: Mutex<Host<M>>,
    sessions: Mutex<Registry<M>>,
    workers: Arc<Semaphore>,
    pub shutdown: Notify,
}
impl<M: Application> Service<M> {
    pub fn new(prepared: Prepared<M>) -> Arc<Self> {
        Arc::new(Self {
            id: prepared.boot,
            adapter: prepared.adapter,
            observer: prepared.host.observer(),
            source: prepared.source,
            host: Mutex::new(prepared.host),
            sessions: Mutex::new(Registry::new()),
            workers: Arc::new(Semaphore::new(MAX_SESSIONS)),
            shutdown: Notify::new(),
        })
    }
    pub fn available(&self) -> bool {
        self.sessions.lock().is_ok_and(|s| s.accepting)
    }
    pub fn create_session(&self) -> Result<Uuid, Failure> {
        self.sessions
            .lock()
            .map_err(|_| Failure::busy())?
            .create(Instant::now())
    }
    pub fn submit(self: &Arc<Self>, id: Uuid, input: Input) -> Result<RecordView, Failure> {
        let permit = self.workers.clone().try_acquire_owned().ok();
        let serial = input.serial;
        let admission = self.sessions.lock().map_err(|_| Failure::busy())?.begin(
            id,
            input,
            Instant::now(),
            permit.is_some(),
        )?;
        match admission {
            Admission::Existing(view) => Ok(view),
            Admission::New(job) => {
                let service = self.clone();
                // Spawn synchronously after admission, before any HTTP await/cancellation point.
                tokio::task::spawn_blocking(move || {
                    let _permit = permit;
                    let completion=catch_unwind(AssertUnwindSafe(||commands::run(&service.host,&service.adapter,&job))).unwrap_or_else(|_|Completion {
                        outcome:json!({"kind":"unknown","code":"workerFailed","message":"无法确认操作结果，请读取状态并重新取得控制权"}),change:Change::Clear,
                    });
                    if let Ok(mut sessions) = service.sessions.lock() {
                        sessions.finish(id, serial.0, completion, Instant::now());
                    }
                });
                Ok(RecordView {
                    serial,
                    status: "pending",
                    outcome: None,
                })
            }
        }
    }
    pub fn receipt(&self, id: Uuid, serial: u64) -> Result<RecordView, Failure> {
        self.sessions
            .lock()
            .map_err(|_| Failure::busy())?
            .receipt(id, serial, Instant::now())
    }
    pub fn begin_shutdown(&self) -> Result<(), Failure> {
        let mut registry = self.sessions.lock().map_err(|_| Failure::busy())?;
        registry.accepting = false;
        self.shutdown.notify_one();
        Ok(())
    }
    pub async fn join(self: &Arc<Self>) -> Result<(), String> {
        let service = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut host = service
                .host
                .lock()
                .map_err(|_| "执行宿主锁不可用".to_string())?;
            loop {
                match host.shutdown(Duration::from_secs(1)) {
                    Ok(_) => return Ok(()),
                    Err(WaitError::Timeout) => {} // Same handle, never reconstruct the host.
                    Err(e) => return Err(e.to_string()),
                }
            }
        })
        .await
        .map_err(|_| "执行宿主关闭任务异常".to_string())?
    }
}
