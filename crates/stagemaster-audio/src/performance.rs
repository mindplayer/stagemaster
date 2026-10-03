//! Prepared repeat PCM plus bounded forward decoding, driven by one sample-frame cursor.
mod control;
mod decode;
mod intent;
mod position;
mod prepare;
mod source;
mod stream;

pub use control::{LoopExitIntent, PerformanceControl, PerformanceSnapshot, RenderObservation};
pub use intent::PlaybackRequest;
pub use prepare::{MAX_PERFORMANCE_CACHE_BYTES, PerformanceAudio};
pub use source::PerformanceSource;

#[cfg(test)]
pub(crate) mod tests;
