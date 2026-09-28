use crate::{Candidate, InstallationPeer, Problem, ProblemCode};
use std::future::Future;

/// One adapter and at most one pending/established physical connection.
/// Cleanup MUST include pending operations after their futures were cancelled.
/// Carries bytes; installation state, storage and playback remain outside the adapter.
pub trait Transport: Send + 'static {
    fn start_scan(&mut self) -> impl Future<Output = Result<(), Problem>> + Send;
    fn discover(&mut self) -> impl Future<Output = Result<Candidate, Problem>> + Send;
    fn stop_scan(&mut self) -> impl Future<Output = Result<(), Problem>> + Send;
    fn connect(&mut self, id: &str) -> impl Future<Output = Result<(), Problem>> + Send;
    fn write(&mut self, bytes: &[u8; 20]) -> impl Future<Output = Result<(), Problem>> + Send;
    fn reply(&mut self) -> impl Future<Output = Result<Vec<u8>, Problem>> + Send;
    fn diagnostics(&mut self) -> impl Future<Output = Result<Vec<u8>, Problem>> + Send;
    /// None means the characteristic is absent on a legacy diagnostic device.
    /// Present-but-invalid data and transport failures MUST NOT return None.
    fn description(&mut self) -> impl Future<Output = Result<Option<Vec<u8>>, Problem>> + Send;
    /// Only a trusted authentication adapter may return verified installation facts.
    /// Public metadata/diagnostic success MUST NOT create a grant. Reset on disconnect.
    fn installation_peer(&self) -> Option<InstallationPeer> {
        None
    }
    /// Accept one ordered fragment; an error/timeout makes delivery uncertain.
    fn write_installation(
        &mut self,
        _bytes: &[u8],
    ) -> impl Future<Output = Result<(), Problem>> + Send {
        async { Err(Problem::new(ProblemCode::Installation)) }
    }
    /// Nonblocking, cancel-safe dequeue from a bounded, connection-local receiver.
    /// # Errors
    /// Queue overflow and revoked authentication MUST return errors, never drop bytes.
    fn try_installation_notification(&mut self) -> Result<Option<Vec<u8>>, Problem> {
        Ok(None)
    }
    fn disconnect(&mut self) -> impl Future<Output = Result<(), Problem>> + Send;
}
