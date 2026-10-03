//! Bounded host-side ingress. Delivery never installs, grants permission or starts playback.
#![forbid(unsafe_code)]
mod error;
mod file;
#[cfg(feature = "http")]
mod http;
mod incoming;
mod package;
pub use error::Error;
pub use file::from_file;
#[cfg(feature = "http")]
pub use http::Http;
pub use incoming::Incoming;
pub use package::Package;
