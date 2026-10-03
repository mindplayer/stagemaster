use crate::support::*;
use core::{pin::pin, task::Poll};
use stagemaster_output_port::dmx::Error;

#[test]
fn cancellation_in_break_mab_write_and_drain_disables_and_next_send_drains_first() {
    for stage in ["break", "mark", "write", "drain"] {
        let (mut tx, wire, time) = transmitter();
        let ticket = ticket();
        {
            let mut s = wire.0.borrow_mut();
            s.hold = match stage {
                "break" | "write" => Some(stage),
                _ => None,
            };
        }
        {
            let mut future = pin!(tx.send(ticket, &[7; 512], 100));
            assert!(poll(future.as_mut()).is_pending());
            if stage == "write" || stage == "drain" {
                wire.0.borrow_mut().hold = Some(stage);
                time.0.set(16);
                assert!(poll(future.as_mut()).is_pending());
            }
            assert!(wire.0.borrow().enabled);
        }
        assert!(!wire.0.borrow().enabled, "{stage}");
        {
            let mut s = wire.0.borrow_mut();
            s.hold = Some("drain");
            s.calls.clear();
        }
        let mut next = pin!(tx.send(ticket, &[8; 512], 100));
        assert!(poll(next.as_mut()).is_pending());
        assert_eq!(wire.0.borrow().calls, ["drain"]);
        assert!(!wire.0.borrow().enabled);
    }
}

#[test]
fn each_io_error_and_invalid_partial_write_gates_off() {
    for stage in ["drain", "break", "write", "zero", "oversized"] {
        let (mut tx, wire, time) = transmitter();
        {
            let mut s = wire.0.borrow_mut();
            match stage {
                "zero" => s.count = Some(0),
                "oversized" => s.count = Some(514),
                _ => s.fail = Some(stage),
            }
        }
        let result = finish(pin!(tx.send(ticket(), &[8; 512], 100)), &time);
        assert!(result.is_err(), "{stage}");
        assert!(!wire.0.borrow().enabled);
    }
}

#[test]
fn timeout_and_clock_regression_do_not_report_completion() {
    for backwards in [false, true] {
        let (mut tx, wire, time) = transmitter();
        time.0.set(1000);
        wire.0.borrow_mut().hold = Some("write");
        let mut send = pin!(tx.send(ticket(), &[9; 512], 100));
        assert!(poll(send.as_mut()).is_pending());
        time.0.set(1016);
        assert!(poll(send.as_mut()).is_pending());
        time.0.set(if backwards { 999 } else { 41000 });
        assert_eq!(
            poll(send.as_mut()),
            Poll::Ready(Err(if backwards {
                Error::Clock
            } else {
                Error::Deadline
            }))
        );
        assert!(!wire.0.borrow().enabled);
    }
}

#[test]
fn quiesce_cancellation_and_timeout_never_acknowledge_uncleared_fifo() {
    let (mut tx, wire, time) = transmitter();
    wire.0.borrow_mut().hold = Some("drain");
    {
        let mut quiet = pin!(tx.quiesce(ticket()));
        assert!(poll(quiet.as_mut()).is_pending());
        assert!(!wire.0.borrow().enabled);
    }
    let mut quiet = pin!(tx.quiesce(ticket()));
    assert!(poll(quiet.as_mut()).is_pending());
    time.0.set(40000);
    assert_eq!(poll(quiet.as_mut()), Poll::Ready(Err(Error::Deadline)));
}
