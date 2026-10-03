use super::{Call, check_connected};
use crate::{
    Problem, ProblemCode as C, Transport,
    service::{
        CONNECT_TIMEOUT, Inner,
        diagnostics::exchange,
        state::{FRESH_MS, Stamp},
        update,
    },
};
use stagemaster_device_link::client::Client;
use stagemaster_runtime_protocol::{Access, Ready, Request};
use std::{sync::Mutex, time::Duration};
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};

struct Pending {
    call: Call,
    request: Request,
}
struct Link {
    client: Client,
    last: Stamp,
    peer: Ready,
}

pub(in crate::service) async fn run<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
    id: &str,
    expected: Access,
) -> Result<(), Problem> {
    let started = tokio::time::Instant::now();
    timeout(CONNECT_TIMEOUT, backend.connect_runtime(id, expected))
        .await
        .map_err(|_| Problem::new(C::Timeout))??;
    let mut client = Client::default();
    let last = exchange(inner, backend, epoch, &mut client, false, None).await?;
    let peer = backend
        .runtime_peer()
        .ok_or_else(|| Problem::new(C::Runtime))?;
    {
        let mut state = inner.lock().map_err(|_| Problem::new(C::Closed))?;
        check_connected(&mut state, epoch)?;
        crate::runtime::validate(peer, state.snapshot.description.as_ref())?;
        if peer.access != expected || backend.installation_peer().is_some() {
            return Err(Problem::new(C::Runtime));
        }
    }
    let (sender, mut receiver) = mpsc::channel(1);
    update(inner, epoch, |state| {
        state.runtime_calls = Some((peer, sender));
        state.runtime_state.peer = Some(peer);
        state.runtime_until = Some(started + Duration::from_millis(u64::from(peer.remaining_ms)));
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
        pending.call.complete(Err(result
            .clone()
            .err()
            .unwrap_or_else(|| Problem::new(C::Lost))));
    }
    result
}

async fn drive<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
    link: &mut Link,
    receiver: &mut mpsc::Receiver<Call>,
    active: &mut Option<Pending>,
) -> Result<(), Problem> {
    loop {
        let age = link.last.age();
        if age >= FRESH_MS {
            return Err(Problem::new(C::Timeout));
        }
        let mut remaining = Duration::from_millis(FRESH_MS - age);
        if let Some(pending) = active.as_ref() {
            if pending.call.reply.is_closed() {
                return Err(Problem::new(C::Lost));
            }
            remaining = remaining.min(pending.call.remaining()?);
        }
        if backend.runtime_peer() != Some(link.peer) {
            return Err(Problem::new(C::Runtime));
        }
        if let Some(response) = backend.try_runtime_response()? {
            let pending = active.take().ok_or_else(|| Problem::new(C::Protocol))?;
            response
                .correlate(pending.request, link.peer.peer.boot)
                .map_err(|e| Problem::new(C::Protocol).detail(e.to_string()))?;
            {
                let mut state = inner.lock().map_err(|_| Problem::new(C::Closed))?;
                check_connected(&mut state, epoch)?;
                if backend.runtime_peer() != Some(link.peer) {
                    return Err(Problem::new(C::Runtime));
                }
                state.runtime_state.pending = None;
                state.runtime_request_until = None;
                state.runtime_state.last_response = Some(response);
                state.touch();
            }
            pending.call.complete(Ok(response));
            continue;
        }
        if age >= 1500 {
            link.last = timeout(
                remaining,
                exchange(
                    inner,
                    backend,
                    epoch,
                    &mut link.client,
                    true,
                    Some(link.last),
                ),
            )
            .await
            .map_err(|_| Problem::new(C::Timeout))??;
            continue;
        }
        if active.is_some() {
            sleep(
                remaining
                    .min(Duration::from_millis(20))
                    .min(Duration::from_millis(1500 - age)),
            )
            .await;
            continue;
        }
        tokio::select! {
            biased;
            () = sleep(Duration::from_millis(1500-age)) => {},
            call = receiver.recv() => {
                let call = call.ok_or_else(|| Problem::new(C::Lost))?;
                if call.reply.is_closed() { continue; }
                start(inner, backend, epoch, link.peer, call, active).await?;
            }
        }
    }
}

async fn start<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
    peer: Ready,
    call: Call,
    active: &mut Option<Pending>,
) -> Result<(), Problem> {
    // Register intent before any cancellable send. clear_live preserves this uncertainty.
    {
        let mut state = inner.lock().map_err(|_| Problem::new(C::Closed))?;
        check_connected(&mut state, epoch)?;
        state.runtime_state.pending = Some(call.intent);
        state.runtime_request_until = Some(call.deadline);
        state.touch();
    }
    let result = timeout(
        call.remaining()?.min(Duration::from_millis(2500)),
        backend.send_runtime(call.intent),
    )
    .await
    .map_err(|_| Problem::new(C::Timeout))
    .and_then(|r| r);
    match result {
        Ok(request) => {
            if request.operation != call.intent.operation
                || request.expected_revision != call.intent.expected_revision
                || request.session != peer.peer.session
            {
                return Err(Problem::new(C::Protocol));
            }
            *active = Some(Pending { call, request });
            Ok(())
        }
        Err(error)
            if backend.runtime_peer() == Some(peer) && backend.runtime_pending().is_none() =>
        {
            update(inner, epoch, |state| {
                state.runtime_state.pending = None;
                state.runtime_request_until = None;
            });
            call.complete(Err(error));
            Ok(())
        }
        Err(error) => {
            call.complete(Err(error.clone()));
            Err(error)
        }
    }
}
