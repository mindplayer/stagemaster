use super::{CONNECT_TIMEOUT, Inner, update};
use crate::{Candidate, MAX_CANDIDATES, Phase, Problem, ProblemCode as C, Snapshot, Transport};
use std::{sync::Mutex, time::Duration};
use tokio::time::{Instant, timeout};
const SCAN_DURATION: Duration = Duration::from_secs(8);

pub(super) async fn run<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
) -> Result<(), Problem> {
    timeout(CONNECT_TIMEOUT, backend.start_scan())
        .await
        .map_err(|_| Problem::new(C::SetupTimeout))??;
    update(inner, epoch, |state| {
        state.snapshot.phase = Phase::Scanning;
        state.snapshot.scan_performed = true;
    });
    let deadline = Instant::now() + SCAN_DURATION;
    loop {
        let candidate = tokio::select! {
            biased;
            () = tokio::time::sleep_until(deadline) => return Ok(()),
            value = backend.discover() => value?,
        };
        update(inner, epoch, |state| {
            add_candidate(&mut state.snapshot, candidate);
        });
    }
}

fn add_candidate(snapshot: &mut Snapshot, candidate: Candidate) {
    if let Some(existing) = snapshot
        .candidates
        .iter_mut()
        .find(|c| c.id == candidate.id)
    {
        *existing = candidate;
    } else if snapshot.candidates.len() < MAX_CANDIDATES {
        snapshot.candidates.push(candidate);
    } else {
        snapshot.truncated = true;
    }
}
