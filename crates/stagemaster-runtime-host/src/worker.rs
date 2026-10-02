use crate::{
    Backend, COMMANDS_PER_CYCLE, Configuration, Error, Frame, Snapshot,
    client::{Acquisition, Command, Envelope, Reply},
    observation::Shared,
};
use stagemaster_runtime::Code;
use std::{
    sync::{atomic::Ordering, mpsc::Receiver},
    time::{Duration, Instant},
};

fn respond<T>(reply: Reply<T>, result: Result<T, Code>) {
    let _ = reply.try_send(result.map_err(Error::Runtime));
    drop(reply);
}
fn expired<T>(reply: Reply<T>) {
    let _ = reply.try_send(Err(Error::Deadline));
    drop(reply);
}

// Keep cadence anchored, skip missed slots, and never burst old frames on recovery.
pub(crate) fn next_cycle(
    due: Instant,
    finished: Instant,
    period: Duration,
) -> Result<(Instant, u64), Code> {
    let elapsed = finished.saturating_duration_since(due).as_nanos();
    let skipped = u64::try_from(elapsed / period.as_nanos()).map_err(|_| Code::Exhausted)?;
    let remainder = Duration::from_nanos(
        u64::try_from(elapsed % period.as_nanos()).map_err(|_| Code::Exhausted)?,
    );
    let next = finished
        .checked_add(period.checked_sub(remainder).ok_or(Code::Clock)?)
        .ok_or(Code::Exhausted)?;
    Ok((next, skipped))
}

pub(crate) fn dispatch<B: Backend>(runtime: &mut B, envelope: Envelope<B::Profile>, now: u64) {
    if Instant::now() >= envelope.deadline {
        match envelope.command {
            Command::Acquire { reply, .. } => expired(reply),
            Command::Submit { reply, .. } => expired(reply),
            Command::Renew { reply, .. } | Command::Release { reply, .. } => expired(reply),
        }
        return;
    }
    match envelope.command {
        Command::Acquire {
            grant,
            takeover,
            reply,
        } => {
            let result = runtime
                .acquire(grant, takeover, now)
                .map(|lease| Acquisition {
                    lease,
                    state: runtime.state(),
                });
            respond(reply, result);
        }
        Command::Submit { request, reply } => respond(reply, runtime.submit(request, now)),
        Command::Renew {
            lease,
            duration_ms,
            reply,
        } => respond(reply, runtime.renew(lease, duration_ms, now)),
        Command::Release { lease, reply } => respond(reply, runtime.release(lease, now)),
    }
}

pub(crate) fn run<B: Backend>(
    mut runtime: B,
    configuration: Configuration,
    receiver: Receiver<Envelope<B::Profile>>,
    shared: &Shared<B::Profile>,
) -> Result<(), Code> {
    let base = runtime.observed_ms();
    let clock = Instant::now();
    let now = || -> Result<u64, Code> {
        base.checked_add(u64::try_from(clock.elapsed().as_millis()).map_err(|_| Code::Exhausted)?)
            .ok_or(Code::Exhausted)
    };
    let mut due = clock;
    let mut cycles = 0_u64;
    let mut skipped = 0_u64;
    let mut max_lateness_ms = 0;
    let mut missed_periods = 0_u64;
    while !shared.stop.load(Ordering::Acquire) {
        let current = Instant::now();
        if current < due {
            std::thread::park_timeout(due - current);
            continue;
        }
        max_lateness_ms = max_lateness_ms
            .max(u64::try_from(current.duration_since(due).as_millis()).unwrap_or(u64::MAX));
        runtime.tick(now()?)?;
        for _ in 0..COMMANDS_PER_CYCLE {
            if shared.stop.load(Ordering::Acquire) {
                break;
            }
            let Ok(envelope) = receiver.try_recv() else {
                break;
            };
            dispatch(&mut runtime, envelope, now()?);
        }
        runtime.tick(now()?)?;
        let mut slots = [0; 512];
        let frame = runtime
            .render(&mut slots)?
            .map(|info| Frame { info, slots });
        let finished = Instant::now();
        let overrun = finished
            .duration_since(due)
            .saturating_sub(configuration.period);
        max_lateness_ms =
            max_lateness_ms.max(u64::try_from(overrun.as_millis()).unwrap_or(u64::MAX));
        let (next, missed) = next_cycle(due, finished, configuration.period)?;
        due = next;
        missed_periods = missed_periods.saturating_add(missed);
        cycles = cycles.saturating_add(1);
        if let Ok(mut output) = shared.snapshot.try_lock() {
            *output = Snapshot {
                state: runtime.state(),
                frame,
                cycles,
                skipped_publications: skipped,
                max_lateness_ms,
                missed_periods,
            };
        } else {
            skipped = skipped.saturating_add(1);
        }
    }
    drop(receiver);
    Ok(())
}
