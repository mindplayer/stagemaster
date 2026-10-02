use stagemaster_output_port::{Config, Driver, Event, Port, Sample, Source, SourceKind, Ticket};
use stagemaster_project::LiveScenePlayer;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

#[derive(Default)]
struct State {
    events: VecDeque<Event>,
    frames: Vec<[u8; 512]>,
}
struct Software(Rc<RefCell<State>>);
impl Driver for Software {
    type Error = ();
    fn submit(&mut self, ticket: Ticket, slots: &[u8; 512], _: u64) -> Result<(), ()> {
        self.0.borrow_mut().frames.push(*slots);
        self.0.borrow_mut().events.push_back(Event::Sent(ticket));
        Ok(())
    }
    fn quiesce(&mut self, ticket: Ticket) -> Result<(), ()> {
        self.0.borrow_mut().events.push_back(Event::Quiet(ticket));
        Ok(())
    }
    fn poll(&mut self) -> Result<Option<Event>, ()> {
        Ok(self.0.borrow_mut().events.pop_front())
    }
}
pub fn verify(scene: &LiveScenePlayer, mixer: &stagemaster_engine::live::LiveMixer) {
    let mut output = scene.prepare_output().unwrap();
    let mut slots = [0; 512];
    let universe = output.render(mixer, &mut slots).unwrap();
    assert!(output.winners().iter().any(Option::is_some));
    assert_eq!(output.values()[1], 0x1fff);
    assert_eq!(slots[1], 0x1f);
    assert_eq!(slots[5], 0xff, "16-bit coarse/fine stay together");
    assert_eq!(
        slots[3], 20,
        "wheel is an exact slot, not a blended channel"
    );
    let state = Rc::new(RefCell::new(State::default()));
    let mut port = Port::new(
        Config {
            boot: [8; 16],
            port: 1,
            universe: 1,
            max_age_ms: 100,
            ack_timeout_ms: 50,
        },
        Software(state.clone()),
        500,
    )
    .unwrap();
    port.select(
        Source {
            id: [9; 16],
            kind: SourceKind::Composite,
        },
        false,
        500,
    )
    .unwrap();
    port.poll(500).unwrap();
    let permit = port.state().permit.unwrap();
    port.submit(
        permit,
        Sample {
            serial: 1,
            sampled_ms: 500,
            universe,
            slots: &slots,
        },
        500,
    )
    .unwrap();
    port.poll(500).unwrap();
    assert!(port.state().completed.is_none());
    port.poll(501).unwrap();
    assert!(port.state().completed.is_some());
    assert_eq!(state.borrow().frames, [slots]);
    port.stop(permit, 502).unwrap();
    port.poll(502).unwrap();
    assert!(port.state().quiet);
}
