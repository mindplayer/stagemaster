use crate::PackageInfo;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Querying,
    Transferring,
    Verifying,
    Committing,
    Cancelling,
    Reconnect,
    Failed,
    Installed,
    Cancelled,
    NotStarted,
}
impl Phase {
    #[must_use]
    pub const fn terminal(self) -> bool {
        matches!(self, Self::Installed | Self::Cancelled | Self::NotStarted)
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub generation: String,
    pub digest: String,
    pub bytes: usize,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub package: PackageInfo,
    pub device_id: String,
    pub device_name: String,
    pub connection_epoch: u32,
    pub phase: Phase,
    pub running: bool,
    pub cancel_requested: bool,
    /// Last authoritative application receipt, never ATT bytes sent.
    pub confirmed_bytes: usize,
    pub receipt: Option<Receipt>,
    pub problem: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub revision: u64,
    pub task: Option<Task>,
}
