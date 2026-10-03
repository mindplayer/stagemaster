mod runtime_support;
mod support;
use runtime_support::{Server, connected, description, installed, reply, rights};
use stagemaster_device_auth::application::Role;
use stagemaster_device_channel::{Channel, Error, StreamRecords, runtime::RuntimeClient};
use stagemaster_device_session::Kind;
use stagemaster_runtime::Action;
use stagemaster_runtime_protocol::{Access, Operation, Request};
use support::{config, peer::Peer, temporary};
use tokio::{io::duplex, time::Instant};

#[tokio::test]
async fn playback_without_runtime_capability_never_starts_a_runtime_handshake() {
    let (client, _server) = duplex(4096);
    let mut desc = description(7);
    desc.capabilities &= !stagemaster_device_info::capability::RUNTIME_APPLICATION;
    desc.limits.message_bytes = 0;
    assert!(desc.validate().is_ok());
    let r = Channel::prepare_runtime(
        StreamRecords::new(client),
        desc,
        &config(Role::Controller),
        rights(),
    )
    .await;
    assert!(matches!(r, Err(Error::Denied)));
}

#[tokio::test]
async fn runtime_never_accepts_old_installation_ready_or_elevates_an_installation_channel() {
    for runtime in [true, false] {
        let (client, server) = duplex(4096);
        let config = config(Role::Controller);
        let desc = if runtime {
            description(7)
        } else {
            support::description(7)
        };
        let opening = async {
            if runtime {
                Channel::prepare_runtime(StreamRecords::new(client), desc, &config, rights()).await
            } else {
                Channel::prepare(StreamRecords::new(client), desc, &config).await
            }
        };
        let (channel, server) = tokio::join!(
            opening,
            Peer::accept(StreamRecords::new(server), desc, false)
        );
        let _peer = server.unwrap();
        if runtime {
            assert!(channel.is_err());
        } else {
            assert!(RuntimeClient::new(channel.unwrap()).is_err());
        }
    }
}

#[tokio::test]
async fn authenticated_readiness_must_match_locally_expected_scopes() {
    let directory = temporary();
    let device = installed(directory.path());
    let (client, server) = duplex(4096);
    let config = config(Role::Controller);
    let (channel, _server) = tokio::join!(
        Channel::prepare_runtime(
            StreamRecords::new(client),
            description(7),
            &config,
            rights()
        ),
        Server::accept(
            StreamRecords::new(server),
            description(7),
            device,
            Instant::now(),
            Access {
                observe: true,
                control: false,
                installation: false
            }
        ),
    );
    assert!(channel.is_err());
}

#[tokio::test]
async fn observer_refusal_is_local_keeps_context_and_does_not_consume_request_id() {
    let (_directory, mut client, mut server) = connected(Access {
        observe: true,
        control: false,
        installation: false,
    })
    .await;
    let state = server.device.state();
    assert!(matches!(
        client
            .send(Operation::Apply(Action::Stop), state.revision)
            .await,
        Err(Error::Denied)
    ));
    assert!(client.pending().is_none());
    assert!(client.peer().is_some());
    let request = client
        .send(Operation::Status, state.revision)
        .await
        .unwrap();
    assert_eq!(request.id, 1);
    let (kind, bytes) = server.peer.receive().await.unwrap();
    assert_eq!(kind, Kind::Message);
    assert_eq!(Request::decode(&bytes).unwrap(), request);
    let response = server.process(&bytes);
    server
        .peer
        .send(Kind::Message, response.bytes())
        .await
        .unwrap();
    let received = reply(&mut client).await;
    assert_eq!(received.request, request);
    assert!(client.pending().is_none());
    assert_eq!(server.device.state().owner, None);
    assert_eq!(server.device.state().revision, state.revision);
}

#[tokio::test]
async fn a_channel_with_prior_commands_cannot_reset_its_sequence_by_wrapping_a_new_client() {
    let directory = temporary();
    let device = installed(directory.path());
    let (client, server) = duplex(4096);
    let config = config(Role::Controller);
    let (channel, mut peer) = tokio::join!(
        Channel::prepare_runtime(
            StreamRecords::new(client),
            description(7),
            &config,
            rights()
        ),
        Server::accept(
            StreamRecords::new(server),
            description(7),
            device,
            Instant::now(),
            rights()
        ),
    );
    let mut channel = channel.unwrap();
    let request = Request {
        session: channel.runtime_peer().unwrap().peer.session,
        id: 1,
        expected_revision: 0,
        operation: Operation::Status,
    };
    channel
        .write(request.encode().unwrap().bytes())
        .await
        .unwrap();
    let (_, bytes) = peer.peer.receive().await.unwrap();
    peer.process(&bytes);
    assert!(RuntimeClient::new(channel).is_err());
}
