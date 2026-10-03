mod support;
use stagemaster_device_auth::application::Role;
use stagemaster_device_channel::{Channel, Error, StreamRecords};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};
use support::{Task, config, description, faults::FaultIo, peer::Peer};
use tokio::{io::duplex, time::Instant};

#[tokio::test]
async fn cancelled_or_failed_application_write_never_reuses_the_secure_session() {
    for fault in [1, 2] {
        let (client, server) = duplex(4096);
        let mode = Arc::new(AtomicU8::new(0));
        let io = FaultIo {
            inner: StreamRecords::new(client),
            mode: mode.clone(),
        };
        let config = config(Role::Controller);
        let (channel, peer) = tokio::join!(
            Channel::prepare(io, description(7), &config),
            Peer::accept(StreamRecords::new(server), description(7), false)
        );
        let mut channel = channel.unwrap();
        let _peer = peer.unwrap();
        mode.store(fault, Ordering::Release);
        let result =
            tokio::time::timeout(Duration::from_millis(20), channel.write(b"request")).await;
        if fault == 1 {
            assert!(result.is_err());
        } else {
            assert!(result.unwrap().is_err());
        }
        mode.store(0, Ordering::Release);
        assert!(channel.peer().is_none());
        assert!(channel.write(b"retry").await.is_err());
    }
}

#[tokio::test(start_paused = true)]
async fn handshake_deadline_covers_all_messages_and_closes_the_owned_carrier() {
    let (client, server) = duplex(4096);
    let slow = FaultIo {
        inner: StreamRecords::new(server),
        mode: Arc::new(AtomicU8::new(3)),
    };
    let mut responder = Task(tokio::spawn(Peer::accept(slow, description(7), false)));
    let start = Instant::now();
    let outcome = Channel::prepare(
        StreamRecords::new(client),
        description(7),
        &config(Role::Controller),
    )
    .await;
    assert!(matches!(outcome, Err(Error::Timeout)));
    assert_eq!(start.elapsed(), Duration::from_secs(10));
    assert!((&mut responder.0).await.unwrap().is_err());
}
