#![cfg(feature = "queued-dmx")]
#[path = "queue_cases/admission.rs"]
mod admission;
#[path = "queue_cases/concurrent.rs"]
mod concurrent;
#[path = "queue_cases/lifecycle.rs"]
mod lifecycle;
#[path = "queue_cases/port_fault.rs"]
mod port_fault;
#[path = "queue_cases/races.rs"]
mod races;
#[path = "queue_cases/runtime.rs"]
mod runtime;
#[path = "queue_cases/stopping.rs"]
mod stopping;
#[path = "dmx_support/mod.rs"]
mod support;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use stagemaster_output_port::{dmx::queued::Queue, *};
type LocalQueue = Queue<NoopRawMutex>;

fn tickets() -> [Ticket; 3] {
    let (mut port, driver) = support::setup();
    core::array::from_fn(|_| {
        let t = port.select(support::source(), true, 0).unwrap();
        driver.0.borrow_mut().event = Some(Event::Quiet(t));
        port.poll(0).unwrap();
        t
    })
}
