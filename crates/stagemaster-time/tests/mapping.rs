use stagemaster_time::{Clock, Error, Exchange, Limits, Mapping};

fn clocks() -> (Clock, Clock) {
    (
        Clock::new([1; 16], 7).unwrap(),
        Clock::new([2; 16], 9).unwrap(),
    )
}
fn limits() -> Limits {
    Limits {
        max_round_trip_ns: 10_000_000,
        max_age_ns: 1_000_000_000,
        max_uncertainty_ns: 10_000_000,
        relative_drift_ppm: 0,
        timestamp_error_ns: 0,
    }
}
fn exchange() -> Exchange {
    let (source, target) = clocks();
    Exchange {
        sent: source.at(1_000_000_000),
        received_at_target: target.at(10_000_300_000),
        sent_at_target: target.at(10_000_500_000),
        received: source.at(1_001_000_000),
    }
}

#[test]
fn asymmetric_paths_and_target_processing_are_reported_as_an_interval() {
    let measurement = exchange();
    let map = Mapping::measure(measurement, limits()).unwrap();
    let window = map.convert(measurement.received).unwrap();
    assert_eq!(window.earliest().nanos, 10_000_500_000);
    assert_eq!(window.latest().nanos, 10_001_300_000);
    assert_eq!(window.midpoint().nanos, 10_000_900_000);
    assert_eq!(window.uncertainty_ns(), 400_000);
    // Actual target at receipt is 10_001_000_000: midpoint is not exact one-way compensation.
    assert_ne!(window.midpoint().nanos, 10_001_000_000);
    let later = map.convert(map.source().at(1_001_200_000)).unwrap();
    assert_eq!(later.earliest().nanos, window.earliest().nanos + 200_000);
    assert_eq!(later.latest().clock, map.target());
}

#[test]
fn provider_identity_and_boot_epoch_cannot_be_mixed() {
    assert_eq!(Clock::new([0; 16], 0), Err(Error::Identity));
    let mut input = exchange();
    input.received.clock = Clock::new([1; 16], 8).unwrap();
    assert!(matches!(
        Mapping::measure(input, limits()),
        Err(Error::Clock)
    ));
    input = exchange();
    input.sent_at_target.clock = Clock::new([2; 16], 10).unwrap();
    assert!(matches!(
        Mapping::measure(input, limits()),
        Err(Error::Clock)
    ));
    input = exchange();
    input.received_at_target.clock = input.sent.clock;
    input.sent_at_target.clock = input.sent.clock;
    assert!(matches!(
        Mapping::measure(input, limits()),
        Err(Error::Identity)
    ));
    let map = Mapping::measure(exchange(), limits()).unwrap();
    assert_eq!(map.source().epoch(), 7);
    assert_eq!(
        map.convert(map.target().at(1_001_000_000)),
        Err(Error::Clock)
    );
    assert_eq!(
        map.convert(Clock::new([1; 16], 8).unwrap().at(1_001_000_000)),
        Err(Error::Clock)
    );
}

#[test]
fn repeated_reads_do_not_extend_validity_or_erase_uncertainty() {
    let input = exchange();
    let policy = Limits {
        max_age_ns: 1000,
        ..limits()
    };
    let map = Mapping::measure(input, policy).unwrap();
    for elapsed in [0, 1, 100, 500, 999] {
        let window = map
            .convert(map.source().at(input.received.nanos + elapsed))
            .unwrap();
        assert_eq!(window.uncertainty_ns(), 400_000);
    }
    assert_eq!(
        map.convert(map.source().at(input.received.nanos + 1000)),
        Err(Error::Expired)
    );
    assert_eq!(
        map.convert(map.source().at(input.received.nanos - 1)),
        Err(Error::Backwards)
    );
}

#[test]
fn delay_quality_order_and_final_range_are_checked() {
    let mut input = exchange();
    input.sent.nanos = input.received.nanos + 1;
    assert!(matches!(
        Mapping::measure(input, limits()),
        Err(Error::Backwards)
    ));
    input = exchange();
    input.sent_at_target.nanos = input.received_at_target.nanos - 1;
    assert!(matches!(
        Mapping::measure(input, limits()),
        Err(Error::Backwards)
    ));
    input = exchange();
    input.sent_at_target.nanos += 10_000_000;
    assert!(matches!(
        Mapping::measure(input, limits()),
        Err(Error::Inconsistent)
    ));
    assert!(matches!(
        Mapping::measure(
            exchange(),
            Limits {
                max_round_trip_ns: 999_999,
                ..limits()
            }
        ),
        Err(Error::RoundTrip)
    ));
    assert!(matches!(
        Mapping::measure(
            exchange(),
            Limits {
                max_uncertainty_ns: 399_999,
                ..limits()
            }
        ),
        Err(Error::Uncertain)
    ));
    input = exchange();
    input.received_at_target.nanos = u64::MAX - 1;
    input.sent_at_target.nanos = u64::MAX;
    assert!(matches!(
        Mapping::measure(input, limits()),
        Err(Error::Overflow)
    ));
    assert!(matches!(
        Mapping::measure(
            exchange(),
            Limits {
                relative_drift_ppm: 1_000_000,
                ..limits()
            }
        ),
        Err(Error::Limits)
    ));
}

#[test]
fn fractional_drift_rounds_outwards_and_may_invalidate_an_old_mapping() {
    let (source, target) = clocks();
    let input = Exchange {
        sent: source.at(0),
        received: source.at(0),
        received_at_target: target.at(100),
        sent_at_target: target.at(100),
    };
    let map = Mapping::measure(
        input,
        Limits {
            relative_drift_ppm: 1,
            max_uncertainty_ns: 1,
            ..limits()
        },
    )
    .unwrap();
    let window = map.convert(source.at(1)).unwrap();
    assert_eq!((window.earliest().nanos, window.latest().nanos), (100, 102));
    assert_eq!(map.convert(source.at(1_000_001)), Err(Error::Uncertain));
}

#[test]
fn precision_is_part_of_delay_freshness_and_range_not_a_hidden_zero() {
    assert!(matches!(
        Mapping::measure(
            exchange(),
            Limits {
                timestamp_error_ns: 1,
                max_round_trip_ns: 1_000_000,
                ..limits()
            }
        ),
        Err(Error::RoundTrip)
    ));
    assert!(matches!(
        Mapping::measure(
            exchange(),
            Limits {
                timestamp_error_ns: 5,
                max_age_ns: 10,
                ..limits()
            }
        ),
        Err(Error::Expired)
    ));
    let base = Mapping::measure(exchange(), limits()).unwrap();
    let measured = Mapping::measure(
        exchange(),
        Limits {
            timestamp_error_ns: 5,
            ..limits()
        },
    )
    .unwrap();
    let before = base.convert(exchange().received).unwrap();
    let after = measured.convert(exchange().received).unwrap();
    assert!(after.earliest().nanos < before.earliest().nanos);
    assert!(after.latest().nanos > before.latest().nanos);
}
