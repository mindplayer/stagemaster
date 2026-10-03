#[path = "support/media.rs"]
mod media_support;
mod support;
use media_support::*;
use stagemaster_live::media::Status;
use stagemaster_output_port::{
    Code, Config, Driver, Event, Port, Sample, Source, SourceKind, Ticket,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
struct Probe {
    event: Option<Event>,
    frames: Vec<[u8; 512]>,
}
struct Software(Rc<RefCell<Probe>>);
impl Driver for Software {
    type Error = ();
    fn submit(&mut self, ticket: Ticket, slots: &[u8; 512], _: u64) -> Result<(), ()> {
        let mut probe = self.0.borrow_mut();
        assert!(probe.event.is_none());
        probe.frames.push(*slots);
        probe.event = Some(Event::Sent(ticket));
        Ok(())
    }
    fn quiesce(&mut self, ticket: Ticket) -> Result<(), ()> {
        self.0.borrow_mut().event = Some(Event::Quiet(ticket));
        Ok(())
    }
    fn poll(&mut self) -> Result<Option<Event>, ()> {
        Ok(self.0.borrow_mut().event.take())
    }
}

#[test]
fn media_seek_does_not_reset_output_serial_or_bypass_exclusive_port_authority() {
    let (doc, mut session, map) = setup();
    let probe = Rc::new(RefCell::new(Probe::default()));
    let mut port = Port::new(
        Config {
            boot: [7; 16],
            port: 1,
            universe: 1,
            max_age_ms: 100,
            ack_timeout_ms: 50,
        },
        Software(probe.clone()),
        1000,
    )
    .unwrap();
    port.select(
        Source {
            id: session.boot(),
            kind: SourceKind::Local,
        },
        false,
        1000,
    )
    .unwrap();
    port.poll(1000).unwrap();
    let permit = port.state().permit.unwrap();
    start(&doc, &mut session, &map, 80, 1000);
    let mut expected = Vec::new();
    let mut previous_sequence = 0;
    for time in [1001, 1025, 1050] {
        if time == 1025 {
            start(&doc, &mut session, &map, 0, 1024);
        }
        session.tick(time).unwrap();
        let frame = session.frame().unwrap();
        assert!(frame.sequence > previous_sequence);
        previous_sequence = frame.sequence;
        port.submit(
            permit,
            Sample {
                serial: frame.sequence,
                sampled_ms: frame.sampled_ms,
                universe: frame.universe,
                slots: &frame.slots,
            },
            time,
        )
        .unwrap();
        port.poll(time).unwrap();
        port.poll(time + 1).unwrap();
        expected.push(frame.slots);
    }
    assert_eq!(probe.borrow().frames, expected);
    assert_ne!(expected[0], expected[1]);
    assert_eq!(port.state().permit, Some(permit));
    port.select(
        Source {
            id: [8; 16],
            kind: SourceKind::External,
        },
        true,
        1075,
    )
    .unwrap();
    session.tick(1075).unwrap();
    let frame = session.frame().unwrap();
    assert_eq!(
        port.submit(
            permit,
            Sample {
                serial: frame.sequence,
                sampled_ms: frame.sampled_ms,
                universe: frame.universe,
                slots: &frame.slots
            },
            1075
        ),
        Err(Code::Permit)
    );
    assert_eq!(
        session.media_groups().next().unwrap().status,
        Status::Following
    );
    assert_eq!(probe.borrow().frames.len(), 3);
}
