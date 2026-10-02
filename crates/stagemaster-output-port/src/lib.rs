//! Bounded, exclusive output authority, independent of playback, UI and transport.
//! No hardware implementation is provided. A driver must uphold its completion contract.
#![no_std]
#![forbid(unsafe_code)]
mod driver;
mod lifecycle;
mod port;
mod scheduling;
mod types;

pub use driver::{Driver, Event};
pub use port::Port;
pub use types::{Code, Config, FrameId, Permit, Phase, Sample, Source, SourceKind, State, Ticket};

/// Maximum driver events consumed by one poll. The host must poll regularly.
pub const EVENT_BUDGET: usize = 4;

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod limits_tests;
