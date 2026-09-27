use stagemaster_device_host::{
    Candidate, Phase, Problem, ProblemCode as C, Request, Service, Snapshot, Transport,
};
use stagemaster_device_link::{Packet, Session};
use std::{
    collections::VecDeque,
    future::pending,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)] // Independent injected transport faults, not product state.
struct State {
    calls: Vec<&'static str>,
    devices: VecDeque<Candidate>,
    server: Option<Session>,
    reply: Vec<u8>,
    writes: usize,
    stale: usize,
    block_connect: bool,
    panic_connect: bool,
    block_cleanup: bool,
    fail_scan: bool,
    fail_info: bool,
    wrong_reply: bool,
    self_test: bool,
    outputs_disabled: bool,
}
struct Fake(Arc<Mutex<State>>);
impl Fake {
    fn call(&self, call: &'static str) {
        self.0.lock().unwrap().calls.push(call);
    }
}
impl Transport for Fake {
    async fn start_scan(&mut self) -> Result<(), Problem> {
        self.call("scan");
        if self.0.lock().unwrap().fail_scan {
            Err(Problem::new(C::Permission))
        } else {
            Ok(())
        }
    }
    async fn discover(&mut self) -> Result<Candidate, Problem> {
        let next = self.0.lock().unwrap().devices.pop_front();
        match next {
            Some(value) => Ok(value),
            None => pending().await,
        }
    }
    async fn stop_scan(&mut self) -> Result<(), Problem> {
        self.call("stop_scan");
        if self.0.lock().unwrap().block_cleanup {
            pending::<()>().await;
        }
        Ok(())
    }
    async fn connect(&mut self, _id: &str) -> Result<(), Problem> {
        self.call("connect");
        let panic_connect = self.0.lock().unwrap().panic_connect;
        assert!(!panic_connect, "injected driver panic");
        if self.0.lock().unwrap().block_connect {
            pending::<()>().await;
        }
        self.0.lock().unwrap().server = Some(Session::new(45, 0).unwrap());
        Ok(())
    }
    async fn write(&mut self, bytes: &[u8; 20]) -> Result<(), Problem> {
        self.call("write");
        let mut state = self.0.lock().unwrap();
        state.reply = state
            .server
            .as_mut()
            .unwrap()
            .receive(bytes, 0)
            .encode()
            .to_vec();
        state.writes += 1;
        Ok(())
    }
    async fn reply(&mut self) -> Result<Vec<u8>, Problem> {
        self.call("reply");
        let mut state = self.0.lock().unwrap();
        if state.stale > 0 || state.wrong_reply {
            state.stale = state.stale.saturating_sub(1);
            let mut packet = Packet::decode(&state.reply).unwrap();
            packet.sequence += 1;
            return Ok(packet.encode().to_vec());
        }
        Ok(state.reply.clone())
    }
    async fn diagnostics(&mut self) -> Result<Vec<u8>, Problem> {
        self.call("info");
        let state = self.0.lock().unwrap();
        if state.fail_info {
            return Err(Problem::new(C::Lost));
        }
        let mut bytes = vec![0; 20];
        bytes[0] = 1;
        bytes[1] = u8::from(state.self_test) | (u8::from(state.outputs_disabled) << 1);
        bytes[16..20].copy_from_slice(&(128 * 1024_u32).to_le_bytes());
        Ok(bytes)
    }
    async fn disconnect(&mut self) -> Result<(), Problem> {
        self.call("disconnect");
        if self.0.lock().unwrap().block_cleanup {
            pending::<()>().await;
        }
        self.0.lock().unwrap().server = None;
        Ok(())
    }
}
fn candidate(index: usize) -> Candidate {
    Candidate {
        id: index.to_string(),
        name: format!("设备 {index}"),
        rssi: Some(-60),
    }
}
fn setup(count: usize) -> (Service<Fake>, Arc<Mutex<State>>) {
    let state = Arc::new(Mutex::new(State {
        devices: (0..count).map(candidate).collect(),
        self_test: true,
        outputs_disabled: true,
        ..State::default()
    }));
    (Service::new(Fake(state.clone())), state)
}
fn status(service: &Service<Fake>) -> Snapshot {
    service.request(Request::Status).unwrap()
}
async fn settle() {
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
}
async fn scan(service: &Service<Fake>) -> Snapshot {
    let epoch = status(service).epoch;
    service.request(Request::Scan { epoch }).unwrap();
    settle().await;
    tokio::time::advance(Duration::from_secs(8)).await;
    settle().await;
    let state = status(service);
    assert_eq!(state.phase, Phase::Idle);
    state
}
async fn connect(service: &Service<Fake>) -> Snapshot {
    let state = scan(service).await;
    service
        .request(Request::Connect {
            epoch: state.epoch,
            id: "0".into(),
        })
        .unwrap();
    settle().await;
    status(service)
}

