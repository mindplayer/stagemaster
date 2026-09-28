//! Compile-time local harness, not a wireless authorization or install endpoint.
use crate::installation::{COMPLETIONS, LIVE_EPOCH, READY, REQUESTS};
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Timer, with_timeout};
use stagemaster_install_worker::{Command, Endpoint, Epoch};
use stagemaster_transfer::{Action, Assembler, AuthorizedLink, Frame, Request, Response};

#[cfg(feature = "worker-write-test")]
static PACKAGE: &[u8] = include_bytes!(env!("STAGEMASTER_PROBE_PACKAGE"));

fn now() -> u64 {
    embassy_time::Instant::now().as_millis()
}

// Exercises the production byte-channel rules through the actual cross-core queue.
// Only this compile-time LOCAL harness supplies the public fixture grant. There
// is still no GATT writer or authentication adapter using these credentials.
struct LocalChannel {
    endpoint: Endpoint,
    payload: usize,
    requests: u32,
    incoming: u32,
    outgoing: u32,
}
impl LocalChannel {
    async fn open(epoch: Epoch, session: [u8; 16], payload: usize) -> Self {
        let (endpoint, command) = Endpoint::open(
            epoch,
            AuthorizedLink {
                principal: [0x5a; 16],
                session,
            },
            payload,
            now(),
        )
        .unwrap();
        let mut channel = Self {
            endpoint,
            payload,
            requests: 0,
            incoming: 0,
            outgoing: 0,
        };
        channel.publish_epoch();
        channel.dispatch(command).await;
        channel
    }

    fn publish_epoch(&self) {
        LIVE_EPOCH.store(
            self.endpoint.live_epoch().map_or(0, Epoch::get),
            Ordering::Release,
        );
    }

    async fn dispatch(&mut self, command: Command) {
        assert_eq!(Some(command.epoch()), self.endpoint.live_epoch());
        if REQUESTS.try_send(command).is_err() {
            self.endpoint.close();
            self.publish_epoch();
            panic!("local worker queue full");
        }
        loop {
            let polled = self.endpoint.poll(now());
            self.publish_epoch();
            polled.unwrap();
            if let Ok(completion) =
                with_timeout(Duration::from_millis(100), COMPLETIONS.receive()).await
            {
                let completed = self.endpoint.complete(completion, now());
                self.publish_epoch();
                if completed.unwrap() {
                    break;
                }
                // Obsolete completions are consumed, never left blocking the worker.
            }
        }
    }

    async fn exchange(&mut self, frame: &Frame) -> Frame {
        let mut completion_received = false;
        for bytes in frame.bytes().chunks(self.payload) {
            self.incoming += 1;
            let result = self.endpoint.receive(bytes, now());
            self.publish_epoch();
            if let Some(command) = result.unwrap() {
                assert!(!completion_received);
                self.dispatch(command).await;
                completion_received = true;
            }
        }
        assert!(completion_received);
        let mut response = Assembler::new();
        loop {
            let ready = match self.endpoint.fragment(now()) {
                Ok(Some(bytes)) => {
                    self.outgoing += 1;
                    response.push(bytes).unwrap()
                }
                Ok(None) => break,
                Err(error) => {
                    self.publish_epoch();
                    panic!("local channel fragment failed: {}", error);
                }
            };
            let result = self.endpoint.sent(now());
            self.publish_epoch();
            assert_eq!(result.unwrap(), ready);
        }
        self.requests += 1;
        response.take().unwrap()
    }

    fn close(&mut self) {
        self.endpoint.close();
        self.publish_epoch();
        esp_println::println!(
            "LOCAL BYTE CHANNEL payload={} requests={} incoming_parts={} outgoing_parts={} endpoint_bytes={}",
            self.payload,
            self.requests,
            self.incoming,
            self.outgoing,
            core::mem::size_of::<Endpoint>()
        );
    }
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
    let session = boot;
    let mut channel = LocalChannel::open(epoch, session, 20).await;
    let frame = channel
        .exchange(
            &Request {
                link: session,
                id: 1,
                action: Action::Status,
            }
            .encode()
            .unwrap(),
        )
        .await;
    let response = Response::decode(frame.bytes()).unwrap();
    assert!(response.result.is_ok());
    esp_println::println!("LOCAL WORKER PROBE status {:?}", response.state);
    // A fresh local epoch avoids sharing request counters with Upload.
    channel.close();
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
    let mut channel = LocalChannel::open(epoch, session, 244).await;
    let mut upload = Upload::new(PACKAGE).unwrap();
    esp_println::println!("LOCAL WORKER WRITE TEST begin {:?}", upload.identity());
    upload.connect(session).unwrap();
    let start = esp_hal::time::Instant::now();
    while let Some(frame) = upload.outbound().unwrap().cloned() {
        let command = Request::decode(frame.bytes()).unwrap().action.command();
        let frame = channel.exchange(&frame).await;
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
    channel.close();
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
    let Ok(stagemaster_install_worker::Reply::Frame(frame)) = &completion.result else {
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
