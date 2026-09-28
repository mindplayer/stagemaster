use crate::{Candidate, Problem};
use std::future::Future;

/// One adapter and at most one pending/established physical connection.
/// Cleanup MUST include pending operations after their futures were cancelled.
/// No implementation may install programs or issue playback commands here.
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
    fn disconnect(&mut self) -> impl Future<Output = Result<(), Problem>> + Send;
}
