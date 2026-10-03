//! JSON presentation adapter. No new transport, scheduler, or authority owner.
mod access;
mod command;
mod view;
use crate::{Problem, ProblemCode, RuntimeIntent, Service, Transport};
pub use access::ExpectedAccess;
pub use command::{Action, Key, Request};
use stagemaster_runtime_protocol::Operation;
pub use view::{Reply, View};

impl<B: Transport> Service<B> {
    /// Read or operate through the existing admitted runtime connection.
    /// # Errors
    /// Refuse malformed intent, stale epochs, overlapping requests and transport failures.
    pub async fn runtime_request(
        &self,
        request: Request,
        access: &ExpectedAccess,
    ) -> Result<View, Problem> {
        let epoch = request.epoch();
        let operation = match request {
            Request::Connect { epoch, id } => {
                let connected = access.connect(self, epoch, id)?;
                return Ok(View::new(
                    connected.epoch,
                    &self.runtime_snapshot(connected.epoch)?,
                    None,
                ));
            }
            Request::Snapshot { .. } => None,
            Request::Refresh { .. } => Some((0, Operation::Status)),
            Request::Catalog {
                revision, index, ..
            } => Some((command::number(&revision)?, Operation::Catalog { index })),
            Request::Step {
                revision, index, ..
            } => Some((command::number(&revision)?, Operation::Step { index })),
            Request::Apply {
                revision, action, ..
            } => {
                let duration = if matches!(action, Action::Acquire { .. } | Action::Renew { .. }) {
                    self.runtime_control_duration(epoch)?
                } else {
                    0
                };
                Some((command::number(&revision)?, action.operation(duration)?))
            }
        };
        let reply = if let Some((expected_revision, operation)) = operation {
            Some(
                self.exchange_runtime(
                    epoch,
                    RuntimeIntent {
                        operation,
                        expected_revision,
                    },
                )
                .await?,
            )
        } else {
            None
        };
        let snapshot = self.runtime_snapshot(epoch)?;
        // Snapshot rechecks the epoch after await. Old history is explicitly labelled.
        Ok(View::new(epoch, &snapshot, reply.as_ref()))
    }
}
fn invalid() -> Problem {
    Problem::new(ProblemCode::Protocol).detail("设备操作参数无效，请刷新状态后重试".into())
}
