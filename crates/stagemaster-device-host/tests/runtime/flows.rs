use crate::{adapter::*, runtime_support::rights};
use stagemaster_device_host::{Phase, ProblemCode, Request, RuntimeIntent};
use stagemaster_runtime::{Action, State, Status};
use stagemaster_runtime_protocol::{Body, Operation};

pub(super) async fn prepare(host: &Host) -> [u8; 16] {
    command(host, Operation::Status).await;
    let page = command(host, Operation::Catalog { index: 0 }).await;
    let Body::Program(Some(program)) = page.body else {
        panic!("missing program")
    };
    command(
        host,
        Operation::Acquire {
            duration_ms: 10_000,
            takeover: false,
        },
    )
    .await;
    command(host, Operation::Apply(Action::Select(program.key))).await;
    command(host, Operation::Apply(Action::Load)).await;
    let Body::Step(Some(step)) = command(host, Operation::Step { index: 0 }).await.body else {
        panic!("missing step")
    };
    step.id
}
fn state(body: &Body) -> State {
    let Body::State { state, result } = body else {
        panic!("missing state")
    };
    result.unwrap();
    *state
}

#[tokio::test(start_paused = true)]
async fn native_service_controls_original_runtime_and_reconnects_without_restarting_or_taking_control()
 {
    let (host, data) = setup();
    let epoch = connect(&host, rights()).await;
    assert_eq!(status(&host).phase, Phase::Connected);
    assert!(host.installation_peer(epoch).unwrap().is_none());
    assert!(data.lock().unwrap().commands.is_empty());
    let step = prepare(&host).await;
    let running = state(
        &command(&host, Operation::Apply(Action::Start { step }))
            .await
            .body,
    );
    assert_eq!(running.status, Some(Status::Running));
    assert_eq!(
        state(&command(&host, Operation::Apply(Action::Pause)).await.body).status,
        Some(Status::Paused)
    );
    command(&host, Operation::Apply(Action::Resume)).await;
    disconnect(&host).await;
    let old = host.runtime_snapshot(epoch).unwrap();
    assert!(old.peer.is_none());
    assert!(old.pending.is_none());
    assert!(old.last_response.is_some());
    {
        let s = data.lock().unwrap();
        assert_eq!(s.observation.instance, running.instance);
        assert_eq!(s.observation.owner, None);
        assert_eq!(s.observation.status, Some(Status::Running));
    }
    let next = connect(&host, rights()).await;
    assert_eq!(
        host.runtime_snapshot(epoch).unwrap_err().code,
        ProblemCode::Stale
    );
    assert!(host.runtime_snapshot(next).unwrap().last_response.is_none());
    assert_eq!(
        host.exchange_runtime(
            epoch,
            RuntimeIntent {
                operation: Operation::Status,
                expected_revision: 0
            }
        )
        .await
        .unwrap_err()
        .code,
        ProblemCode::Stale
    );
    let observed = state(&command(&host, Operation::Status).await.body);
    assert_eq!(observed.instance, running.instance);
    assert_eq!(observed.owner, None);
    command(
        &host,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
    )
    .await;
    command(&host, Operation::Apply(Action::Next)).await;
    assert_eq!(
        state(&command(&host, Operation::Apply(Action::Stop)).await.body).status,
        Some(Status::Idle)
    );
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn diagnostic_mode_does_not_become_runtime_and_connection_modes_cannot_overlap() {
    let (host, _) = setup();
    let epoch = connect(&host, rights()).await;
    assert_eq!(
        host.request(Request::Connect {
            epoch,
            id: "runtime".into()
        })
        .unwrap_err()
        .code,
        ProblemCode::Busy
    );
    disconnect(&host).await;
    host.request(Request::Connect {
        epoch,
        id: "runtime".into(),
    })
    .unwrap();
    settle().await;
    let current = status(&host).epoch;
    assert_eq!(status(&host).phase, Phase::Connected);
    assert!(host.runtime_snapshot(current).unwrap().peer.is_none());
    assert_eq!(
        host.exchange_runtime(
            current,
            RuntimeIntent {
                operation: Operation::Status,
                expected_revision: 0
            }
        )
        .await
        .unwrap_err()
        .code,
        ProblemCode::Runtime
    );
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn observer_denial_is_local_and_confirmed_business_failure_keeps_the_connection_usable() {
    let (host, data) = setup();
    let mut access = rights();
    access.control = false;
    let epoch = connect(&host, access).await;
    let denied = host
        .exchange_runtime(
            epoch,
            RuntimeIntent {
                operation: Operation::Apply(Action::Stop),
                expected_revision: 0,
            },
        )
        .await
        .unwrap_err();
    assert_eq!(denied.code, ProblemCode::Runtime);
    let observed = host.runtime_snapshot(epoch).unwrap();
    assert!(observed.peer.is_some());
    assert!(observed.pending.is_none());
    assert_eq!(command(&host, Operation::Status).await.request.id, 1);
    assert_eq!(data.lock().unwrap().commands.len(), 1);
    disconnect(&host).await;
    connect(&host, rights()).await;
    command(&host, Operation::Status).await;
    let failure = command(&host, Operation::Release).await;
    assert!(matches!(failure.body, Body::State { result: Err(_), .. }));
    assert_eq!(status(&host).phase, Phase::Connected);
    assert!(
        host.runtime_snapshot(status(&host).epoch)
            .unwrap()
            .pending
            .is_none()
    );
    host.shutdown().await.unwrap();
}
