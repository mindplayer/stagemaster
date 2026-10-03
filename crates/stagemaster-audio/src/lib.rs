//! Bounded host media resources and audio output; independent of project/UI/device protocols.
mod consumption;
pub use consumption::Consumption;
mod resource_health;
mod resources;
pub use resource_health::{ResourceFileHealth, ResourceHealth, ResourceSource};
mod looping;
mod performance;
pub use looping::{LoopRange, LoopRequest, MAX_LOOP_MS, PreparedLoop};
pub use performance::{
    LoopExitIntent, MAX_PERFORMANCE_CACHE_BYTES, PerformanceAudio, PerformanceControl,
    PerformanceSnapshot, PerformanceSource, PlaybackRequest, RenderObservation,
};
mod transport;
mod waveform;
pub use resources::{MAX_FILE_BYTES, Resources, verify};
pub use transport::{
    AudioLoadRequest, AudioLoadTicket, AudioSeekRequest, PendingExit, PerformanceObservation,
    PerformancePosition, Position, PreparedAudioLoad, PreparedAudioSeek, Transport,
};
pub use waveform::{BUCKET_MS, MAX_DURATION_MS, Waveform, analyze};
