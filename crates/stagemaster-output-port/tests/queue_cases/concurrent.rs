#[path = "thread_support.rs"]
mod support;
use crate::tickets;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use stagemaster_output_port::{
    Driver, Event,
    dmx::{Clock, Transmitter, queued::Queue},
};
use support::*;

#[test]
fn separate_thread_wakes_for_frames_idle_expiry_and_stop_while_completion_is_unread() {
    let mut queue = Queue::<CriticalSectionRawMutex>::new();
    let (driver, receiver) = queue.split().unwrap();
    let wire = Wire::default();
    let clock = Time::default();
    let tx = Transmitter::new(wire.clone(), clock.clone(), 40_000).unwrap();
    let t = tickets();
    std::thread::scope(|scope| {
        let mut driver = driver;
        let task = scope.spawn(|| run(receiver.run(tx)));
        for value in 1..=16_u8 {
            driver
                .submit(t[0], &[value; 512], clock.now_us() / 1000 + 100)
                .unwrap();
            wait(|| {
                clock.advance(clock.now_us() + 16);
                driver.poll().unwrap() == Some(Event::Sent(t[0]))
            });
            assert_eq!(wire.0.lock().unwrap().bytes.len(), usize::from(value) * 513);
        }
        let idle_bytes = wire.0.lock().unwrap().bytes.len();
        clock.advance(clock.now_us() + 100_000);
        // Driver is not polled. The independently woken sender must still gate off.
        wait(|| !wire.0.lock().unwrap().enabled);
        assert_eq!(wire.0.lock().unwrap().bytes.len(), idle_bytes);

        driver
            .submit(t[0], &[77; 512], clock.now_us() / 1000 + 100)
            .unwrap();
        wait(|| {
            clock.advance(clock.now_us() + 16);
            wire.0.lock().unwrap().bytes.len() == idle_bytes + 513
        });
        // Do not consume Sent; a stop must not wait for that slot to become free.
        wire.0.lock().unwrap().hold = true;
        driver.quiesce(t[1]).unwrap();
        wait(|| wire.0.lock().unwrap().waiting.is_some());
        assert!(!wire.0.lock().unwrap().enabled);
        assert_eq!(driver.poll(), Ok(None));
        let wake = {
            let mut trace = wire.0.lock().unwrap();
            trace.hold = false;
            trace.waiting.take().unwrap()
        };
        wake.wake();
        wait(|| driver.poll().unwrap() == Some(Event::Quiet(t[1])));
        let trace = wire.0.lock().unwrap();
        for (index, frame) in trace.bytes.chunks_exact(513).take(16).enumerate() {
            assert_eq!(frame[0], 0);
            assert_eq!(&frame[1..], &[u8::try_from(index + 1).unwrap(); 512]);
        }
        drop(trace);
        drop(driver);
        task.join().unwrap();
    });
    assert!(!wire.0.lock().unwrap().enabled);
}
