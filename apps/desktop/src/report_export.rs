//! One native report dialog at a time, shared across report formats.
use std::sync::Mutex;
#[derive(Default)]
pub(crate) struct Service(pub(crate) Mutex<()>);
