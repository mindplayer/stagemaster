//! Bounded conversion between identified monotonic clocks. No I/O, allocation or clock adjustment.
#![no_std]
#![forbid(unsafe_code)]
mod mapping;
mod types;

pub use mapping::Mapping;
pub use types::{Clock, Error, Exchange, Instant, Limits, Window};
