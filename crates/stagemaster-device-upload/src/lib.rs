//! Application installation intents; no UI, file dialogs, physical transport or playback I/O.
#![forbid(unsafe_code)]
mod connection;
mod model;
mod package;
mod runner;
mod service;
mod state;

pub use connection::{Connection, Target};
pub use model::{Phase, Receipt, Snapshot, Task};
pub use package::{PackageInfo, Prepared};
pub use service::Service;
