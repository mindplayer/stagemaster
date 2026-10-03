//! Independent logical playback; this image retains OutputDisabled for its entire boot.
use super::{COMPLETIONS, REQUESTS, live_epoch, runtime, runtime_io};
use embassy_futures::select::{Either3, select3};
use embassy_time::{Duration, Ticker};
use stagemaster_install::Storage;
use stagemaster_install_worker::{ManagedWorker, runtime_queue::Endpoint};
use stagemaster_runtime::PlaybackPolicy;

pub(super) async fn serve<S: Storage, P: PlaybackPolicy>(worker: &mut ManagedWorker<S, P>) -> bool {
    let mut endpoint = Endpoint::default();
    let mut ticker = Ticker::every(Duration::from_millis(25));
    let mut install_reply: Option<stagemaster_install_worker::Completion> = None;
    let mut runtime_reply: Option<stagemaster_install_worker::runtime_queue::Completion> = None;
    let mut frame = [0; 512];
    loop {
        worker.observe(live_epoch());
        if endpoint
            .tick(worker, runtime::now(), runtime_io::live)
            .is_err()
        {
            return false;
        }
        // main holds the actual disabled transmitter; no output queue exists in this image.
        if let Some(request) = worker.quiescence_request()
            && worker.confirm_quiescent(request, runtime::now()).is_err()
        {
            return false;
        }
        if let Some(reply) = install_reply.take()
            && live_epoch() == Some(reply.epoch)
            && let Err(embassy_sync::channel::TrySendError::Full(reply)) =
                COMPLETIONS.try_send(reply)
        {
            install_reply = Some(reply);
        }
        if let Some(reply) = runtime_reply.take()
            && runtime_io::live().is_some_and(|live| {
                live.epoch() == reply.epoch() && live.grant(runtime::now()).is_some()
            })
            && let Err(embassy_sync::channel::TrySendError::Full(reply)) =
                runtime_io::COMPLETIONS.try_send(reply)
        {
            runtime_reply = Some(reply);
        }
        if install_reply.is_some() || runtime_reply.is_some() {
            // Backpressure is bounded; it never waits inside a blocking completion send.
            ticker.next().await;
            if endpoint
                .tick(worker, runtime::now(), runtime_io::live)
                .is_err()
                || worker.render(&mut frame).is_err()
            {
                return false;
            }
            continue;
        }
        match select3(
            ticker.next(),
            REQUESTS.receive(),
            runtime_io::REQUESTS.receive(),
        )
        .await
        {
            Either3::First(()) => {
                if endpoint
                    .tick(worker, runtime::now(), runtime_io::live)
                    .is_err()
                    || worker.render(&mut frame).is_err()
                {
                    return false;
                }
            }
            Either3::Second(command) => {
                super::sample_stack();
                let start = esp_hal::time::Instant::now();
                install_reply = Some(worker.process(command, runtime::now(), live_epoch));
                super::record_operation(start);
            }
            Either3::Third(command) => {
                super::sample_stack();
                let start = esp_hal::time::Instant::now();
                runtime_reply =
                    Some(endpoint.process(worker, command, runtime::now, runtime_io::live));
                super::record_operation(start);
            }
        }
    }
}
