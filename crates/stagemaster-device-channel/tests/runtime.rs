mod runtime_support;
mod support;
use runtime_support::{Server, command, description, installed, rights};
use stagemaster_device_auth::application::Role;
use stagemaster_device_channel::{Channel, RecordIo, runtime::RuntimeClient};
use stagemaster_runtime::{Action, Status};
use stagemaster_runtime_protocol::{Body, Operation};
use support::{Task, config, packets, tcp, temporary};
use tokio::time::Instant;

async fn exercise<R: RecordIo>(client: R, server: R, next_client: R, next_server: R) {
    let dir = temporary();
    let device = installed(dir.path());
    let origin = Instant::now();
    let mut task = Task(tokio::spawn(async move {
        Server::accept(server, description(7), device, origin, rights())
            .await
            .serve()
            .await
    }));
    let channel =
        Channel::prepare_runtime(client, description(7), &config(Role::Controller), rights())
            .await
            .unwrap();
    assert!(channel.peer().is_none());
    assert!(channel.runtime_peer().is_some());
    channel
        .correlate(&description(7).encode().unwrap())
        .unwrap();
    assert!(
        channel
            .correlate(&description(8).encode().unwrap())
            .is_err()
    );
    let mut client = RuntimeClient::new(channel).unwrap();
    let status = command(&mut client, Operation::Status).await;
    assert!(
        matches!(status.body, Body::State { state, .. } if state.owner.is_none() && state.loaded.is_none())
    );
    let catalog = command(&mut client, Operation::Catalog { index: 0 }).await;
    let Body::Program(Some(program)) = catalog.body else {
        panic!("missing installed program")
    };
    assert!(!program.name.as_str().is_empty());
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
    let page = command(&mut client, Operation::Step { index: 0 }).await;
    let Body::Step(Some(step)) = page.body else {
        panic!("missing loaded step")
    };
    let started = command(
        &mut client,
        Operation::Apply(Action::Start { step: step.id }),
    )
    .await;
    let Body::State { state: running, .. } = started.body else {
        unreachable!()
    };
    assert_eq!(running.status, Some(Status::Running));
    command(&mut client, Operation::Apply(Action::Pause)).await;
    command(&mut client, Operation::Apply(Action::Resume)).await;
    client.close();
    let device = (&mut task.0).await.unwrap();
    assert_eq!(device.state().instance, running.instance);
    assert_eq!(device.state().status, Some(Status::Running));
    assert_eq!(device.state().owner, None);
    check_frame(&device);
    Box::pin(reconnect(
        next_client,
        next_server,
        device,
        origin,
        running.instance,
    ))
    .await;
}

async fn reconnect<R: RecordIo>(
    next_client: R,
    next_server: R,
    device: runtime_support::device::Device,
    origin: Instant,
    instance: Option<stagemaster_runtime::Instance>,
) {
    let mut again = Task(tokio::spawn(async move {
        Server::accept(next_server, description(8), device, origin, rights())
            .await
            .serve()
            .await
    }));
    let channel = Channel::prepare_runtime(
        next_client,
        description(8),
        &config(Role::Controller),
        rights(),
    )
    .await
    .unwrap();
    let mut fresh = RuntimeClient::new(channel).unwrap();
    assert!(fresh.pending().is_none());
    assert!(fresh.last_response().is_none());
    let status = command(&mut fresh, Operation::Status).await;
    assert!(
        matches!(status.body, Body::State { state, .. } if state.instance == instance && state.owner.is_none())
    );
    command(
        &mut fresh,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
    )
    .await;
    command(&mut fresh, Operation::Apply(Action::Next)).await;
    command(&mut fresh, Operation::Apply(Action::Stop)).await;
    fresh.close();
    let device = (&mut again.0).await.unwrap();
    assert_eq!(device.state().instance, None);
    assert_eq!(device.state().status, Some(Status::Idle));
}

fn check_frame(device: &runtime_support::device::Device) {
    let loaded = device.snapshot().unwrap().load(0).unwrap();
    let mut reference = stagemaster_playback::Player::new(loaded.plan, 0);
    reference.execute(0, 0).unwrap();
    reference.advance(device.state().elapsed_ms).unwrap();
    let mut expected = [0; 512];
    let mut actual = [0; 512];
    loaded
        .output
        .render(reference.values(), &mut expected)
        .unwrap();
    device.render(&mut actual).unwrap().unwrap();
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn tcp_runtime_and_reconnection_control_original_installed_program() {
    let (client, server) = tcp().await;
    let (next_client, next_server) = tcp().await;
    Box::pin(exercise(client, server, next_client, next_server)).await;
}

#[tokio::test]
async fn gatt_sized_fragments_use_the_same_runtime_client_and_messages() {
    let (client, server) = packets::pair();
    let (next_client, next_server) = packets::pair();
    Box::pin(exercise(client, server, next_client, next_server)).await;
}
