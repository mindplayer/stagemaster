use super::RuntimeClient;
use crate::{Error, RecordIo, wire};
use stagemaster_runtime_protocol::{Access, Operation, Request};
use tokio::time::Instant;

fn allowed(access: Access, operation: Operation) -> bool {
    match operation {
        Operation::Status | Operation::Catalog { .. } | Operation::Step { .. } => access.observe,
        Operation::FinishMaintenance => access.control && access.installation,
        Operation::Apply(action) => {
            // Maintenance actions are the only Apply variants requiring installation scope.
            access.control
                && (!matches!(
                    action,
                    stagemaster_runtime::Action::BeginMaintenance
                        | stagemaster_runtime::Action::CancelMaintenance
                ) || access.installation)
        }
        _ => access.control,
    }
}
impl<R: RecordIo> RuntimeClient<R> {
    /// Submit one new intent with an explicit observed revision. Receipt success is separate.
    /// # Errors
    /// Refuse busy/unsupported operations locally; failed or cancelled sends invalidate the channel
    /// and retain the uncertain request. No automatic re-acquisition or command replay.
    pub async fn send(
        &mut self,
        operation: Operation,
        expected_revision: u64,
    ) -> Result<Request, Error> {
        self.check()?;
        if self.pending.is_some() {
            return Err(Error::Protocol("设备仍有待确认操作，请先读取回复".into()));
        }
        let peer = self.peer().ok_or(Error::Closed)?;
        if !allowed(peer.access, operation) {
            return Err(Error::Denied);
        }
        let id = self.serial.checked_add(1).ok_or(Error::Bounds)?;
        let request = Request {
            session: peer.peer.session,
            id,
            expected_revision,
            operation,
        };
        let frame = request.encode().map_err(wire)?;
        self.serial = id;
        self.pending = Some((request, Instant::now()));
        let result = self.channel.write(frame.bytes()).await;
        self.checked(result)?;
        self.check()?;
        Ok(request)
    }
    /// Explicit same-request retry while the exact channel is still valid. The original
    /// deadline is retained; late identical replies are historical duplicates.
    /// # Errors
    /// Refuse absent/stale work and preserve ambiguity if sending fails or is cancelled.
    pub async fn retry_pending(&mut self) -> Result<Request, Error> {
        self.check()?;
        let request = self
            .pending()
            .ok_or_else(|| Error::Protocol("没有待确认的设备操作".into()))?;
        let frame = request.encode().map_err(wire)?;
        let result = self.channel.write(frame.bytes()).await;
        self.checked(result)?;
        self.check()?;
        Ok(request)
    }
}
