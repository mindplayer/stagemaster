use super::*;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

const UUID: Uuid = Uuid::from_u128(1);
fn channel(limit: u16) -> (UnboundedSender<ValueNotification>, Receiver) {
    let (tx, rx) = unbounded_channel();
    let stream =
        futures_util::stream::unfold(rx, |mut rx| async { rx.recv().await.map(|v| (v, rx)) })
            .boxed();
    (tx, Receiver::spawn(stream, UUID, limit))
}
fn send(tx: &UnboundedSender<ValueNotification>, serial: u32, body: &[u8]) {
    let mut value = serial.to_le_bytes().to_vec();
    value.extend_from_slice(body);
    tx.send(ValueNotification {
        uuid: UUID,
        service_uuid: super::super::installation::SERVICE,
        value,
    })
    .unwrap();
}
async fn until(mut done: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while !done() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn ordered_notifications_are_bounded_and_connection_local() {
    let (tx, mut rx) = channel(16);
    tx.send(ValueNotification {
        uuid: Uuid::nil(),
        service_uuid: super::super::installation::SERVICE,
        value: vec![99],
    })
    .unwrap();
    send(&tx, 1, &[7; 16]);
    send(&tx, 2, &[8]);
    until(|| rx.queue.len() == 2).await;
    assert_eq!(rx.receive().unwrap(), Some(vec![7; 16]));
    assert_eq!(rx.receive().unwrap(), Some(vec![8]));
    assert_eq!(rx.receive().unwrap(), None);
    drop(rx);
    until(|| tx.is_closed()).await;
    let (next, mut fresh) = channel(16);
    send(&next, 1, &[9]);
    until(|| fresh.queue.len() == 1).await;
    assert_eq!(fresh.receive().unwrap(), Some(vec![9]));
}

#[tokio::test]
async fn gaps_duplicates_oversize_and_stream_end_revoke_even_queued_data() {
    for (serial, length) in [(3, 1), (1, 1), (2, 17)] {
        let (tx, mut rx) = channel(16);
        send(&tx, 1, &[7]);
        send(&tx, serial, &vec![8; length]);
        until(|| !rx.healthy()).await;
        assert!(rx.receive().is_err());
    }
    let (tx, mut rx) = channel(16);
    send(&tx, 1, &[7]);
    drop(tx);
    until(|| !rx.healthy()).await;
    assert!(rx.receive().is_err());
}

#[tokio::test]
async fn overflow_fails_closed_without_silently_truncating_a_response() {
    let (tx, mut rx) = channel(240);
    for serial in 1..=65 {
        send(&tx, serial, &[7; 240]);
    }
    until(|| !rx.healthy()).await;
    assert_eq!(rx.queue.len(), CAPACITY);
    assert!(rx.receive().is_err());
}