#[tokio::test(start_paused = true)]
async fn bounded_discovery_deduplicates_then_stops_and_rejects_old_operations() {
    let (service, state) = setup(40);
    state.lock().unwrap().devices.push_front(candidate(0));
    let initial = status(&service);
    assert!(
        state.lock().unwrap().calls.is_empty(),
        "viewing status must not acquire BLE permission"
    );
    let ready = scan(&service).await;
    assert_eq!(ready.candidates.len(), 32);
    assert!(ready.scan_performed);
    assert!(ready.truncated);
    assert_eq!(state.lock().unwrap().calls, ["scan", "stop_scan"]);
    assert_eq!(
        service
            .request(Request::Scan {
                epoch: initial.epoch
            })
            .unwrap_err()
            .code,
        C::Stale
    );
    assert_eq!(
        service
            .request(Request::Connect {
                epoch: ready.epoch,
                id: "39".into()
            })
            .unwrap_err()
            .code,
        C::Unavailable
    );
    service.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn heartbeat_lives_outside_ui_and_disconnect_clears_telemetry_and_old_cancel() {
    let (service, state) = setup(1);
    let connected = connect(&service).await;
    assert_eq!(connected.phase, Phase::Connected);
    assert!(connected.diagnostics.unwrap().self_test);
    for _ in 0..5 {
        tokio::time::advance(Duration::from_secs(2)).await;
        settle().await;
    }
    let active = status(&service);
    assert_eq!(active.heartbeat_count, 5);
    assert_eq!(state.lock().unwrap().writes, 6);
    assert_eq!(
        service
            .request(Request::Scan {
                epoch: active.epoch
            })
            .unwrap_err()
            .code,
        C::Busy
    );
    service
        .request(Request::Cancel {
            epoch: active.epoch,
        })
        .unwrap();
    settle().await;
    let idle = status(&service);
    assert_eq!(idle.phase, Phase::Idle);
    assert!(idle.diagnostics.is_none() && idle.last_reply_age_ms.is_none());
    service
        .request(Request::Connect {
            epoch: idle.epoch,
            id: "0".into(),
        })
        .unwrap();
    settle().await;
    assert_eq!(
        service
            .request(Request::Cancel {
                epoch: active.epoch
            })
            .unwrap_err()
            .code,
        C::Stale
    );
    assert_eq!(status(&service).phase, Phase::Connected);
    service.shutdown().await.unwrap();
    assert_eq!(
        state
            .lock()
            .unwrap()
            .calls
            .iter()
            .filter(|c| **c == "disconnect")
            .count(),
        2
    );
    assert_eq!(
        service.request(Request::Status).unwrap_err().code,
        C::Closed
    );
}

#[tokio::test(start_paused = true)]
async fn cancelling_pending_connect_cleans_before_allowing_another_operation() {
    let (service, state) = setup(1);
    state.lock().unwrap().block_connect = true;
    let connecting = connect(&service).await;
    assert_eq!(connecting.phase, Phase::Connecting);
    let stopping = service
        .request(Request::Cancel {
            epoch: connecting.epoch,
        })
        .unwrap();
    assert_eq!(stopping.phase, Phase::Stopping);
    assert_eq!(
        service
            .request(Request::Scan {
                epoch: connecting.epoch
            })
            .unwrap_err()
            .code,
        C::Busy
    );
    settle().await;
    assert_eq!(status(&service).phase, Phase::Idle);
    assert_eq!(state.lock().unwrap().calls.last(), Some(&"disconnect"));
    assert_eq!(state.lock().unwrap().writes, 0);
    state.lock().unwrap().block_connect = false;
    service
        .request(Request::Connect {
            epoch: connecting.epoch,
            id: "0".into(),
        })
        .unwrap();
    settle().await;
    assert_eq!(status(&service).phase, Phase::Connected);
    service.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn cleanup_timeout_blocks_reuse_and_never_claims_a_disconnect() {
    let (service, state) = setup(1);
    let connected = connect(&service).await;
    state.lock().unwrap().block_cleanup = true;
    service
        .request(Request::Cancel {
            epoch: connected.epoch,
        })
        .unwrap();
    settle().await;
    tokio::time::advance(Duration::from_secs(5)).await;
    settle().await;
    let failed = status(&service);
    assert_eq!(failed.phase, Phase::Blocked);
    assert_eq!(failed.problem.unwrap().code, C::Cleanup);
    assert!(failed.diagnostics.is_none());
    assert_eq!(
        service
            .request(Request::Scan {
                epoch: failed.epoch
            })
            .unwrap_err()
            .code,
        C::Cleanup
    );
    assert_eq!(service.shutdown().await.unwrap_err().code, C::Cleanup);
}

#[tokio::test(start_paused = true)]
async fn stale_cache_is_reread_but_never_resends_a_heartbeat() {
    let (service, state) = setup(1);
    state.lock().unwrap().stale = 2;
    let connecting = connect(&service).await;
    assert_eq!(connecting.phase, Phase::Connecting);
    for _ in 0..2 {
        tokio::time::advance(Duration::from_millis(30)).await;
        settle().await;
    }
    assert_eq!(status(&service).phase, Phase::Connected);
    assert_eq!(state.lock().unwrap().writes, 1);
    service.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn mismatched_receipt_and_failed_diagnostics_never_report_online() {
    for wrong_reply in [true, false] {
        let (service, state) = setup(1);
        {
            let mut value = state.lock().unwrap();
            value.wrong_reply = wrong_reply;
            value.fail_info = !wrong_reply;
        }
        connect(&service).await;
        tokio::time::advance(Duration::from_millis(2500)).await;
        settle().await;
        let failed = status(&service);
        assert_eq!(failed.phase, Phase::Fault);
        assert!(failed.diagnostics.is_none());
        assert_eq!(state.lock().unwrap().writes, 1);
        assert_eq!(state.lock().unwrap().calls.last(), Some(&"disconnect"));
        service.shutdown().await.unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn expired_scheduler_gap_cannot_silently_resume_or_keep_old_diagnostics() {
    let (service, state) = setup(1);
    connect(&service).await;
    tokio::time::advance(Duration::from_secs(7)).await;
    let expired = status(&service);
    assert_ne!(expired.phase, Phase::Connected);
    assert!(expired.diagnostics.is_none());
    settle().await;
    assert_eq!(status(&service).phase, Phase::Fault);
    assert_eq!(state.lock().unwrap().writes, 1);
    service.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn permission_failure_can_retry_and_connect_timeout_releases_pending_handle() {
    let (service, state) = setup(1);
    state.lock().unwrap().fail_scan = true;
    service.request(Request::Scan { epoch: 0 }).unwrap();
    settle().await;
    let failed = status(&service);
    assert_eq!(failed.phase, Phase::Fault);
    assert_eq!(failed.problem.unwrap().code, C::Permission);
    {
        let mut value = state.lock().unwrap();
        value.fail_scan = false;
        value.block_connect = true;
    }
    connect(&service).await;
    tokio::time::advance(Duration::from_secs(15)).await;
    settle().await;
    assert_eq!(status(&service).problem.unwrap().code, C::Timeout);
    assert_eq!(state.lock().unwrap().calls.last(), Some(&"disconnect"));
    service.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn diagnostics_flags_are_reported_as_read_and_not_forged_from_connection_success() {
    let (service, state) = setup(1);
    {
        let mut value = state.lock().unwrap();
        value.self_test = false;
        value.outputs_disabled = false;
    }
    let connected = connect(&service).await;
    let diagnostics = connected.diagnostics.unwrap();
    assert!(!diagnostics.self_test && !diagnostics.output_disabled);
    service.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn driver_panic_is_cleaned_and_quarantined_instead_of_leaving_a_stuck_online_state() {
    let (service, state) = setup(1);
    state.lock().unwrap().panic_connect = true;
    let failed = connect(&service).await;
    assert_eq!(failed.phase, Phase::Blocked);
    assert!(failed.diagnostics.is_none());
    assert_eq!(state.lock().unwrap().calls.last(), Some(&"disconnect"));
    assert_eq!(service.shutdown().await.unwrap_err().code, C::Cleanup);
}

#[tokio::test(start_paused = true)]
async fn cancel_before_worker_starts_does_not_begin_scanning_or_acquire_permission() {
    let (service, state) = setup(1);
    let scanning = service.request(Request::Scan { epoch: 0 }).unwrap();
    service
        .request(Request::Cancel {
            epoch: scanning.epoch,
        })
        .unwrap();
    settle().await;
    assert_eq!(status(&service).phase, Phase::Idle);
    assert!(!status(&service).scan_performed);
    assert_eq!(state.lock().unwrap().calls, ["stop_scan"]);
    service.shutdown().await.unwrap();
}

#[test]
fn request_contract_rejects_unknown_mutations_and_fields() {
    assert!(serde_json::from_str::<Request>(r#"{"kind":"upload","epoch":1}"#).is_err());
    assert!(
        serde_json::from_str::<Request>(r#"{"kind":"scan","epoch":1,"address":"anything"}"#)
            .is_err()
    );
}

#[tokio::test(start_paused = true)]
async fn dropping_the_service_cannot_leave_a_detached_heartbeat_loop() {
    let (service, state) = setup(1);
    connect(&service).await;
    drop(service);
    settle().await;
    assert_eq!(state.lock().unwrap().calls.last(), Some(&"disconnect"));
    tokio::time::advance(Duration::from_secs(30)).await;
    settle().await;
    assert_eq!(state.lock().unwrap().writes, 1);
    assert!(state.lock().unwrap().server.is_none());
}
