mod pending;
pub(super) use pending::Pending;

use super::Service;
use crate::{InstallationPeer, Phase, Problem, ProblemCode as C, Transport};
use stagemaster_transfer::Frame;
use std::sync::Arc;
use tokio::sync::oneshot;

pub(super) struct Call {
    pub frame: Frame,
    pub reply: oneshot::Sender<Result<Frame, Problem>>,
    lease: Arc<()>,
}
impl Call {
    pub fn complete(self, result: Result<Frame, Problem>) {
        drop(self.lease);
        let _ = self.reply.send(result);
    }
}

impl<B: Transport> Service<B> {
    /// Authenticated facts, scoped to a fresh physical connection. None is diagnostic-only.
    /// # Errors
    /// Reject closed, stale, disconnected or expired connections.
    pub fn installation_peer(&self, epoch: u32) -> Result<Option<InstallationPeer>, Problem> {
        let mut state = self.inner.lock().map_err(|_| Problem::new(C::Closed))?;
        let snapshot = state.snapshot();
        if state.closed {
            return Err(Problem::new(C::Closed));
        }
        if snapshot.epoch != epoch {
            return Err(Problem::new(C::Stale));
        }
        if snapshot.phase != Phase::Connected {
            return Err(Problem::new(C::Lost));
        }
        Ok(state.installation.as_ref().map(|(peer, _)| *peer))
    }

    /// Exchange one complete installation message through the existing connection task.
    /// This native API does not grant permission or retry uncertain messages. Dropping
    /// its future after transmission starts disconnects; reconnect and reconcile via Upload.
    /// # Errors
    /// Reject missing authentication, invalid/currently busy requests, expired connections,
    /// uncertain delivery, malformed responses and timeouts.
    pub async fn exchange_installation(&self, epoch: u32, frame: Frame) -> Result<Frame, Problem> {
        let receive = {
            let mut state = self.inner.lock().map_err(|_| Problem::new(C::Closed))?;
            let snapshot = state.snapshot();
            if state.closed {
                return Err(Problem::new(C::Closed));
            }
            if snapshot.epoch != epoch {
                return Err(Problem::new(C::Stale));
            }
            if snapshot.phase != Phase::Connected {
                return Err(Problem::new(C::Lost));
            }
            let (peer, sender) = state
                .installation
                .as_ref()
                .ok_or_else(|| Problem::new(C::Installation))?;
            peer.request(&frame)?;
            if state.install_busy.upgrade().is_some() {
                return Err(Problem::new(C::Busy));
            }
            let lease = Arc::new(());
            let busy = Arc::downgrade(&lease);
            let (reply, receive) = oneshot::channel();
            sender
                .try_send(Call {
                    frame,
                    reply,
                    lease,
                })
                .map_err(|_| Problem::new(C::Lost))?;
            state.install_busy = busy;
            receive
        };
        receive.await.map_err(|_| Problem::new(C::Lost))?
    }
}
