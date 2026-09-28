use super::*;
use stagemaster_device_link::secure::Sender;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
fn channel() -> (UnboundedSender<ValueNotification>, Incoming) {
    let (tx, rx) = unbounded_channel();
    let stream =
        futures_util::stream::unfold(rx, |mut rx| async { rx.recv().await.map(|n| (n, rx)) })
            .boxed();
    (tx, Incoming::spawn(stream, Instant::now(), 20))
}
fn notification(value: Vec<u8>) -> ValueNotification {
    ValueNotification {
        uuid: RESPONSE,
        service_uuid: SERVICE,
        value,
    }
}
fn packets(sender: &mut Sender, bytes: &[u8]) -> Vec<ValueNotification> {
    sender.queue(bytes, 0).unwrap();
    let mut out = Vec::new();
    while let Some(packet) = sender.fragment(0).unwrap() {
        out.push(notification(packet.bytes().to_vec()));
        sender.sent(0).unwrap();
    }
    out
}
async fn until(mut predicate: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while !predicate() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}
#[tokio::test]
async fn records_remain_ordered_complete_and_connection_local() {
    let (tx, mut rx) = channel();
    let mut sender = Sender::new(20, 0).unwrap();
    let mut other = notification(vec![255]);
    other.service_uuid = uuid::Uuid::nil();
    tx.send(other).unwrap();
    for byte in [7, 8] {
        for packet in packets(&mut sender, &[byte; 1297]) {
            tx.send(packet).unwrap();
        }
    }
    until(|| rx.queue.len() == 2).await;
    assert_eq!(rx.next().unwrap().unwrap().bytes(), [7; 1297]);
    assert_eq!(rx.next().unwrap().unwrap().bytes(), [8; 1297]);
    assert!(rx.next().unwrap().is_none());
    drop(rx);
    until(|| tx.is_closed()).await;
    let (tx, mut rx) = channel();
    for packet in packets(&mut Sender::new(20, 0).unwrap(), b"fresh") {
        tx.send(packet).unwrap();
    }
    assert_eq!(rx.receive().await.unwrap().bytes(), b"fresh");
}
#[tokio::test]
async fn corruption_gap_repeat_and_stream_end_revoke_queued_records() {
    for fault in 0..4 {
        let (tx, mut rx) = channel();
        let mut sender = Sender::new(20, 0).unwrap();
        for packet in packets(&mut sender, b"valid") {
            tx.send(packet).unwrap();
        }
        let mut next = packets(&mut sender, b"next").remove(0);
        match fault {
            0 => next.value[0] += 1,
            1 => next.value[0] -= 1,
            2 => next.value.resize(21, 0),
            _ => (),
        }
        if fault == 3 {
            drop(tx);
        } else {
            tx.send(next).unwrap();
        }
        until(|| !rx.healthy()).await;
        assert!(rx.next().is_err());
    }
}
#[tokio::test]
async fn full_queue_fails_without_silently_losing_a_record() {
    let (tx, mut rx) = channel();
    let mut sender = Sender::new(20, 0).unwrap();
    for byte in 1..=5 {
        for packet in packets(&mut sender, &[byte]) {
            tx.send(packet).unwrap();
        }
    }
    until(|| !rx.healthy()).await;
    assert_eq!(rx.queue.len(), 4);
    assert!(rx.next().is_err());
}
#[tokio::test(start_paused = true)]
async fn incomplete_record_expires_without_waiting_for_more_notifications() {
    let (tx, mut rx) = channel();
    let mut sender = Sender::new(20, 0).unwrap();
    tx.send(packets(&mut sender, &[7; 100]).remove(0)).unwrap();
    tokio::time::sleep(Duration::from_millis(1)).await;
    assert!(rx.healthy());
    tokio::time::advance(Duration::from_millis(5100)).await;
    until(|| !rx.healthy()).await;
    assert!(rx.next().is_err());
}
