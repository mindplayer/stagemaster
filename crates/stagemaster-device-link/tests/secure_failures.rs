use stagemaster_device_link::secure::{Error, MAX_RECORD_BYTES, RECORD_MS, Receiver, Sender};

fn first_two() -> (Vec<u8>, Vec<u8>) {
    let mut sender = Sender::new(20, 0).unwrap();
    sender.queue(&[0x5a; 17], 0).unwrap();
    let first = sender.fragment(0).unwrap().unwrap().bytes().to_vec();
    sender.sent(0).unwrap();
    let second = sender.fragment(0).unwrap().unwrap().bytes().to_vec();
    (first, second)
}

#[test]
fn loss_duplicates_reordering_and_restart_are_terminal() {
    let (first, second) = first_two();
    for packets in [
        vec![second.clone()],
        vec![first.clone(), first.clone()],
        vec![first.clone(), second.clone(), first.clone()],
    ] {
        let mut receiver = Receiver::new(20, 0).unwrap();
        let (last, prefix) = packets.split_last().unwrap();
        for packet in prefix {
            receiver.push(packet, 0).unwrap();
        }
        assert!(matches!(receiver.push(last, 0), Err(Error::Fragment(_))));
        assert_eq!(receiver.poll(0), Err(Error::Closed));
        assert!(matches!(receiver.push(&first, 0), Err(Error::Closed)));
    }
}

#[test]
fn malformed_headers_overflow_and_coalesced_records_are_terminal() {
    for bytes in [
        vec![],
        vec![1, 0, 0, 0],
        vec![1, 0, 0, 0, 2, 0, 0, 1, 1],
        vec![1, 0, 0, 0, 1, 1, 0, 1, 1],
        vec![1, 0, 0, 0, 1, 0, 0, 0],
        vec![1, 0, 0, 0, 1, 0, 5, 18],
        vec![1, 0, 0, 0, 1, 0, 0, 1, 9, 1, 0, 0, 1, 8],
        vec![0; 21],
    ] {
        let mut receiver = Receiver::new(20, 0).unwrap();
        assert!(receiver.push(&bytes, 0).is_err());
        assert_eq!(receiver.poll(0), Err(Error::Closed));
    }
    // A claimed maximum record still cannot overrun the fixed storage on its final fragment.
    let mut sender = Sender::new(244, 0).unwrap();
    let mut receiver = Receiver::new(244, 0).unwrap();
    sender.queue(&[0x5a; MAX_RECORD_BYTES], 0).unwrap();
    loop {
        let mut packet = sender.fragment(0).unwrap().unwrap().bytes().to_vec();
        let last = sender.sent(0).unwrap();
        if last {
            packet.push(1);
            assert!(matches!(receiver.push(&packet, 0), Err(Error::Bounds)));
            break;
        }
        assert!(receiver.push(&packet, 0).unwrap().is_none());
    }
}

#[test]
fn fixed_deadlines_are_not_extended_by_fragments_or_polling() {
    let (first, second) = first_two();
    let mut receiver = Receiver::new(20, 0).unwrap();
    receiver.push(&first, 10).unwrap();
    receiver.poll(10 + RECORD_MS - 1).unwrap();
    assert!(matches!(
        receiver.push(&second, 10 + RECORD_MS),
        Err(Error::Expired)
    ));
    assert_eq!(receiver.poll(10 + RECORD_MS), Err(Error::Closed));

    let mut sender = Sender::new(20, 0).unwrap();
    sender.queue(&[0x5a; 17], 10).unwrap();
    sender.fragment(10 + RECORD_MS - 2).unwrap();
    sender.sent(10 + RECORD_MS - 1).unwrap();
    assert!(matches!(
        sender.fragment(10 + RECORD_MS),
        Err(Error::Expired)
    ));
    assert_eq!(sender.poll(10 + RECORD_MS), Err(Error::Closed));

    let mut receiver = Receiver::new(20, 0).unwrap();
    receiver.push(&[1, 0, 0, 0, 1], 1).unwrap();
    receiver.push(&[2, 0, 0, 0, 0], 4_999).unwrap();
    assert_eq!(receiver.poll(5_001), Err(Error::Expired));
}

#[test]
fn cancellation_clock_rollback_and_overflow_are_terminal() {
    let (first, _) = first_two();
    let mut receiver = Receiver::new(20, 1).unwrap();
    assert!(matches!(receiver.push(&first, 0), Err(Error::Clock)));
    assert_eq!(receiver.poll(1), Err(Error::Closed));
    let mut receiver = Receiver::new(20, u64::MAX).unwrap();
    assert!(matches!(receiver.push(&first, u64::MAX), Err(Error::Clock)));
    let mut sender = Sender::new(20, 0).unwrap();
    assert_eq!(sender.queue(&[1], u64::MAX), Err(Error::Clock));
    let mut sender = Sender::new(20, 1).unwrap();
    assert_eq!(sender.queue(&[1], 0), Err(Error::Clock));
    assert_eq!(sender.poll(1), Err(Error::Closed));
    for partial in [false, true] {
        let mut sender = Sender::new(20, 0).unwrap();
        let mut receiver = Receiver::new(20, 0).unwrap();
        if partial {
            sender.queue(&[0; 100], 0).unwrap();
            receiver
                .push(sender.fragment(0).unwrap().unwrap().bytes(), 0)
                .unwrap();
        }
        sender.close();
        receiver.close();
        assert!(matches!(sender.fragment(1), Err(Error::Closed)));
        assert!(matches!(receiver.push(&first, 1), Err(Error::Closed)));
    }
}

#[test]
fn invalid_budgets_payloads_and_send_order_cannot_be_retried() {
    for size in [0, 19, 245, usize::MAX] {
        assert!(matches!(Sender::new(size, 0), Err(Error::Bounds)));
        assert!(matches!(Receiver::new(size, 0), Err(Error::Bounds)));
    }
    for payload in [vec![], vec![0; MAX_RECORD_BYTES + 1]] {
        let mut sender = Sender::new(20, 0).unwrap();
        assert_eq!(sender.queue(&payload, 0), Err(Error::Bounds));
        assert_eq!(sender.queue(&[1], 0), Err(Error::Closed));
    }
    let mut sender = Sender::new(20, 0).unwrap();
    assert_eq!(sender.sent(0), Err(Error::State));
    let mut sender = Sender::new(20, 0).unwrap();
    sender.queue(&[1], 0).unwrap();
    assert_eq!(sender.sent(0), Err(Error::State));
    let mut sender = Sender::new(20, 0).unwrap();
    sender.queue(&[1], 0).unwrap();
    sender.fragment(0).unwrap();
    assert_eq!(sender.queue(&[2], 0), Err(Error::State));
    assert_eq!(sender.sent(0), Err(Error::Closed));
    let mut sender = Sender::new(20, 0).unwrap();
    sender.queue(&[0; 100], 0).unwrap();
    sender.fragment(0).unwrap();
    sender.sent(0).unwrap();
    assert_eq!(sender.sent(0), Err(Error::State));
}
