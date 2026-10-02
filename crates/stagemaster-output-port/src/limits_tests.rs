use crate::*;
use std::{collections::VecDeque, vec::Vec};

#[derive(Default)]
struct DriverProbe {
    events: VecDeque<Event>,
    stops: Vec<Ticket>,
    frames: usize,
}
impl Driver for DriverProbe {
    type Error = ();
    fn submit(&mut self, _: Ticket, _: &[u8; 512], _: u64) -> Result<(), ()> {
        self.frames += 1;
        Ok(())
    }
    fn quiesce(&mut self, ticket: Ticket) -> Result<(), ()> {
        self.stops.push(ticket);
        Ok(())
    }
    fn poll(&mut self) -> Result<Option<Event>, ()> {
        Ok(self.events.pop_front())
    }
}
fn port() -> Port<DriverProbe> {
    Port::new(
        Config {
            boot: [1; 16],
            port: 1,
            universe: 1,
            max_age_ms: 100,
            ack_timeout_ms: 50,
        },
        DriverProbe::default(),
        0,
    )
    .unwrap()
}
fn activate(port: &mut Port<DriverProbe>) -> Permit {
    let ticket = port
        .select(
            Source {
                id: [2; 16],
                kind: SourceKind::Local,
            },
            false,
            0,
        )
        .unwrap();
    port.driver.events.push_back(Event::Quiet(ticket));
    port.poll(0).unwrap();
    port.state().permit.unwrap()
}
#[test]
fn final_ticket_is_reserved_for_shutdown_and_never_reused() {
    let mut port = port();
    let permit = activate(&mut port);
    port.counter = u64::MAX - 1;
    port.submit(
        permit,
        Sample {
            serial: 1,
            sampled_ms: 0,
            universe: 1,
            slots: &[1; 512],
        },
        0,
    )
    .unwrap();
    assert_eq!(port.poll(0), Err(Code::Exhausted));
    assert_eq!(port.driver.frames, 0);
    let stop = *port.driver.stops.last().unwrap();
    assert_eq!(stop.counter, u64::MAX);
    port.driver.events.push_back(Event::Quiet(stop));
    assert_eq!(port.poll(1), Err(Code::Exhausted));
    assert_eq!(port.state().phase, Phase::Faulted);
    assert!(port.state().quiet);
    assert!(port.state().permit.is_none());
    assert_eq!(port.shutdown(2), Err(Code::Exhausted));
    assert_eq!(port.driver.stops.len(), 2);
}
#[test]
fn unrelated_or_future_receipt_cannot_grant_authority() {
    for bad in [
        Ticket {
            boot: [8; 16],
            port: 1,
            counter: 1,
        },
        Ticket {
            boot: [1; 16],
            port: 2,
            counter: 1,
        },
        Ticket {
            boot: [1; 16],
            port: 1,
            counter: 9,
        },
    ] {
        let mut port = port();
        activate(&mut port);
        port.driver.events.push_back(Event::Quiet(bad));
        assert_eq!(port.poll(0), Err(Code::DriverReport));
        assert!(port.state().permit.is_none());
        assert!(!port.state().quiet);
    }
}
#[test]
fn poll_budget_prevents_stale_receipts_from_blocking_the_scheduler() {
    let mut port = port();
    let permit = activate(&mut port);
    for _ in 0..20 {
        port.driver.events.push_back(Event::Quiet(permit.ticket));
    }
    port.submit(
        permit,
        Sample {
            serial: 1,
            sampled_ms: 0,
            universe: 1,
            slots: &[8; 512],
        },
        0,
    )
    .unwrap();
    port.poll(0).unwrap();
    assert_eq!(port.driver.events.len(), 20 - EVENT_BUDGET);
    assert_eq!(port.driver.frames, 1);
    assert!(port.state().completed.is_none());
}
#[test]
fn counter_exhaustion_during_selection_cannot_activate_a_new_source() {
    let mut port = port();
    port.counter = u64::MAX - 1;
    assert_eq!(
        port.select(
            Source {
                id: [2; 16],
                kind: SourceKind::External
            },
            false,
            0
        ),
        Err(Code::Exhausted)
    );
    let stop = port.state().stopping.unwrap();
    port.driver.events.push_back(Event::Quiet(stop));
    assert_eq!(port.poll(0), Err(Code::Exhausted));
    assert!(port.state().permit.is_none());
    assert!(port.state().quiet);
}

#[test]
fn clock_addition_cannot_wrap_and_leave_a_source_active() {
    let configuration = Config {
        boot: [1; 16],
        port: 1,
        universe: 1,
        max_age_ms: 100,
        ack_timeout_ms: 50,
    };
    let mut port = Port::new(configuration, DriverProbe::default(), u64::MAX - 150).unwrap();
    let ticket = port
        .select(
            Source {
                id: [2; 16],
                kind: SourceKind::Local,
            },
            false,
            u64::MAX - 150,
        )
        .unwrap();
    port.driver.events.push_back(Event::Quiet(ticket));
    port.poll(u64::MAX - 150).unwrap();
    let permit = port.state().permit.unwrap();
    assert_eq!(
        port.submit(
            permit,
            Sample {
                serial: 1,
                sampled_ms: u64::MAX - 99,
                universe: 1,
                slots: &[1; 512]
            },
            u64::MAX - 99
        ),
        Err(Code::Exhausted)
    );
    assert!(port.state().permit.is_none());
    assert!(!port.state().quiet);
    assert_eq!(port.driver.frames, 0);
    assert_eq!(port.driver.stops.len(), 2);
    assert!(matches!(
        Port::new(configuration, DriverProbe::default(), u64::MAX - 50),
        Err(Code::Exhausted)
    ));
}

#[test]
fn invalid_configuration_or_source_is_rejected_before_output() {
    let config = Config {
        boot: [1; 16],
        port: 1,
        universe: 1,
        max_age_ms: 100,
        ack_timeout_ms: 50,
    };
    for invalid in [
        Config {
            boot: [0; 16],
            ..config
        },
        Config { port: 0, ..config },
        Config {
            universe: 0,
            ..config
        },
        Config {
            max_age_ms: 0,
            ..config
        },
        Config {
            ack_timeout_ms: 60_001,
            ..config
        },
    ] {
        assert!(matches!(
            Port::new(invalid, DriverProbe::default(), 0),
            Err(Code::Identity)
        ));
    }
    let mut port = port();
    assert_eq!(
        port.select(
            Source {
                id: [0; 16],
                kind: SourceKind::External
            },
            false,
            0
        ),
        Err(Code::Identity)
    );
    assert!(port.driver.stops.is_empty());
    assert_eq!(port.state().phase, Phase::Unconfirmed);
}
