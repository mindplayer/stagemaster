use stagemaster_device_link::secure::{MAX_RECORD_BYTES, Receiver, Sender};

#[test]
fn golden_packets_keep_serials_across_records_and_wait_for_send_confirmation() {
    let mut sender = Sender::new(20, 0).unwrap();
    let mut receiver = Receiver::new(20, 0).unwrap();
    sender.queue(&[0xa5; 17], 1).unwrap();
    let first = sender.fragment(2).unwrap().unwrap();
    assert_eq!(
        first.bytes(),
        &[
            1, 0, 0, 0, 1, 0, 0, 17, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5,
            0xa5, 0xa5
        ]
    );
    assert_eq!(sender.fragment(3).unwrap().unwrap().bytes(), first.bytes());
    assert!(receiver.push(first.bytes(), 3).unwrap().is_none());
    assert!(!sender.sent(3).unwrap());
    let last = sender.fragment(4).unwrap().unwrap();
    assert_eq!(last.bytes(), &[2, 0, 0, 0, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5]);
    assert_eq!(
        receiver.push(last.bytes(), 4).unwrap().unwrap().bytes(),
        &[0xa5; 17]
    );
    assert!(sender.sent(4).unwrap());
    assert!(sender.fragment(5).unwrap().is_none());
    sender.queue(&[0x77], 500_000).unwrap();
    let next = sender.fragment(500_001).unwrap().unwrap();
    assert_eq!(next.bytes(), &[3, 0, 0, 0, 1, 0, 0, 1, 0x77]);
    assert_eq!(
        receiver
            .push(next.bytes(), 500_001)
            .unwrap()
            .unwrap()
            .bytes(),
        &[0x77]
    );
    assert!(sender.sent(500_001).unwrap());
}

#[test]
fn every_bounded_length_survives_small_and_large_packet_budgets() {
    for budget in [20, 21, 244] {
        let mut sender = Sender::new(budget, 0).unwrap();
        let mut receiver = Receiver::new(budget, 0).unwrap();
        let mut serial = 1u32;
        for length in 1..=MAX_RECORD_BYTES {
            let payload: Vec<u8> = (0..=255).cycle().take(length).collect();
            sender.queue(&payload, 1).unwrap();
            let mut complete = None;
            while let Some(packet) = sender.fragment(1).unwrap() {
                assert!(packet.bytes().len() <= budget);
                assert_eq!(&packet.bytes()[..4], &serial.to_le_bytes());
                serial += 1;
                let result = receiver.push(packet.bytes(), 1).unwrap();
                assert!(complete.is_none());
                let done = sender.sent(1).unwrap();
                assert_eq!(done, result.is_some());
                complete = result;
            }
            assert_eq!(complete.unwrap().bytes(), payload);
        }
    }
}

#[test]
fn header_can_be_split_but_incomplete_data_is_never_exposed() {
    let mut receiver = Receiver::new(20, 0).unwrap();
    for packet in [&[1, 0, 0, 0, 1][..], &[2, 0, 0, 0, 0, 0], &[3, 0, 0, 0, 1]] {
        assert!(receiver.push(packet, 1).unwrap().is_none());
    }
    assert_eq!(
        receiver
            .push(&[4, 0, 0, 0, 0x77], 2)
            .unwrap()
            .unwrap()
            .bytes(),
        &[0x77]
    );
}
