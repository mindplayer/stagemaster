//! Hand-calculated assertions run on the MCU. Test plans never enter a user project.
use alloc::vec;
use stagemaster_playback::{Plan, Player, Status, Step};

pub fn verify() {
    let plan = Plan::new(
        vec![0, 10_000],
        vec![
            Step {
                target: vec![60_000, 30_000],
                delay_ms: 500,
                fade_ms: 1000,
                wait_ms: Some(500),
            },
            Step {
                target: vec![0, 50_000],
                delay_ms: 0,
                fade_ms: 1000,
                wait_ms: None,
            },
        ],
        false,
    )
    .unwrap();
    let mut player = Player::new(plan, 0);
    player.execute(0, 0).unwrap();
    player.advance(499).unwrap();
    assert_eq!(player.values(), &[0, 10_000]);
    player.advance(1000).unwrap();
    assert_eq!(player.values(), &[30_000, 20_000]);
    player.pause(1000).unwrap();
    player.advance(9000).unwrap();
    assert_eq!(player.values(), &[30_000, 20_000]);
    assert_eq!(player.status(), Status::Paused);
    player.resume(9000).unwrap();
    player.advance(9500).unwrap();
    assert_eq!(player.values(), &[60_000, 30_000]);
    player.advance(10_500).unwrap();
    assert_eq!(player.index(), Some(1));
    assert_eq!(player.values(), &[30_000, 40_000]);
    player.execute(0, 10_500).unwrap();
    player.advance(11_500).unwrap();
    assert_eq!(player.values(), &[45_000, 35_000]);
    assert!(player.advance(11_499).is_err());
    assert_eq!(player.values(), &[45_000, 35_000]);
    player.stop(11_500).unwrap();
    assert_eq!(player.values(), &[0, 10_000]);
    assert_eq!(player.status(), Status::Idle);

    let mut looping = benchmark_player();
    looping.execute(0, 0).unwrap();
    looping.advance(1_000_001_000).unwrap();
    assert_eq!(looping.values()[0], 65_535);
    assert_eq!(looping.index(), Some(1));
}

pub fn benchmark_player() -> Player {
    Player::new(
        Plan::new(
            vec![0; 512],
            vec![
                Step {
                    target: vec![65_535; 512],
                    delay_ms: 0,
                    fade_ms: 1000,
                    wait_ms: Some(0),
                },
                Step {
                    target: vec![0; 512],
                    delay_ms: 0,
                    fade_ms: 1000,
                    wait_ms: Some(0),
                },
            ],
            true,
        )
        .unwrap(),
        0,
    )
}
