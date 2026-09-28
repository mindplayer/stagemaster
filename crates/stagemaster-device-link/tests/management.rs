use stagemaster_device_link::management::{Error, Receipt, Serial};

#[test]
fn receipt_strict_profile_and_connection_correlation() {
    let receipt = Receipt {
        device: [1; 16],
        boot: [2; 16],
        diagnostic: 3,
        session: [4; 16],
        fragment_bytes: 240,
    };
    let bytes = receipt.encode().unwrap();
    assert_eq!(Receipt::decode(&bytes), Ok(receipt));
    assert_eq!(receipt.correlate([1; 16], [2; 16], 3), Ok(()));
    for (device, boot, diagnostic) in [
        ([9; 16], [2; 16], 3),
        ([1; 16], [9; 16], 3),
        ([1; 16], [2; 16], 9),
    ] {
        assert_eq!(
            receipt.correlate(device, boot, diagnostic),
            Err(Error::Identity)
        );
    }
    for i in (0..8).chain(64..68).chain(70..80) {
        let mut corrupt = bytes;
        corrupt[i] ^= 1;
        assert!(Receipt::decode(&corrupt).is_err(), "offset {i}");
    }
    for range in [8..24, 24..40, 40..48, 48..64] {
        let mut corrupt = bytes;
        corrupt[range].fill(0);
        assert_eq!(Receipt::decode(&corrupt), Err(Error::Identity));
    }
    for fragment_bytes in [0, 15, 241, u16::MAX] {
        assert_eq!(
            Receipt {
                fragment_bytes,
                ..receipt
            }
            .validate(),
            Err(Error::Limits)
        );
    }
    for n in 0..80 {
        assert!(Receipt::decode(&bytes[..n]).is_err());
    }
    let mut extra = bytes.to_vec();
    extra.push(0);
    assert!(Receipt::decode(&extra).is_err());
}

#[test]
fn lost_duplicate_truncated_and_unacknowledged_packets_never_advance_silently() {
    let mut tx = Serial::default();
    let mut rx = Serial::default();
    for len in [1, 16, 240, 5] {
        let body = vec![0x5a; len];
        let packet = tx.encode(&body).unwrap();
        assert_eq!(packet.bytes(), tx.encode(&body).unwrap().bytes());
        assert_eq!(rx.receive(packet.bytes()).unwrap(), body);
        assert_eq!(rx.receive(packet.bytes()), Err(Error::Sequence));
        tx.advance().unwrap();
    }
    let dropped = tx.encode(&[1]).unwrap();
    tx.advance().unwrap();
    let later = tx.encode(&[2]).unwrap();
    assert_eq!(rx.receive(later.bytes()), Err(Error::Sequence));
    assert_eq!(rx.receive(dropped.bytes()).unwrap(), [1]);
    for n in 0..5 {
        assert!(rx.receive(&later.bytes()[..n]).is_err());
    }
    assert!(tx.encode(&[]).is_err());
    assert!(tx.encode(&[0; 241]).is_err());
}
