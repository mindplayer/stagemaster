mod support;
use stagemaster_device_auth::application::Role;
use stagemaster_device_channel::{Channel, RecordIo, StreamRecords};
use stagemaster_device_session::Kind;
use std::time::Duration;
use support::{config, description, message, peer::Peer};
use tokio::io::{DuplexStream, duplex};

type Client = Channel<StreamRecords<DuplexStream>>;
type Device = Peer<StreamRecords<DuplexStream>>;
#[tokio::test]
async fn wrong_trusted_device_key_fails_without_plaintext_fallback() {
    let (client, server) = duplex(4096);
    let controller = support::config_with_trusted(Role::Controller, 6);
    let (channel, peer) = tokio::join!(
        Channel::prepare(StreamRecords::new(client), description(7), &controller),
        Peer::accept(StreamRecords::new(server), description(7), false)
    );
    assert!(channel.is_err());
    assert!(peer.is_err());
}

async fn connected(connection: u64) -> (Client, Device) {
    let (client, server) = duplex(4096);
    let controller = config(Role::Controller);
    let (channel, peer) = tokio::join!(
        Channel::prepare(
            StreamRecords::new(client),
            description(connection),
            &controller
        ),
        Peer::accept(StreamRecords::new(server), description(connection), false)
    );
    (channel.unwrap(), peer.unwrap())
}

#[tokio::test]
async fn stale_receipt_and_mismatched_crypto_context_do_not_create_permission() {
    for stale_receipt in [false, true] {
        let (client, server) = duplex(4096);
        let controller = config(Role::Controller);
        let mut desc = description(7);
        if !stale_receipt {
            desc.boot = [3; 16];
        }
        let (channel, peer) = tokio::join!(
            Channel::prepare(StreamRecords::new(client), description(7), &controller),
            Peer::accept(StreamRecords::new(server), desc, stale_receipt)
        );
        assert!(channel.is_err());
        if stale_receipt {
            assert!(peer.is_ok());
        } else {
            assert!(peer.is_err());
        }
    }
}

#[tokio::test]
async fn tampering_and_replayed_ciphertext_permanently_revoke_the_channel() {
    for tamper in [false, true] {
        let (mut client, mut device) = connected(7).await;
        let mut cipher = device.seal(Kind::Message, b"confirmed").unwrap();
        if tamper {
            cipher[0] ^= 1;
        } else {
            device.io.send(&cipher).await.unwrap();
            assert_eq!(message(&mut client).await.unwrap(), b"confirmed");
        }
        device.io.send(&cipher).await.unwrap();
        assert!(message(&mut client).await.is_err());
        assert!(client.peer().is_none());
        assert!(client.write(b"retry").await.is_err());
    }
}

#[tokio::test]
async fn reconnect_has_new_session_and_rejects_old_ciphertext() {
    let (old, mut old_device) = connected(7).await;
    let session = old.peer().unwrap().session;
    let cipher = old_device.seal(Kind::Message, b"old").unwrap();
    drop(old);
    drop(old_device);
    let (mut fresh, mut device) = connected(8).await;
    assert_ne!(fresh.peer().unwrap().session, session);
    device.io.send(&cipher).await.unwrap();
    assert!(message(&mut fresh).await.is_err());
}

#[tokio::test]
async fn cancelled_heartbeat_cannot_reuse_a_late_reply() {
    let (mut client, mut device) = connected(7).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(20), client.heartbeat())
            .await
            .is_err()
    );
    assert!(client.peer().is_none());
    assert_eq!(device.receive().await.unwrap().0, Kind::Heartbeat);
    device.send(Kind::HeartbeatReply, &[]).await.unwrap();
    assert!(client.receive().is_err());
    assert!(client.heartbeat().await.is_err());
}

#[tokio::test(start_paused = true)]
async fn idle_timeout_and_fixed_permission_do_not_renew_from_queries_or_keepalive() {
    let (mut idle, _device) = connected(7).await;
    for _ in 0..6 {
        tokio::time::advance(Duration::from_secs(1)).await;
        let _ = idle.peer();
    }
    assert!(idle.peer().is_none());
    assert!(idle.receive().is_err());
    let (mut active, mut device) = connected(8).await;
    for _ in 0..29 {
        tokio::time::advance(Duration::from_secs(2)).await;
        let (reply, ()) = tokio::join!(active.heartbeat(), async {
            assert_eq!(device.receive().await.unwrap().0, Kind::Heartbeat);
            device.send(Kind::HeartbeatReply, &[]).await.unwrap();
        });
        reply.unwrap();
        assert!(active.peer().is_some());
    }
    tokio::time::advance(Duration::from_secs(2)).await;
    assert!(active.peer().is_none());
    assert!(active.write(b"expired").await.is_err());
}
