use super::{
    Inner,
    state::{FRESH_MS, Stamp, millis},
    update,
};
use crate::{DeviceDescription, Diagnostics, Phase, Problem, ProblemCode as C, Transport};
use stagemaster_device_link::client::{Client, Diagnostics as WireDiagnostics, Error as WireError};
use std::{sync::Mutex, time::Duration};
use tokio::time::{Instant, sleep, timeout};
const IO_TIMEOUT: Duration = Duration::from_millis(2500);

pub(super) async fn exchange<B: Transport>(
    inner: &Mutex<Inner>,
    backend: &mut B,
    epoch: u32,
    client: &mut Client,
    heartbeat: bool,
    prior: Option<Stamp>,
) -> Result<Stamp, Problem> {
    let started = Instant::now();
    let stamp = timeout(IO_TIMEOUT, async {
        let request = client.request().map_err(protocol)?;
        backend.write(&request).await?;
        // Firmware stores the application receipt before acknowledging ATT write.
        // A mismatched cache may be read again; a request is NEVER resent.
        loop {
            let bytes = backend.reply().await?;
            match client.accept(&bytes) {
                Ok(()) => break,
                Err(WireError::Correlation) => sleep(Duration::from_millis(30)).await,
                Err(value) => return Err(protocol(value)),
            }
        }
        let stamp = Stamp::now();
        let diagnostics =
            WireDiagnostics::decode(&backend.diagnostics().await?).map_err(protocol)?;
        let description = if heartbeat {
            None
        } else {
            backend
                .description()
                .await?
                .map(|bytes| DeviceDescription::decode(&bytes, client.session_id().unwrap_or(0)))
                .transpose()?
        };
        Ok::<_, Problem>((stamp, Diagnostics::from(diagnostics), description))
    })
    .await
    .map_err(|_| Problem::new(C::Timeout))??;
    if prior.is_some_and(|last| last.age() >= FRESH_MS) {
        return Err(Problem::new(C::Timeout));
    }
    update(inner, epoch, |state| {
        state.snapshot.phase = Phase::Connected;
        state.snapshot.diagnostics = Some(stamp.1);
        if !heartbeat {
            state.snapshot.description = stamp.2;
        }
        state.snapshot.round_trip_ms = Some(millis(started.elapsed()));
        state.snapshot.heartbeat_count += u64::from(heartbeat);
        state.last_reply = Some(stamp.0);
        state.snapshot.last_reply_age_ms = Some(stamp.0.age());
    });
    Ok(stamp.0)
}
fn protocol(error: WireError) -> Problem {
    Problem::new(C::Protocol).detail(format!("{error:?}"))
}
