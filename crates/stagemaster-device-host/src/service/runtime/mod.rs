pub(in crate::service) mod drive;
use super::{Inner, Service};
use crate::{
    Phase, Problem, ProblemCode as C, Request, RuntimeIntent, RuntimeSnapshot, Snapshot, Transport,
};
use stagemaster_runtime_protocol::{Access, Response};
use std::sync::Arc;
use tokio::{sync::oneshot, time::Instant};

pub(super) struct Call {
    intent: RuntimeIntent,
    reply: oneshot::Sender<Result<Response, Problem>>,
    deadline: Instant,
    lease: Arc<()>,
}
impl Call {
    fn complete(self, result: Result<Response, Problem>) {
        drop(self.lease);
        let _ = self.reply.send(result);
    }
    fn remaining(&self) -> Result<std::time::Duration, Problem> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| Problem::new(C::Timeout))
    }
}

impl<B: Transport> Service<B> {
    /// Explicit native runtime connection; never installs, takes control or starts playback.
    /// Expected access is a local expectation, not a remote permission grant.
    /// # Errors
    /// Reject stale/overlapping work, missing observation scope and unsupported devices.
    pub fn connect_runtime(
        &self,
        epoch: u32,
        id: String,
        expected: Access,
    ) -> Result<Snapshot, Problem> {
        if !expected.observe {
            return Err(Problem::new(C::Runtime));
        }
        self.request_with_runtime(Request::Connect { epoch, id }, Some(expected))
    }

    /// Current admission and historical observations of this connection epoch.
    /// May be read after disconnection; pending work is uncertain, not proof of failure.
    /// # Errors
    /// Reject old epochs, a closed service or poisoned state.
    pub fn runtime_snapshot(&self, epoch: u32) -> Result<RuntimeSnapshot, Problem> {
        let mut state = self.inner.lock().map_err(|_| Problem::new(C::Closed))?;
        check_epoch(&mut state, epoch)?;
        Ok(state.runtime_state.clone())
    }

    /// One typed operation through the existing background connection task. No automatic retry.
    /// Dropping a queued call cancels it; dropping a started call disconnects with uncertainty.
    /// # Errors
    /// Busy, stale, unsupported and transport errors are separate from a confirmed business failure.
    pub async fn exchange_runtime(
        &self,
        epoch: u32,
        intent: RuntimeIntent,
    ) -> Result<Response, Problem> {
        let receive = {
            let mut state = self.inner.lock().map_err(|_| Problem::new(C::Closed))?;
            check_connected(&mut state, epoch)?;
            let (_, sender) = state
                .runtime_calls
                .as_ref()
                .ok_or_else(|| Problem::new(C::Runtime))?;
            if state.runtime_busy.upgrade().is_some() {
                return Err(Problem::new(C::Busy));
            }
            let lease = Arc::new(());
            let busy = Arc::downgrade(&lease);
            let (reply, receive) = oneshot::channel();
            sender
                .try_send(Call {
                    intent,
                    reply,
                    lease,
                    deadline: Instant::now() + stagemaster_device_channel::runtime::REQUEST_TIMEOUT,
                })
                .map_err(|_| Problem::new(C::Lost))?;
            state.runtime_busy = busy;
            receive
        };
        receive.await.map_err(|_| Problem::new(C::Lost))?
    }
}

fn check_epoch(state: &mut Inner, epoch: u32) -> Result<(), Problem> {
    let snapshot = state.snapshot();
    if state.closed {
        return Err(Problem::new(C::Closed));
    }
    if snapshot.epoch != epoch {
        return Err(Problem::new(C::Stale));
    }
    Ok(())
}
fn check_connected(state: &mut Inner, epoch: u32) -> Result<(), Problem> {
    check_epoch(state, epoch)?;
    if state.snapshot.phase != Phase::Connected {
        return Err(Problem::new(C::Lost));
    }
    Ok(())
}
