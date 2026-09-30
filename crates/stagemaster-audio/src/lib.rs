//! Bounded host media resources and audio output; independent of project/UI/device protocols.
mod resource_health;
mod resources;
pub use resource_health::{ResourceFileHealth, ResourceHealth, ResourceSource};
mod transport;
mod waveform;
pub use resources::{MAX_FILE_BYTES, Resources, verify};
pub use transport::{Position, Transport};
pub use waveform::{BUCKET_MS, MAX_DURATION_MS, Waveform, analyze};
