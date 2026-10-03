#![allow(dead_code)]
pub mod project;
use core::{
    future::{Future, poll_fn},
    task::{Context, Poll, Waker},
};
use stagemaster_output_port::{
    dmx::{Clock, Line, Transmitter},
    *,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Default)]
pub struct Time(pub Rc<Cell<u64>>);
impl Clock for Time {
    fn now_us(&self) -> u64 {
        self.0.get()
    }
    async fn wait_until_us(&self, until: u64) {
        poll_fn(|_| {
            if self.0.get() >= until {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}
#[derive(Default)]
pub struct Trace {
    pub enabled: bool,
    pub calls: Vec<String>,
    pub bytes: Vec<u8>,
    pub hold: Option<&'static str>,
    pub fail: Option<&'static str>,
    pub count: Option<usize>,
    pub chunk: usize,
    pub drain_calls: usize,
}
#[derive(Clone)]
pub struct Wire(pub Rc<RefCell<Trace>>);
impl Wire {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Trace {
            chunk: 128,
            ..Trace::default()
        })))
    }
    async fn complete(&self, kind: &'static str) -> Result<(), ()> {
        poll_fn(|_| {
            let trace = self.0.borrow();
            if trace.fail == Some(kind) {
                return Poll::Ready(Err(()));
            }
            if trace.hold == Some(kind) {
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        })
        .await
    }
}
impl Line for Wire {
    type Error = ();
    fn disable(&mut self) {
        let mut s = self.0.borrow_mut();
        s.enabled = false;
        s.calls.push("disable".into());
    }
    fn enable(&mut self) {
        let mut s = self.0.borrow_mut();
        s.enabled = true;
        s.calls.push("enable".into());
    }
    async fn break_signal(&mut self, bits: u32) -> Result<(), ()> {
        assert_eq!(bits, 30);
        self.0.borrow_mut().calls.push("break".into());
        self.complete("break").await
    }
    async fn write(&mut self, bytes: &[u8]) -> Result<usize, ()> {
        self.complete("write").await?;
        let mut s = self.0.borrow_mut();
        let count = s.count.unwrap_or(bytes.len().min(s.chunk));
        if count <= bytes.len() {
            s.bytes.extend_from_slice(&bytes[..count]);
        }
        s.calls.push(format!("write:{count}"));
        Ok(count)
    }
    async fn drain(&mut self) -> Result<(), ()> {
        {
            let mut s = self.0.borrow_mut();
            s.drain_calls += 1;
            s.calls.push("drain".into());
        }
        self.complete("drain").await
    }
}
pub type Tx = Transmitter<Wire, Time>;
pub fn transmitter() -> (Tx, Wire, Time) {
    let wire = Wire::new();
    let time = Time::default();
    (
        Transmitter::new(wire.clone(), time.clone(), 40_000).unwrap(),
        wire,
        time,
    )
}
pub fn poll<F: Future>(f: core::pin::Pin<&mut F>) -> Poll<F::Output> {
    f.poll(&mut Context::from_waker(Waker::noop()))
}
#[derive(Default)]
pub struct Queue {
    pub frame: Option<(Ticket, [u8; 512], u64)>,
    pub stop: Option<Ticket>,
    pub event: Option<Event>,
}
#[derive(Clone, Default)]
pub struct DriverSide(pub Rc<RefCell<Queue>>);
impl Driver for DriverSide {
    type Error = ();
    fn submit(&mut self, t: Ticket, s: &[u8; 512], u: u64) -> Result<(), ()> {
        let mut q = self.0.borrow_mut();
        assert!(q.frame.is_none());
        q.frame = Some((t, *s, u));
        Ok(())
    }
    fn quiesce(&mut self, t: Ticket) -> Result<(), ()> {
        self.0.borrow_mut().stop = Some(t);
        Ok(())
    }
    fn poll(&mut self) -> Result<Option<Event>, ()> {
        Ok(self.0.borrow_mut().event.take())
    }
}
pub fn setup() -> (Port<DriverSide>, DriverSide) {
    let driver = DriverSide::default();
    let port = Port::new(
        Config {
            boot: [1; 16],
            port: 1,
            universe: 1,
            max_age_ms: 100,
            ack_timeout_ms: 50,
        },
        driver.clone(),
        0,
    )
    .unwrap();
    (port, driver)
}
pub fn source() -> Source {
    Source {
        id: [2; 16],
        kind: SourceKind::Local,
    }
}
pub fn ticket() -> Ticket {
    let (mut port, driver) = setup();
    port.shutdown(0).unwrap();
    driver.0.borrow_mut().stop.take().unwrap()
}
pub fn finish<F: Future>(mut f: core::pin::Pin<&mut F>, time: &Time) -> F::Output {
    for _ in 0..1000 {
        if let Poll::Ready(value) = poll(f.as_mut()) {
            return value;
        }
        time.0.set(time.0.get() + 100);
    }
    panic!("operation never completed")
}
