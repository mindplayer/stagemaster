use super::*;

#[test]
fn empty_disconnected_bad_order_and_decoder_error_are_distinguished() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let failed = Arc::new(AtomicBool::new(false));
    let mut feed = Feed {
        receiver: Some(receiver),
        current: None,
        failed: failed.clone(),
        cancelled: Arc::new(AtomicBool::new(false)),
    };
    assert!(matches!(feed.frame(0, 1), Err(Failure::Underflow)));
    sender
        .send(Block {
            start: 10,
            frames: 1,
            samples: [0.25; BLOCK_FRAMES * 2],
        })
        .unwrap();
    assert!(matches!(feed.frame(0, 1), Err(Failure::Sequence)));
    assert_eq!(
        feed.frame(10, 2).unwrap().map(f32::to_bits),
        [0.25_f32.to_bits(); 2]
    );
    drop(sender);
    assert!(matches!(feed.frame(11, 1), Err(Failure::Decode)));
    failed.store(true, Ordering::Release);
    assert!(matches!(feed.frame(11, 1), Err(Failure::Decode)));
    feed.cancel();
    assert!(feed.cancelled.load(Ordering::Acquire));
}

#[test]
fn full_fixed_queue_refuses_an_extra_block() {
    let (sender, _receiver) = mpsc::sync_channel(QUEUE_BLOCKS);
    for start in 0..QUEUE_BLOCKS {
        sender
            .try_send(Block {
                start: start as u64,
                frames: 1,
                samples: [0.0; BLOCK_FRAMES * 2],
            })
            .unwrap();
    }
    assert!(matches!(
        sender.try_send(Block {
            start: 999,
            frames: 1,
            samples: [0.0; BLOCK_FRAMES * 2]
        }),
        Err(mpsc::TrySendError::Full(_))
    ));
    assert_eq!(
        QUEUE_BLOCKS * BLOCK_FRAMES * 2 * size_of::<f32>(),
        1_024 * 1_024
    );
}
