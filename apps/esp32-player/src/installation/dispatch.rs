//! Independent playback. UART builds obtain maintenance from actual queue completion.
use super::{COMPLETIONS, REQUESTS, live_epoch, runtime, runtime_io};
use embassy_futures::select::{Either3, select3};
use embassy_time::{Duration, Ticker};
use stagemaster_install::Storage;
use stagemaster_install_worker::{ManagedWorker, runtime_queue::Endpoint};
use stagemaster_runtime::PlaybackPolicy;

pub(super) async fn serve<S: Storage, P: PlaybackPolicy>(
    worker: &mut ManagedWorker<S, P>,
    #[cfg(feature = "runtime-dmx-probe")] mut output: super::output::Output,
) -> bool {
    let mut endpoint = Endpoint::default();
    let mut ticker = Ticker::every(Duration::from_millis(25));
    let mut install_reply: Option<stagemaster_install_worker::Completion> = None;
    let mut runtime_reply: Option<stagemaster_install_worker::runtime_queue::Completion> = None;
    let mut frame = [0; 512];
    let mut sampler = super::frame_probe::Sampler::default();
    loop {
        worker.observe(live_epoch());
        if endpoint
            .tick(worker, runtime::now(), runtime_io::live)
            .is_err()
        {
            sampler.failed();
            return false;
        }
        #[cfg(feature = "runtime-dmx-probe")]
        {
            let result = output.service(worker, runtime::now());
            super::output::publish(&output);
            if result.is_err() {
                sampler.failed();
                return false;
            }
            crate::board::watchdog::WORKER.beat();
        }
        // Non-UART builds continuously own the disabled transmitter.
        #[cfg(not(feature = "runtime-dmx-probe"))]
        if let Some(request) = worker.quiescence_request()
            && worker.confirm_quiescent(request, runtime::now()).is_err()
        {
            sampler.failed();
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
            if !sample(
                &mut sampler,
                &mut endpoint,
                worker,
                &mut frame,
                true,
                #[cfg(feature = "runtime-dmx-probe")]
                &mut output,
            ) {
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
                if !sample(
                    &mut sampler,
                    &mut endpoint,
                    worker,
                    &mut frame,
                    false,
                    #[cfg(feature = "runtime-dmx-probe")]
                    &mut output,
                ) {
                    return false;
                }
            }
            Either3::Second(command) => {
                super::sample_stack();
                let start = esp_hal::time::Instant::now();
                install_reply = Some(worker.process(command, runtime::now(), live_epoch));
                sampler.command();
                super::record_operation(start);
            }
            Either3::Third(command) => {
                // Do not combine an almost-expired frame period with a slow load.
                // Endpoint still validates this command at its actual execution time.
                if sampler.before_command()
                    && !sample(
                        &mut sampler,
                        &mut endpoint,
                        worker,
                        &mut frame,
                        false,
                        #[cfg(feature = "runtime-dmx-probe")]
                        &mut output,
                    )
                {
                    return false;
                }
                super::sample_stack();
                let start = esp_hal::time::Instant::now();
                runtime_reply =
                    Some(endpoint.process(worker, command, runtime::now, runtime_io::live));
                sampler.command();
                super::record_operation(start);
            }
        }
    }
}

fn sample<S: Storage, P: PlaybackPolicy>(
    sampler: &mut super::frame_probe::Sampler,
    endpoint: &mut Endpoint,
    worker: &mut ManagedWorker<S, P>,
    frame: &mut [u8; 512],
    backpressured: bool,
    #[cfg(feature = "runtime-dmx-probe")] output: &mut super::output::Output,
) -> bool {
    let result = sampler.sample(endpoint, worker, frame, backpressured);
    #[cfg(feature = "runtime-dmx-probe")]
    if let Ok(Some(info)) = result {
        let published = output.publish(worker, info, frame, runtime::now());
        super::output::publish(output);
        if published.is_err() {
            sampler.failed();
            return false;
        }
    }
    #[cfg(feature = "runtime-dmx-probe")]
    if result.is_ok() {
        crate::board::watchdog::WORKER.beat();
    }
    result.is_ok()
}
