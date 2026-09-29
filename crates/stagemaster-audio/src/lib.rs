//! Bounded host media resources and audio output; independent of project/UI/device protocols.
mod resources;
mod transport;
mod waveform;
pub use resources::{MAX_FILE_BYTES, Resources, verify};
pub use transport::{Position, Transport};
pub use waveform::{BUCKET_MS, MAX_DURATION_MS, Waveform, analyze};
