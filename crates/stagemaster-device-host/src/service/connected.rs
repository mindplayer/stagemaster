use super::{
    CONNECT_TIMEOUT, Inner,
    diagnostics::exchange,
    installation::{Call, Pending},
    state::{FRESH_MS, Stamp},
    update,
};
use crate::{InstallationPeer, Phase, Problem, ProblemCode as C, Transport};
use stagemaster_device_link::client::Client;
use std::{sync::Mutex, time::Duration};
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};

pub(super) async fn run<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
    id: &str,
) -> Result<(), Problem> {
    timeout(CONNECT_TIMEOUT, backend.connect(id))
        .await
        .map_err(|_| Problem::new(C::Timeout))??;
    let mut client = Client::default();
    let last = exchange(inner, backend, epoch, &mut client, false, None).await?;
    let peer = backend.installation_peer();
    if let Some(peer) = peer {
        let state = inner.lock().map_err(|_| Problem::new(C::Closed))?;
        peer.validate(state.snapshot.description.as_ref())?;
    }
    let (sender, mut receiver) = mpsc::channel(1);
    update(inner, epoch, |state| {
        state.installation = peer.map(|peer| (peer, sender));
    });
    let mut pending = None;
    let mut link = Link { client, last, peer };
    let result = drive(
        inner,
        backend,
        epoch,
        &mut link,
        &mut receiver,
        &mut pending,
    )
    .await;
    if let Some(pending) = pending {
        pending.complete(Err(result
            .clone()
            .err()
            .unwrap_or_else(|| Problem::new(C::Lost))));
    }
    result
}

struct Link {
    client: Client,
    last: Stamp,
    peer: Option<InstallationPeer>,
}

async fn drive<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
    link: &mut Link,
    receiver: &mut mpsc::Receiver<Call>,
    active: &mut Option<Pending>,
) -> Result<(), Problem> {
    let Link { client, last, peer } = link;
    let peer = *peer;
    loop {
        let age = last.age();
        if age >= FRESH_MS {
            return Err(Problem::new(C::Timeout));
        }
        if backend.installation_peer() != peer {
            return Err(Problem::new(C::Installation));
        }
        let mut remaining = Duration::from_millis(FRESH_MS - age);
        if let Some(pending) = active.as_ref() {
            if pending.abandoned() {
                if pending.started() {
                    return Err(Problem::new(C::Lost));
                }
                *active = None;
                continue;
            }
            remaining = remaining.min(pending.remaining()?);
        }
        let interval = if active.is_some() { 1500 } else { 2000 };
        if age >= interval {
            *last = timeout(
                remaining,
                exchange(inner, backend, epoch, client, true, Some(*last)),
            )
            .await
            .map_err(|_| Problem::new(C::Timeout))??;
            continue;
        }
        if let Some(pending) = active.as_mut() {
            if let Some(fragment) = pending.fragment() {
                // A response before all request fragments is unsolicited. Responses
                // queued during the final write are read after that write completes.
                reject_unsolicited(backend)?;
                timeout(
                    remaining.min(Duration::from_millis(500)),
                    backend.write_installation(fragment),
                )
                .await
                .map_err(|_| Problem::new(C::Timeout))??;
                pending.remaining()?;
                pending.sent();
                tokio::task::yield_now().await;
                continue;
            }
            if let Some(bytes) = backend.try_installation_notification()? {
                if let Some(frame) = pending.receive(&bytes)? {
                    // Extra already-queued bytes cannot be consumed by the next request.
                    reject_unsolicited(backend)?;
                    ensure_current(inner, backend, epoch, peer)?;
                    if let Some(finished) = active.take() {
                        finished.complete(Ok(frame));
                    }
                }
                tokio::task::yield_now().await;
            } else {
                sleep(
                    remaining
                        .min(Duration::from_millis(20))
                        .min(Duration::from_millis(interval - age)),
                )
                .await;
            }
            continue;
        }
        if peer.is_some() {
            reject_unsolicited(backend)?;
        }
        tokio::select! {
            biased;
            () = sleep(Duration::from_millis(interval - age)) => {},
            call = receiver.recv(), if peer.is_some() => {
                let call = call.ok_or_else(|| Problem::new(C::Lost))?;
                if !call.reply.is_closed() {
                    // Recheck after waiting: a stray old response must not become a new one.
                    reject_unsolicited(backend)?;
                    *active = Some(Pending::new(call, peer.ok_or_else(|| Problem::new(C::Installation))?)?);
                }
            }
        }
    }
}

fn ensure_current(
    inner: &Mutex<Inner>,
    backend: &impl Transport,
    epoch: u32,
    peer: Option<InstallationPeer>,
) -> Result<(), Problem> {
    let mut state = inner.lock().map_err(|_| Problem::new(C::Closed))?;
    let snapshot = state.snapshot();
    if state.closed || snapshot.epoch != epoch || snapshot.phase != Phase::Connected {
        return Err(Problem::new(C::Lost));
    }
    if backend.installation_peer() != peer {
        return Err(Problem::new(C::Installation));
    }
    Ok(())
}

fn reject_unsolicited(backend: &mut impl Transport) -> Result<(), Problem> {
    if backend.try_installation_notification()?.is_some() {
        Err(Problem::new(C::Protocol))
    } else {
        Ok(())
    }
}
