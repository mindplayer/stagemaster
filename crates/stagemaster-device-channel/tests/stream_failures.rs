use stagemaster_device_channel::{RecordIo, StreamRecords, receive};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};

async fn settle() {
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
}

#[tokio::test]
async fn segmented_and_coalesced_records_preserve_exact_boundaries() {
    let (stream, mut remote) = duplex(4096);
    let mut records = StreamRecords::new(stream);
    remote.write_all(&[0]).await.unwrap();
    settle().await;
    assert!(records.try_receive().unwrap().is_none());
    remote.write_all(&[3, 1]).await.unwrap();
    settle().await;
    assert!(records.try_receive().unwrap().is_none());
    remote.write_all(&[2, 3, 0, 2, 7, 8]).await.unwrap();
    assert_eq!(receive(&mut records).await.unwrap(), [1, 2, 3]);
    assert_eq!(receive(&mut records).await.unwrap(), [7, 8]);
    records.send(&[9; 1297]).await.unwrap();
    let mut wire = [0; 1299];
    remote.read_exact(&mut wire).await.unwrap();
    assert_eq!(&wire[..2], &1297_u16.to_be_bytes());
    assert_eq!(&wire[2..], &[9; 1297]);
}

#[tokio::test]
async fn invalid_lengths_disconnect_and_overflow_revoke_all_queued_records() {
    for bytes in [
        vec![0, 0],
        vec![5, 18],
        vec![255, 255],
        vec![0, 1, 42, 0, 0],
        [0, 1, 42].repeat(5),
    ] {
        let (stream, mut remote) = duplex(4096);
        let mut records = StreamRecords::new(stream);
        remote.write_all(&bytes).await.unwrap();
        settle().await;
        assert!(!records.healthy());
        assert!(records.try_receive().is_err());
        assert!(records.send(b"new").await.is_err());
    }
    let (stream, mut remote) = duplex(4096);
    let mut records = StreamRecords::new(stream);
    remote.write_all(&[0, 9, 1, 2]).await.unwrap();
    drop(remote);
    settle().await;
    assert!(records.try_receive().is_err());
}

#[tokio::test(start_paused = true)]
async fn partial_record_deadline_is_absolute_and_no_input_does_not_reset_it() {
    let (stream, mut remote) = duplex(4096);
    let mut records = StreamRecords::new(stream);
    remote.write_all(&[0]).await.unwrap();
    settle().await;
    tokio::time::advance(Duration::from_secs(4)).await;
    remote.write_all(&[3, 1]).await.unwrap();
    settle().await;
    assert!(records.healthy());
    tokio::time::advance(Duration::from_secs(1)).await;
    settle().await;
    assert!(!records.healthy());
    assert!(records.try_receive().is_err());
}

#[tokio::test]
async fn cancelled_partial_write_is_terminal_and_drop_releases_the_stream() {
    let (stream, mut remote) = duplex(1);
    let mut records = StreamRecords::new(stream);
    assert!(
        tokio::time::timeout(Duration::from_millis(20), records.send(&[4; 1297]))
            .await
            .is_err()
    );
    assert!(!records.healthy());
    assert!(records.send(b"next").await.is_err());
    let mut bytes = Vec::new();
    tokio::time::timeout(Duration::from_secs(1), remote.read_to_end(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(bytes.len(), 1);
    let (stream, mut remote) = duplex(1);
    drop(StreamRecords::new(stream));
    let mut byte = [0];
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(1), remote.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
}

#[tokio::test(start_paused = true)]
async fn write_timeout_and_oversized_payload_cannot_be_retried() {
    let (stream, _remote) = duplex(1);
    let mut records = StreamRecords::new(stream);
    assert!(records.send(&[4; 1297]).await.is_err());
    assert!(!records.healthy());
    let (stream, _remote) = duplex(4096);
    let mut records = StreamRecords::new(stream);
    assert!(records.send(&[4; 1298]).await.is_err());
    assert!(!records.healthy());
}
