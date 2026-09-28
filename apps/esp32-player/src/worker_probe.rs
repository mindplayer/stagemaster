//! Compile-time local harness, not a wireless authorization or install endpoint.
use crate::installation::{COMPLETIONS, LIVE_EPOCH, READY, REQUESTS};
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Timer, with_timeout};
use stagemaster_install_worker::{Command, Epoch, Reply};
use stagemaster_transfer::{Action, AuthorizedLink, Request, Response};

#[cfg(feature = "worker-write-test")]
static PACKAGE: &[u8] = include_bytes!(env!("STAGEMASTER_PROBE_PACKAGE"));

async fn exchange(command: Command) -> Reply {
    let epoch = command.epoch();
    REQUESTS.send(command).await;
    let completion = with_timeout(Duration::from_secs(30), COMPLETIONS.receive())
        .await
        .unwrap();
    assert_eq!(completion.epoch, epoch);
    assert_eq!(crate::installation::live_epoch(), Some(epoch));
    completion.result.unwrap()
}

#[embassy_executor::task]
pub async fn run(boot: [u8; 16]) {
    for _ in 0..100 {
        match READY.load(Ordering::Acquire) {
            1 => break,
            2 => return,
            _ => Timer::after_millis(100).await,
        }
    }
    if READY.load(Ordering::Acquire) != 1 {
        esp_println::println!("LOCAL WORKER PROBE readiness timeout");
        return;
    }
    // Allow a separate real host to connect and measure heartbeat latency.
    Timer::after_secs(45).await;
    let epoch = Epoch::new(1).unwrap();
    LIVE_EPOCH.store(epoch.get(), Ordering::Release);
    let session = boot;
    let reply = exchange(Command::Open {
        epoch,
        link: AuthorizedLink {
            principal: [0x5a; 16],
            session,
        },
    })
    .await;
    assert!(matches!(reply, Reply::Opened));
    let reply = exchange(Command::Frame {
        epoch,
        frame: Request {
            link: session,
            id: 1,
            action: Action::Status,
        }
        .encode()
        .unwrap(),
    })
    .await;
    let Reply::Frame(frame) = reply else {
        panic!("expected status")
    };
    let response = Response::decode(frame.bytes()).unwrap();
    assert!(response.result.is_ok());
    esp_println::println!("LOCAL WORKER PROBE status {:?}", response.state);
    // A fresh local epoch avoids sharing request counters with Upload.
    LIVE_EPOCH.store(0, Ordering::Release);
    #[cfg(feature = "worker-write-test")]
    upload(boot).await;
    crate::installation::report();
    esp_println::println!("LOCAL WORKER PROBE PASS; no GATT install permission, RS485 disabled");
}

#[cfg(feature = "worker-write-test")]
async fn upload(mut session: [u8; 16]) {
    use stagemaster_transfer::{Outcome, Upload};
    let epoch = Epoch::new(2).unwrap();
    // Distinct public test session, never advertised as authenticated identity.
    session[0] ^= 0x80;
    if session == [0; 16] {
        session[15] = 1;
    }
    LIVE_EPOCH.store(epoch.get(), Ordering::Release);
    assert!(matches!(
        exchange(Command::Open {
            epoch,
            link: AuthorizedLink {
                principal: [0x5a; 16],
                session
            },
        })
        .await,
        Reply::Opened
    ));
    let mut upload = Upload::new(PACKAGE).unwrap();
    esp_println::println!("LOCAL WORKER WRITE TEST begin {:?}", upload.identity());
    upload.connect(session).unwrap();
    let start = esp_hal::time::Instant::now();
    while let Some(frame) = upload.outbound().unwrap().cloned() {
        let command = Request::decode(frame.bytes()).unwrap().action.command();
        let reply = exchange(Command::Frame { epoch, frame }).await;
        let Reply::Frame(frame) = reply else {
            panic!("expected frame")
        };
        upload.accept(frame.bytes()).unwrap();
        if command != stagemaster_transfer::Command::Write {
            esp_println::println!("LOCAL WORKER WRITE TEST command={:?}", command);
        }
    }
    assert!(matches!(upload.outcome(), Some(Outcome::Installed(_))));
    esp_println::println!(
        "LOCAL WORKER WRITE TEST installed {:?}; elapsed_us={}",
        upload.outcome(),
        start.elapsed().as_micros()
    );
    LIVE_EPOCH.store(0, Ordering::Release);
}

// Runs only on the storage-owning core, once for a matching durable commit.
// Produces software frames in RAM. No physical output driver exists here.
#[cfg(feature = "worker-write-test")]
pub fn verify_reply<S: stagemaster_install::Storage>(
    worker: &stagemaster_install_worker::Worker<S>,
    completion: &stagemaster_install_worker::Completion,
    replayed: &mut Option<stagemaster_install::Commit>,
) where
    S::Error: core::fmt::Debug,
{
    use sha2::{Digest, Sha256};
    let Ok(Reply::Frame(frame)) = &completion.result else {
        return;
    };
    let response = Response::decode(frame.bytes()).unwrap();
    let Some(head) = response.state.head else {
        return;
    };
    if response.result.is_err()
        || Some(head) == *replayed
        || head.identity.bytes != PACKAGE.len()
        || head.identity.digest != PACKAGE[32..64]
    {
        return;
    }
    let before = esp_alloc::HEAP.used();
    let installed = worker.snapshot().unwrap();
    assert_eq!(installed.commit(), head);
    for index in 0..installed.archive().entries().len() {
        let program = installed.load(index).unwrap();
        let mut player = stagemaster_playback::Player::new(program.plan, 0);
        player.execute(0, 0).unwrap();
        let mut digest = Sha256::new();
        for time in (0..10_000).step_by(25) {
            player.advance(time).unwrap();
            let mut frame = [0; 512];
            program.output.render(player.values(), &mut frame).unwrap();
            digest.update(frame);
        }
        esp_println::println!(
            "LOCAL WORKER REPLAY index={} frames=400 digest={:x} loaded_heap={} heap_peak={}",
            index,
            digest.finalize(),
            esp_alloc::HEAP.used(),
            esp_alloc::HEAP.stats().max_usage
        );
    }
    *replayed = Some(head);
    drop(installed);
    // This allocator is shared with live radio tasks on the other core. Two
    // immediate samples can differ by a transient BLE allocation. The host
    // checks retained heap after replay while heartbeats continue.
    esp_println::println!(
        "LOCAL WORKER REPLAY global_heap_before={} global_heap_after={}",
        before,
        esp_alloc::HEAP.used()
    );
    esp_println::println!("LOCAL WORKER REPLAY PASS; physical-output=false");
}
