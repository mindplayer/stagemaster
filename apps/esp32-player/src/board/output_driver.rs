//! Preserve the concrete queue failure before Port intentionally maps it to Driver.
use core::cell::Cell;
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use stagemaster_output_port::{
    Driver as PortDriver, Event, Ticket,
    dmx::queued::{Fault, QueuedDriver},
};

static FAULT: Mutex<CriticalSectionRawMutex, Cell<Option<Fault>>> = Mutex::new(Cell::new(None));
pub struct Driver(QueuedDriver<'static, CriticalSectionRawMutex>);
impl Driver {
    pub(super) fn new(inner: QueuedDriver<'static, CriticalSectionRawMutex>) -> Self {
        Self(inner)
    }
}
fn record<T>(result: Result<T, Fault>) -> Result<T, Fault> {
    if let Err(error) = result {
        FAULT.lock(|slot| {
            if slot.get().is_none() {
                slot.set(Some(error));
            }
        });
    }
    result
}
impl PortDriver for Driver {
    type Error = Fault;
    fn submit(&mut self, ticket: Ticket, slots: &[u8; 512], until: u64) -> Result<(), Fault> {
        record(self.0.submit(ticket, slots, until))
    }
    fn quiesce(&mut self, ticket: Ticket) -> Result<(), Fault> {
        record(self.0.quiesce(ticket))
    }
    fn poll(&mut self) -> Result<Option<Event>, Fault> {
        record(self.0.poll())
    }
}
pub(super) fn report() {
    let fault = FAULT.lock(Cell::get);
    crate::diagnostics::report_line(format_args!("UART QUEUE fault={fault:?}; RS485=disabled"));
}
