//! Whole-value cross-core authority publication, separate from bounded work queues.
use core::cell::RefCell;
use embassy_sync::{
    blocking_mutex::{Mutex, raw::CriticalSectionRawMutex},
    channel::Channel,
};
use stagemaster_install_worker::runtime_queue::{Command, Completion, Live};

pub static REQUESTS: Channel<CriticalSectionRawMutex, Command, 1> = Channel::new();
pub static COMPLETIONS: Channel<CriticalSectionRawMutex, Completion, 1> = Channel::new();
static LIVE: Mutex<CriticalSectionRawMutex, RefCell<Option<Live>>> = Mutex::new(RefCell::new(None));

pub fn publish(value: Option<Live>) {
    LIVE.lock(|slot| *slot.borrow_mut() = value);
}
pub fn live() -> Option<Live> {
    LIVE.lock(|slot| *slot.borrow())
}
