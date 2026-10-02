use crate::{
    Backend, Configuration, Connection, Deadline, Device, Error, Fault, Observer, Phase, Profile,
    QUEUE_CAPACITY, WaitError,
    client::{Command, Ingress},
    observation::Shared,
    worker,
};
use stagemaster_package::ReadAt;
use stagemaster_runtime::{Grant, PlaybackPolicy, Runtime};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

/// Trusted process owner. Never expose its acquire/takeover/shutdown methods directly to peers.
pub struct Host<M: Profile = Device> {
    ingress: Ingress<M>,
    thread: Option<thread::JoinHandle<()>>,
    done: mpsc::Receiver<()>,
}
impl Host {
    /// Transfer a fully prepared, idle runtime to its only scheduling owner.
    /// Policy implementations must be bounded, nonblocking and free of I/O.
    /// # Errors
    /// Reject invalid cadence, non-idle/unloaded/owned runtimes, or thread creation failure.
    pub fn start<R: ReadAt + Send + 'static, P: PlaybackPolicy + Send + 'static>(
        runtime: Runtime<R, P>,
        configuration: Configuration,
    ) -> Result<Self, Error> {
        Self::start_backend(runtime, configuration)
    }
}
impl<M: Profile> Host<M> {
    /// Transfer a prepared backend to the existing single scheduling worker.
    /// # Errors
    /// Reject invalid cadence, unprepared state or thread creation failure.
    pub fn start_backend<B: Backend<Profile = M>>(
        runtime: B,
        configuration: Configuration,
    ) -> Result<Self, Error> {
        if !(Duration::from_millis(1)..=Duration::from_millis(100)).contains(&configuration.period)
        {
            return Err(Error::Configuration);
        }
        if !runtime.is_prepared() {
            return Err(Error::NotPrepared);
        }
        let state = runtime.state();
        let shared = Arc::new(Shared::<M>::new(state));
        let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
        let (completed, done) = mpsc::sync_channel(1);
        let owner = shared.clone();
        let thread = thread::Builder::new()
            .name("stagemaster-runtime".into())
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    worker::run(runtime, configuration, receiver, &owner)
                }));
                owner.finish(match result {
                    Ok(Ok(())) => None,
                    Ok(Err(code)) => Some(Fault::Runtime(code)),
                    Err(_) => Some(Fault::Panic),
                });
                let _ = completed.try_send(());
            })
            .map_err(|_| Error::ThreadSpawn)?;
        Ok(Self {
            ingress: Ingress { sender, shared },
            thread: Some(thread),
            done,
        })
    }
    #[must_use]
    pub fn observer(&self) -> Observer<M> {
        Observer {
            shared: self.ingress.shared.clone(),
        }
    }

    /// Grant comes from a verified operator session; takeover is a trusted policy decision.
    /// # Errors
    /// Admission is bounded. Acquisition itself may refuse active ownership or invalid grants.
    pub fn connect(
        &self,
        grant: Grant,
        takeover: bool,
        ttl: impl Into<Deadline>,
    ) -> Result<Connection<M>, Error> {
        let ticket = self.ingress.send(ttl, |reply| Command::Acquire {
            grant,
            takeover,
            reply,
        })?;
        Ok(Connection {
            ticket,
            ingress: self.ingress.clone(),
        })
    }

    /// Request shutdown outside the command queue. Timeout preserves this handle for another wait.
    /// # Errors
    /// Timeout does not prove completion. A joined worker reports its final phase, including faults.
    pub fn shutdown(&mut self, timeout: Duration) -> Result<Phase, WaitError> {
        self.request_stop();
        if self.thread.is_none() {
            return Ok(self.ingress.shared.phase());
        }
        self.done.recv_timeout(timeout).map_err(|e| match e {
            mpsc::RecvTimeoutError::Timeout => WaitError::Timeout,
            mpsc::RecvTimeoutError::Disconnected => WaitError::Unavailable,
        })?;
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            self.ingress.shared.finish(Some(Fault::Panic));
        }
        Ok(self.ingress.shared.phase())
    }
    fn request_stop(&self) {
        self.ingress.shared.stop();
        if let Some(thread) = &self.thread {
            thread.thread().unpark();
        }
    }
}
impl<M: Profile> Drop for Host<M> {
    fn drop(&mut self) {
        self.request_stop();
    }
}
