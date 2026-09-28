use crate::{
    Connection, Phase, Prepared, Snapshot, Task, runner,
    state::{Job, State},
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    sync::watch,
    time::{sleep, timeout},
};

/// App-owned task. UI panels may unmount without cancelling it.
pub struct Service<C: Connection> {
    pub(crate) inner: Arc<Mutex<State>>,
    pub(crate) connection: Arc<C>,
    stop: watch::Sender<bool>,
}
impl<C: Connection> Service<C> {
    #[must_use]
    pub fn new(connection: Arc<C>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(State::default())),
            connection,
            stop: watch::channel(false).0,
        }
    }
    /// # Errors
    /// Report a failed task-state lock rather than return stale progress.
    pub fn snapshot(&self) -> Result<Snapshot, String> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| "安装任务状态不可用")?
            .view
            .clone())
    }
    /// Starts one immutable package on the explicitly selected authenticated device.
    /// # Errors
    /// Reject an unresolved task, changed target/epoch, missing authentication or excessive package.
    pub fn start(
        &self,
        mut prepared: Prepared,
        epoch: u32,
        expected_device: &str,
    ) -> Result<Snapshot, String> {
        let mut state = self.inner.lock().map_err(|_| "安装任务状态不可用")?;
        if state.closed {
            return Err("安装任务服务已关闭".into());
        }
        if state
            .view
            .task
            .as_ref()
            .is_some_and(|t| !t.phase.terminal())
        {
            return Err("请先处理当前安装任务".into());
        }
        let target = self.connection.target(epoch)?;
        target.check(&prepared.info, expected_device)?;
        let next_id = state.next_id(&target)?;
        prepared
            .upload
            .connect_at(target.peer.session, next_id)
            .map_err(|e| e.to_string())?;
        let id = uuid::Uuid::new_v4().to_string();
        state.view.task = Some(Task {
            id: id.clone(),
            package: prepared.info.clone(),
            device_id: expected_device.into(),
            device_name: target.name.clone(),
            connection_epoch: target.epoch,
            phase: Phase::Querying,
            running: true,
            cancel_requested: false,
            confirmed_bytes: 0,
            receipt: None,
            problem: None,
        });
        state.job = None;
        state.cursor = Some((target.clone(), None));
        state.touch();
        self.spawn(
            id,
            Job {
                prepared,
                target,
                connected: true,
                cancel_applied: false,
            },
        );
        Ok(state.view.clone())
    }
    /// Explicitly resume on the same device; uncertain transport needs a fresh authenticated link.
    /// # Errors
    /// Reject stale tasks, active/finished work, another device or incompatible/reused connections.
    pub fn resume(&self, id: &str, epoch: u32) -> Result<Snapshot, String> {
        let mut state = self.inner.lock().map_err(|_| "安装任务状态不可用")?;
        let task = state.task(id)?;
        if task.running || task.phase.terminal() {
            return Err("当前任务不需要恢复或正在处理".into());
        }
        let target = self.connection.target(epoch)?;
        target.check(&task.package, &task.device_id)?;
        let mut job = state.job.take().ok_or("安装任务的原始播放包不可用")?;
        if let Err(error) = job.resume(target) {
            state.job = Some(job);
            return Err(error);
        }
        let task = state.task(id)?;
        task.running = true;
        task.connection_epoch = epoch;
        task.problem = None;
        task.phase = if task.cancel_requested {
            Phase::Cancelling
        } else {
            Phase::Querying
        };
        state.touch();
        self.spawn(id.into(), job);
        Ok(state.view.clone())
    }
    /// Record a sticky cancel intent. An in-flight message is allowed to finish.
    /// # Errors
    /// Reject stale/finished tasks; absence of a connection does not pretend cancellation succeeded.
    pub fn cancel(&self, id: &str) -> Result<Snapshot, String> {
        let resume = {
            let mut state = self.inner.lock().map_err(|_| "安装任务状态不可用")?;
            let task = state.task(id)?;
            if task.phase.terminal() {
                return Err("任务已经结束，无需取消".into());
            }
            task.cancel_requested = true;
            let running = task.running;
            if running {
                task.phase = Phase::Cancelling;
            }
            state.touch();
            if running {
                None
            } else {
                state.job.as_ref().map(|job| job.target.epoch)
            }
        };
        if let Some(epoch) = resume
            && let Err(error) = self.resume(id, epoch)
        {
            let mut state = self.inner.lock().map_err(|_| "安装任务状态不可用")?;
            let task = state.task(id)?;
            task.problem = Some(format!("取消尚未确认：{error}。请连接原设备后继续核对"));
            state.touch();
        }
        self.snapshot()
    }
    /// Forget local state only. The UI must explicitly explain unresolved remote effects.
    /// # Errors
    /// Reject stale tasks or any running exchange; never cancels a remote transaction.
    pub fn forget(&self, id: &str) -> Result<Snapshot, String> {
        let mut state = self.inner.lock().map_err(|_| "安装任务状态不可用")?;
        if state.task(id)?.running {
            return Err("任务仍在处理，请先取消或等待通信结束".into());
        }
        state.job = None;
        state.view.task = None;
        state.touch();
        Ok(state.view.clone())
    }
    fn spawn(&self, id: String, job: Job) {
        tokio::spawn(runner::run(
            self.inner.clone(),
            self.connection.clone(),
            self.stop.subscribe(),
            id,
            job,
        ));
    }
    /// Stop local work before the connection service is shut down. Remote state may need recovery.
    /// # Errors
    /// Report failure to observe worker shutdown instead of claiming remote cancellation.
    pub async fn shutdown(&self) -> Result<(), String> {
        self.begin_shutdown();
        timeout(Duration::from_secs(1), async {
            loop {
                if !self.snapshot()?.task.is_some_and(|t| t.running) {
                    return Ok(());
                }
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_| "安装通信尚未停止".to_string())?
    }
    /// Prevent new work and stop local communication; never claims remote cancellation.
    pub fn begin_shutdown(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.closed = true;
        }
        let _ = self.stop.send(true);
    }
}
impl<C: Connection> Drop for Service<C> {
    fn drop(&mut self) {
        self.begin_shutdown();
    }
}
