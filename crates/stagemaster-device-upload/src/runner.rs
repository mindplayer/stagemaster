use crate::{
    Connection, Phase, Receipt,
    package::hex,
    state::{Job, State},
};
use futures_util::FutureExt;
use stagemaster_transfer::{Command, Outcome, Request, Response, UploadError};
use std::{
    panic::AssertUnwindSafe,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;

struct Failure {
    message: String,
    reconnect: bool,
}
impl Failure {
    fn link(message: String) -> Self {
        Self {
            message,
            reconnect: true,
        }
    }
    fn upload(error: &UploadError) -> Self {
        Self {
            reconnect: !matches!(error, UploadError::Remote(_)),
            message: error.to_string(),
        }
    }
}
pub(crate) async fn run<C: Connection>(
    inner: Arc<Mutex<State>>,
    connection: Arc<C>,
    mut stop: watch::Receiver<bool>,
    id: String,
    mut job: Job,
) {
    let result = if *stop.borrow() {
        Err(Failure::link("应用已结束安装通信，设备结果仍需核对".into()))
    } else {
        tokio::select! {
            biased;
            _ = stop.changed() => {
                Err(Failure::link("应用已结束安装通信，设备结果仍需核对".into()))
            },
            result = AssertUnwindSafe(drive(&inner, &*connection, &id, &mut job)).catch_unwind() => {
                result.unwrap_or_else(|_| {
                    Err(Failure::link("安装通信发生异常，请重连原设备核对结果".into()))
                })
            },
        }
    };
    if let Ok(mut state) = inner.lock() {
        let Some(task) = state.view.task.as_mut().filter(|t| t.id == id) else {
            return;
        };
        task.running = false;
        if result
            .as_ref()
            .err()
            .is_some_and(|failure| failure.reconnect)
        {
            job.prepared.upload.disconnect();
            job.connected = false;
        }
        let cursor = (job.target.clone(), job.prepared.upload.next_request_id());
        match result {
            Ok(outcome) => {
                task.problem = None;
                task.phase = match outcome {
                    Outcome::Installed(commit) => {
                        task.confirmed_bytes = commit.identity.bytes;
                        task.receipt = Some(Receipt {
                            generation: commit.generation.to_string(),
                            digest: hex(&commit.identity.digest),
                            bytes: commit.identity.bytes,
                        });
                        Phase::Installed
                    }
                    Outcome::Cancelled => Phase::Cancelled,
                    Outcome::NotStarted => Phase::NotStarted,
                };
                state.job = None;
            }
            Err(error) => {
                task.phase = if error.reconnect {
                    Phase::Reconnect
                } else {
                    Phase::Failed
                };
                task.problem = Some(error.message);
                state.job = Some(job);
            }
        }
        state.cursor = Some(cursor);
        state.touch();
    }
}
async fn drive<C: Connection>(
    inner: &Mutex<State>,
    connection: &C,
    id: &str,
    job: &mut Job,
) -> Result<Outcome, Failure> {
    loop {
        let current = connection.target(job.target.epoch).map_err(Failure::link)?;
        if current.peer != job.target.peer {
            return Err(Failure::link("设备认证会话已经变化，请重连后核对".into()));
        }
        current
            .check(&job.prepared.info, &hex(&job.target.peer.device))
            .map_err(Failure::link)?;
        {
            let mut state = inner
                .lock()
                .map_err(|_| Failure::link("安装任务状态不可用".into()))?;
            let task = state.task(id).map_err(Failure::link)?;
            if task.cancel_requested && !job.cancel_applied {
                job.prepared.upload.request_cancel();
                job.cancel_applied = true;
            }
        }
        let Some(frame) = job
            .prepared
            .upload
            .outbound()
            .map_err(|error| Failure::upload(&error))?
            .cloned()
        else {
            let outcome = job
                .prepared
                .upload
                .outcome()
                .ok_or_else(|| Failure::link("安装结果缺失，请重新核对".into()))?;
            if matches!(outcome,Outcome::Installed(commit) if commit.identity!=job.prepared.upload.identity())
            {
                return Err(Failure::link("设备提交与当前播放包不一致".into()));
            }
            return Ok(outcome);
        };
        let command = Request::decode(frame.bytes())
            .map_err(|e| Failure::link(e.to_string()))?
            .action
            .command();
        publish(inner, id, job, Some(command))?;
        let response = connection
            .exchange(job.target.epoch, frame)
            .await
            .map_err(Failure::link)?;
        let wire = Response::decode(response.bytes()).map_err(|e| Failure::link(e.to_string()))?;
        if wire.state.boot != job.target.peer.boot
            || wire.state.max_chunk > usize::from(job.target.limits.chunk_bytes)
        {
            return Err(Failure::link("设备回执与当前启动或传输预算不一致".into()));
        }
        let result = job
            .prepared
            .upload
            .accept(response.bytes())
            .map_err(|error| Failure::upload(&error));
        publish(inner, id, job, None)?;
        result?;
        tokio::task::yield_now().await;
    }
}
fn publish(
    inner: &Mutex<State>,
    id: &str,
    job: &Job,
    command: Option<Command>,
) -> Result<(), Failure> {
    let mut state = inner
        .lock()
        .map_err(|_| Failure::link("安装任务状态不可用".into()))?;
    let task = state.task(id).map_err(Failure::link)?;
    if let Some(response) = job.prepared.upload.state() {
        task.confirmed_bytes = response
            .progress
            .filter(|p| p.identity == job.prepared.upload.identity() && response.owned)
            .map_or(0, |p| p.received);
    }
    if let Some(command) = command {
        task.phase = if task.cancel_requested {
            Phase::Cancelling
        } else {
            match command {
                Command::Status | Command::Begin | Command::Reconcile => Phase::Querying,
                Command::Write => Phase::Transferring,
                Command::Verify => Phase::Verifying,
                Command::Commit => Phase::Committing,
                Command::Cancel => Phase::Cancelling,
            }
        };
    }
    state.touch();
    Ok(())
}
