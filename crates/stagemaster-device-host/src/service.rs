mod connected;
mod diagnostics;
mod installation;
mod scan;
mod state;

use crate::{Phase, Problem, ProblemCode as C, Request, Snapshot, Transport};
use futures_util::FutureExt;
use state::{Inner, update};
use std::{
    panic::AssertUnwindSafe,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    sync::{Mutex as AsyncMutex, watch},
    time::{sleep, timeout},
};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Independent from UI lifetime. Call shutdown at application exit.
pub struct Service<B: Transport> {
    inner: Arc<Mutex<Inner>>,
    backend: Arc<AsyncMutex<B>>,
}
impl<B: Transport> Drop for Service<B> {
    fn drop(&mut self) {
        // A forgotten explicit shutdown must not leave a detached heartbeat loop.
        // Applications still await shutdown to observe cleanup failures before exit.
        if let Ok(mut state) = self.inner.lock() {
            state.closed = true;
            if let Some(cancel) = &state.cancel {
                let _ = cancel.send(true);
            }
        }
    }
}
impl<B: Transport> Service<B> {
    #[must_use]
    pub fn new(backend: B) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                snapshot: Snapshot::default(),
                cancel: None,
                last_reply: None,
                closed: false,
                installation: None,
                install_busy: std::sync::Weak::new(),
            })),
            backend: Arc::new(AsyncMutex::new(backend)),
        }
    }

    /// Starts work without waiting for BLE; status is also available without a scan.
    /// Must be called on a Tokio runtime. Errors are Chinese product failures.
    /// # Errors
    /// Rejects old operation epochs, overlapping work, unknown handles or closed service.
    pub fn request(&self, request: Request) -> Result<Snapshot, Problem> {
        let mut state = self.inner.lock().map_err(|_| Problem::new(C::Closed))?;
        if state.closed {
            return Err(Problem::new(C::Closed));
        }
        let snapshot = state.snapshot();
        let (epoch, id) = match request {
            Request::Status => return Ok(snapshot),
            Request::Scan { epoch } => (epoch, None),
            Request::Connect { epoch, id } => (epoch, Some(id)),
            Request::Cancel { epoch } => {
                if epoch != snapshot.epoch {
                    return Err(Problem::new(C::Stale));
                }
                if !matches!(
                    snapshot.phase,
                    Phase::Preparing | Phase::Scanning | Phase::Connecting | Phase::Connected
                ) {
                    return Err(Problem::new(C::Busy));
                }
                state.snapshot.phase = Phase::Stopping;
                state.clear_live();
                if let Some(cancel) = &state.cancel {
                    let _ = cancel.send(true);
                }
                state.touch();
                return Ok(state.snapshot());
            }
        };
        if epoch != snapshot.epoch {
            return Err(Problem::new(C::Stale));
        }
        if snapshot.phase == Phase::Blocked {
            return Err(Problem::new(C::Cleanup));
        }
        if !matches!(snapshot.phase, Phase::Idle | Phase::Fault) {
            return Err(Problem::new(C::Busy));
        }
        let selected = if let Some(id) = &id {
            Some(
                snapshot
                    .candidates
                    .iter()
                    .find(|c| c.id == *id)
                    .cloned()
                    .ok_or_else(|| Problem::new(C::Unavailable))?,
            )
        } else {
            None
        };
        let epoch = snapshot
            .epoch
            .checked_add(1)
            .ok_or_else(|| Problem::new(C::Closed))?;
        state.snapshot.epoch = epoch;
        state.snapshot.phase = if id.is_some() {
            Phase::Connecting
        } else {
            Phase::Preparing
        };
        state.snapshot.problem = None;
        state.snapshot.selected = selected;
        state.snapshot.heartbeat_count = 0;
        state.clear_live();
        if id.is_none() {
            state.snapshot.candidates.clear();
            state.snapshot.scan_performed = false;
            state.snapshot.truncated = false;
        }
        let (cancel, rx) = watch::channel(false);
        state.cancel = Some(cancel);
        state.touch();
        let inner = self.inner.clone();
        let backend = self.backend.clone();
        tokio::spawn(async move {
            run(inner, backend, epoch, id, rx).await;
        });
        Ok(state.snapshot())
    }

    /// Cancels all work and waits a bounded time for adapter cleanup.
    /// # Errors
    /// Reports cleanup uncertainty instead of claiming resources were released.
    pub async fn shutdown(&self) -> Result<(), Problem> {
        {
            let mut state = self.inner.lock().map_err(|_| Problem::new(C::Closed))?;
            state.closed = true;
            if let Some(cancel) = &state.cancel {
                let _ = cancel.send(true);
            }
        }
        timeout(CLEANUP_TIMEOUT + Duration::from_secs(1), async {
            loop {
                let done = {
                    let state = self.inner.lock().map_err(|_| Problem::new(C::Closed))?;
                    if state.snapshot.phase == Phase::Blocked {
                        return Err(Problem::new(C::Cleanup));
                    }
                    state.cancel.is_none()
                };
                if done {
                    return Ok(());
                }
                sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .map_err(|_| Problem::new(C::Cleanup))?
    }
}

async fn run<B: Transport>(
    inner: Arc<Mutex<Inner>>,
    backend: Arc<AsyncMutex<B>>,
    epoch: u32,
    id: Option<String>,
    mut cancel: watch::Receiver<bool>,
) {
    let mut backend = backend.lock().await;
    let result = tokio::select! {
        biased;
        _ = cancel.changed() => Ok(()),
        result = AssertUnwindSafe(operate(&inner, &mut *backend, epoch, id.as_deref())).catch_unwind() =>
            result.unwrap_or_else(|_| Err(Problem::new(C::Cleanup).detail("蓝牙适配器内部异常".into()))),
    };
    // Both paths use cleanup; never abort the task itself or reuse its adapter early.
    if let Ok(mut state) = inner.lock() {
        state.snapshot.phase = Phase::Stopping;
        state.clear_live();
        state.touch();
    }
    let cleanup = timeout(
        CLEANUP_TIMEOUT,
        AssertUnwindSafe(async {
            if id.is_some() {
                backend.disconnect().await
            } else {
                backend.stop_scan().await
            }
        })
        .catch_unwind(),
    )
    .await;
    if let Ok(mut state) = inner.lock() {
        let problem = match cleanup {
            Ok(Ok(Ok(()))) => result.err().or_else(|| state.snapshot.problem.take()),
            Ok(Ok(Err(value))) => Some(Problem::new(C::Cleanup).detail(value.to_string())),
            Err(_) | Ok(Err(_)) => Some(Problem::new(C::Cleanup)),
        };
        state.snapshot.phase = match problem.as_ref().map(|p| p.code) {
            Some(C::Cleanup) => Phase::Blocked,
            Some(_) => Phase::Fault,
            None => Phase::Idle,
        };
        state.snapshot.problem = problem;
        state.cancel = None;
        state.clear_live();
        state.touch();
    }
}

async fn operate<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
    id: Option<&str>,
) -> Result<(), Problem> {
    if let Some(id) = id {
        connected::run(inner, backend, epoch, id).await
    } else {
        scan::run(inner, backend, epoch).await
    }
}
