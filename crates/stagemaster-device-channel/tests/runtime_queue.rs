#[path = "runtime_support/queued.rs"]
mod queued;
mod runtime_support;
mod support;
use runtime_support::{command, description, installed, rights};
use stagemaster_device_auth::application::Role;
use stagemaster_device_channel::{Channel, RecordIo, runtime::RuntimeClient};
use stagemaster_runtime::{Action, Status};
use stagemaster_runtime_protocol::{Body, Operation};
use support::{Task, config, packets, tcp, temporary};

async fn exercise<R: RecordIo>(client: R, server: R) {
    let dir = temporary();
    let device = installed(dir.path());
    let mut task = Task(tokio::spawn(queued::serve(
        server,
        device,
        tokio::time::Instant::now(),
    )));
    let channel =
        Channel::prepare_runtime(client, description(7), &config(Role::Controller), rights())
            .await
            .unwrap();
    let mut client = RuntimeClient::new(channel).unwrap();
    command(&mut client, Operation::Status).await;
    let entry = command(&mut client, Operation::Catalog { index: 0 }).await;
    let Body::Program(Some(program)) = entry.body else {
        panic!("missing program")
    };
    command(
        &mut client,
        Operation::Acquire {
            duration_ms: 10_000,
            takeover: false,
        },
    )
    .await;
    command(&mut client, Operation::Apply(Action::Select(program.key))).await;
    command(&mut client, Operation::Apply(Action::Load)).await;
    let step = command(&mut client, Operation::Step { index: 0 }).await;
    let Body::Step(Some(step)) = step.body else {
        panic!("missing step")
    };
    command(
        &mut client,
        Operation::Apply(Action::Start { step: step.id }),
    )
    .await;
    command(&mut client, Operation::Apply(Action::Pause)).await;
    let state = command(&mut client, Operation::Apply(Action::Resume)).await;
    let Body::State { state, .. } = state.body else {
        panic!("missing state")
    };
    client.close();
    let device = (&mut task.0).await.unwrap();
    assert_eq!(device.state().instance, state.instance);
    assert_eq!(device.state().status, Some(Status::Running));
    assert_eq!(device.state().owner, None);
    assert!(device.state().elapsed_ms >= state.elapsed_ms);
    assert!(device.render(&mut [0; 512]).unwrap().is_some());
}

#[tokio::test]
async fn tcp_client_crosses_real_bounded_queues_and_independent_worker() {
    let (client, server) = tcp().await;
    Box::pin(exercise(client, server)).await;
}

#[tokio::test]
async fn gatt_record_client_crosses_the_same_queues_and_worker() {
    let (client, server) = packets::pair();
    Box::pin(exercise(client, server)).await;
}
