use crate::{
    Candidate, DeviceDescription, Diagnostics, MAX_CANDIDATES, Phase, Problem, ProblemCode as C,
    Request, Snapshot, Transport,
};
use futures_util::FutureExt;
use stagemaster_device_link::client::{Client, Diagnostics as WireDiagnostics, Error as WireError};
use std::panic::AssertUnwindSafe;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use tokio::{
    sync::{Mutex as AsyncMutex, watch},
    time::{Instant, sleep, timeout},
};

const IO_TIMEOUT: Duration = Duration::from_millis(2500);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const SCAN_DURATION: Duration = Duration::from_secs(8);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(5);
const FRESH_MS: u64 = 4500;

#[derive(Clone, Copy)]
struct Stamp {
    monotonic: Instant,
    wall: SystemTime,
}
impl Stamp {
    fn now() -> Self {
        Self {
            monotonic: Instant::now(),
            wall: SystemTime::now(),
        }
    }
    fn age(self) -> u64 {
        // Wall clock only invalidates connections across suspend/clock changes.
        // It never grants liveness or acts as a playback/licensing clock.
        let wall = self.wall.elapsed().map_or(Duration::MAX, |v| v);
        millis(self.monotonic.elapsed().max(wall))
    }
}
fn millis(value: Duration) -> u64 {
    u64::try_from(value.as_millis()).unwrap_or(u64::MAX)
}

struct Inner {
    snapshot: Snapshot,
    cancel: Option<watch::Sender<bool>>,
    last_reply: Option<Stamp>,
    closed: bool,
}
impl Inner {
    fn touch(&mut self) {
        self.snapshot.revision += 1;
    }
    fn clear_live(&mut self) {
        self.snapshot.diagnostics = None;
        self.snapshot.description = None;
        self.snapshot.round_trip_ms = None;
        self.snapshot.last_reply_age_ms = None;
        self.last_reply = None;
    }
    fn snapshot(&mut self) -> Snapshot {
        if let Some(last) = self.last_reply {
            let age = last.age();
            if self.snapshot.phase == Phase::Connected && age >= FRESH_MS {
                self.snapshot.phase = Phase::Stopping;
                self.snapshot.problem = Some(Problem::new(C::Timeout));
                self.clear_live();
                if let Some(cancel) = &self.cancel {
                    let _ = cancel.send(true);
                }
                self.touch();
            } else {
                self.snapshot.last_reply_age_ms = Some(age);
            }
        }
        self.snapshot.clone()
    }
}

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

fn update(inner: &Mutex<Inner>, epoch: u32, change: impl FnOnce(&mut Inner)) {
    if let Ok(mut state) = inner.lock()
        && state.snapshot.epoch == epoch
        && state.snapshot.phase != Phase::Stopping
        && !state.closed
    {
        change(&mut state);
        state.touch();
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
    let Some(id) = id else {
        return scan(inner, backend, epoch).await;
    };
    timeout(CONNECT_TIMEOUT, backend.connect(id))
        .await
        .map_err(|_| Problem::new(C::Timeout))??;
    let mut client = Client::default();
    let mut last = exchange(inner, backend, epoch, &mut client, false).await?;
    loop {
        sleep(Duration::from_secs(2)).await;
        if last.age() >= FRESH_MS {
            return Err(Problem::new(C::Timeout));
        }
        last = exchange(inner, backend, epoch, &mut client, true).await?;
    }
}

async fn scan<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
) -> Result<(), Problem> {
    timeout(CONNECT_TIMEOUT, backend.start_scan())
        .await
        .map_err(|_| Problem::new(C::SetupTimeout))??;
    update(inner, epoch, |state| {
        state.snapshot.phase = Phase::Scanning;
        state.snapshot.scan_performed = true;
    });
    let deadline = Instant::now() + SCAN_DURATION;
    loop {
        let candidate = tokio::select! {
            biased;
            () = tokio::time::sleep_until(deadline) => return Ok(()),
            value = backend.discover() => value?,
        };
        update(inner, epoch, |state| {
            add_candidate(&mut state.snapshot, candidate);
        });
    }
}

fn add_candidate(snapshot: &mut Snapshot, candidate: Candidate) {
    if let Some(existing) = snapshot
        .candidates
        .iter_mut()
        .find(|c| c.id == candidate.id)
    {
        *existing = candidate;
    } else if snapshot.candidates.len() < MAX_CANDIDATES {
        snapshot.candidates.push(candidate);
    } else {
        snapshot.truncated = true;
    }
}

async fn exchange<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
    client: &mut Client,
    heartbeat: bool,
) -> Result<Stamp, Problem> {
    let started = Instant::now();
    let stamp = timeout(IO_TIMEOUT, async {
        let request = client.request().map_err(protocol)?;
        backend.write(&request).await?;
        // Firmware stores the application receipt before acknowledging ATT write.
        // A mismatched cache may be read again; a request is NEVER resent.
        loop {
            let bytes = backend.reply().await?;
            match client.accept(&bytes) {
                Ok(()) => break,
                Err(WireError::Correlation) => sleep(Duration::from_millis(30)).await,
                Err(value) => return Err(protocol(value)),
            }
        }
        let stamp = Stamp::now();
        let diagnostics =
            WireDiagnostics::decode(&backend.diagnostics().await?).map_err(protocol)?;
        let description = if heartbeat {
            None
        } else {
            backend
                .description()
                .await?
                .map(|bytes| DeviceDescription::decode(&bytes, client.session_id().unwrap_or(0)))
                .transpose()?
        };
        Ok::<_, Problem>((stamp, Diagnostics::from(diagnostics), description))
    })
    .await
    .map_err(|_| Problem::new(C::Timeout))??;
    update(inner, epoch, |state| {
        state.snapshot.phase = Phase::Connected;
        state.snapshot.diagnostics = Some(stamp.1);
        if !heartbeat {
            state.snapshot.description = stamp.2;
        }
        state.snapshot.round_trip_ms = Some(millis(started.elapsed()));
        state.snapshot.heartbeat_count += u64::from(heartbeat);
        state.last_reply = Some(stamp.0);
        state.snapshot.last_reply_age_ms = Some(stamp.0.age());
    });
    Ok(stamp.0)
}
fn protocol(error: WireError) -> Problem {
    Problem::new(C::Protocol).detail(format!("{error:?}"))
}
