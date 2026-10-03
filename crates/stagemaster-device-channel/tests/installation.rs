mod support;
use stagemaster_device_auth::application::Role;
use stagemaster_device_channel::{Channel, RecordIo};
use stagemaster_device_session::Kind;
use stagemaster_install::Installer;
use stagemaster_install_store::FileStore;
use stagemaster_package::Archive;
use stagemaster_transfer::{AuthorizedLink, Outcome, Service, Upload};
use support::{
    Task, config, description, message, package::package, packets, peer::Peer, tcp, temporary,
};

async fn install<R: RecordIo>(client: R, server: R) {
    let directory = temporary();
    let path = directory.path().to_owned();
    let bytes = package();
    let (stop, mut stopping) = tokio::sync::oneshot::channel();
    let mut endpoint = Task(tokio::spawn(async move {
        let mut peer = Peer::authenticate(server, description(7)).await.unwrap();
        let grant = peer.secure.grant(support::now(peer.origin)).unwrap();
        let mut service = Service::new(
            Installer::open(FileStore::open(&path).unwrap(), [2; 16])
                .unwrap()
                .0,
        )
        .unwrap();
        service
            .attach(AuthorizedLink {
                principal: grant.principal(),
                session: grant.session(),
            })
            .unwrap();
        // Match the device boundary: announce readiness only after the actual installer opens.
        peer.ready(description(7), false).await.unwrap();
        loop {
            let incoming = tokio::select! {
                _ = &mut stopping => break,
                result = peer.receive() => result.unwrap(),
            };
            match incoming {
                (Kind::Heartbeat, _) => peer.send(Kind::HeartbeatReply, &[]).await.unwrap(),
                (Kind::Message, message) => {
                    peer.secure.grant(support::now(peer.origin)).unwrap();
                    let response = service.process(&message).unwrap();
                    peer.send(Kind::Message, response.bytes()).await.unwrap();
                }
                _ => panic!("意外消息"),
            }
        }
        service.detach();
        service.snapshot().unwrap()
    }));
    let mut channel = Channel::prepare(client, description(7), &config(Role::Controller))
        .await
        .unwrap();
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    upload.connect(channel.peer().unwrap().session).unwrap();
    for _ in 0..100 {
        let Some(frame) = upload.outbound().unwrap().cloned() else {
            break;
        };
        channel.write(frame.bytes()).await.unwrap();
        // Heartbeat can interleave with the installation response, which must remain available.
        channel.heartbeat().await.unwrap();
        upload
            .accept(&message(&mut channel).await.unwrap())
            .unwrap();
    }
    assert!(matches!(upload.outcome(),Some(Outcome::Installed(c)) if c.generation==1));
    stop.send(()).unwrap();
    let snapshot = (&mut endpoint.0).await.unwrap();
    let expected = Archive::open(bytes.as_slice()).unwrap();
    assert_eq!(snapshot.archive().entries(), expected.entries());
    let installed = std::fs::read(
        directory
            .path()
            .join(format!("slot-{}.smpkg", snapshot.commit().slot.index())),
    )
    .unwrap();
    assert_eq!(installed, bytes);
}
#[tokio::test]
async fn real_tcp_and_gatt_sized_packets_install_the_same_package() {
    let (client, server) = tcp().await;
    Box::pin(install(client, server)).await;
    let (client, server) = packets::pair();
    Box::pin(install(client, server)).await;
}
