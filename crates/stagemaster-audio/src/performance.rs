//! Prepared repeat PCM plus bounded forward decoding, driven by one sample-frame cursor.
mod control;
mod decode;
mod position;
mod prepare;
mod source;
mod stream;

pub use control::{LoopExitIntent, PerformanceControl, PerformanceSnapshot};
pub use prepare::{MAX_PERFORMANCE_CACHE_BYTES, PerformanceAudio};
pub use source::PerformanceSource;

#[cfg(test)]
mod tests;
