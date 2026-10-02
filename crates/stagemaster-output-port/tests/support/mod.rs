pub mod project;
use stagemaster_output_port::*;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub struct Submitted {
    pub ticket: Ticket,
    pub slots: [u8; 512],
    pub until: u64,
}
#[derive(Default)]
pub struct Probe {
    pub stops: Vec<Ticket>,
    pub frames: Vec<Submitted>,
    pub sent: Vec<[u8; 512]>,
    pub events: VecDeque<Event>,
    pub fail_submit: bool,
    pub fail_stop: bool,
    pub fail_poll: bool,
    flight: Option<usize>,
    stopping: Option<Ticket>,
}
#[derive(Clone, Default)]
pub struct Control(pub Rc<RefCell<Probe>>);
pub struct SoftwareDriver(Control);
impl Driver for SoftwareDriver {
    type Error = ();
    fn submit(&mut self, ticket: Ticket, slots: &[u8; 512], until: u64) -> Result<(), ()> {
        let mut state = self.0.0.borrow_mut();
        assert!(
            state.stopping.is_none(),
            "cannot send before the driver is quiet"
        );
        assert!(
            state.flight.is_none(),
            "driver cannot accept a hidden backlog"
        );
        // Deliberately accept before returning failure: admission can be ambiguous.
        state.flight = Some(state.frames.len());
        state.frames.push(Submitted {
            ticket,
            slots: *slots,
            until,
        });
        if state.fail_submit { Err(()) } else { Ok(()) }
    }
    fn quiesce(&mut self, ticket: Ticket) -> Result<(), ()> {
        let mut state = self.0.0.borrow_mut();
        state.stops.push(ticket);
        state.stopping = Some(ticket);
        if state.fail_stop { Err(()) } else { Ok(()) }
    }
    fn poll(&mut self) -> Result<Option<Event>, ()> {
        let mut state = self.0.0.borrow_mut();
        if state.fail_poll {
            return Err(());
        }
        Ok(state.events.pop_front())
    }
}
impl Control {
    pub fn quiet(&self) {
        let mut state = self.0.borrow_mut();
        let ticket = state.stopping.take().expect("stop requested");
        state.flight = None;
        state.events.push_back(Event::Quiet(ticket));
    }
    pub fn finish(&self, now: u64) {
        let mut state = self.0.borrow_mut();
        let index = state.flight.take().expect("frame accepted");
        let Submitted {
            ticket,
            slots,
            until,
        } = state.frames[index];
        if now >= until {
            state.events.push_back(Event::Expired(ticket));
        } else {
            state.sent.push(slots);
            state.events.push_back(Event::Sent(ticket));
        }
    }
    pub fn inject(&self, event: Event) {
        self.0.borrow_mut().events.push_back(event);
    }
}
pub fn config() -> Config {
    Config {
        boot: [1; 16],
        port: 1,
        universe: 1,
        max_age_ms: 100,
        ack_timeout_ms: 50,
    }
}
pub fn source(kind: SourceKind) -> Source {
    Source { id: [2; 16], kind }
}
pub fn setup() -> (Port<SoftwareDriver>, Control) {
    let control = Control::default();
    (
        Port::new(config(), SoftwareDriver(control.clone()), 0).unwrap(),
        control,
    )
}
pub fn ready(port: &mut Port<SoftwareDriver>, control: &Control) -> Permit {
    port.select(source(SourceKind::Local), false, 0).unwrap();
    control.quiet();
    port.poll(0).unwrap();
    port.state().permit.unwrap()
}
pub fn sample(serial: u64, now: u64, slots: &[u8; 512]) -> Sample<'_> {
    Sample {
        serial,
        sampled_ms: now,
        universe: 1,
        slots,
    }
}
