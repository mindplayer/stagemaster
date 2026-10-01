use stagemaster_playback::RateClock;

#[test]
fn fractional_ticks_and_sparse_polls_are_identical() {
    for rate in [25, 33, 50, 99, 100, 125, 200, 333, 400] {
        let mut sparse = RateClock::new(0);
        sparse.set_rate(0, rate).unwrap();
        let mut dense = sparse;
        for ms in 1..=10_001 {
            dense.advance(ms).unwrap();
        }
        sparse.advance(10_001).unwrap();
        assert_eq!(dense, sparse);
        assert_eq!(
            dense.advance(10_001).unwrap(),
            10_001 * u64::from(rate) / 100
        );
    }
}
#[test]
fn rate_changes_preserve_phase_and_fraction() {
    let mut clock = RateClock::new(100);
    assert_eq!(clock.set_rate(200, 25).unwrap(), 200);
    assert_eq!(clock.advance(203).unwrap(), 200); // .75 retained
    assert_eq!(clock.set_rate(203, 125).unwrap(), 200);
    assert_eq!(clock.advance(204).unwrap(), 202);
    assert_eq!(clock.set_rate(244, 400).unwrap(), 252);
    assert_eq!(clock.advance(254).unwrap(), 292);
}
#[test]
fn invalid_rate_backwards_and_overflow_are_atomic() {
    let mut clock = RateClock::new(100);
    for rate in [0, 24, 401, u16::MAX] {
        let before = clock;
        assert!(clock.set_rate(200, rate).is_err());
        assert_eq!(clock, before);
    }
    clock.set_rate(100, 25).unwrap();
    clock.advance(101).unwrap();
    let before = clock;
    assert!(clock.advance(100).is_err()); // Same rounded logical ms still rejects.
    assert_eq!(clock, before);
    assert!(clock.set_rate(100, 100).is_err());
    assert_eq!(clock, before);
    clock.set_rate(101, 400).unwrap();
    let before = clock;
    assert!(clock.advance(u64::MAX).is_err());
    assert_eq!(clock, before);
}
