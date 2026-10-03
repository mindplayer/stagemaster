use stagemaster_time::{Clock, Exchange, Limits, Mapping};

// Independent forward model: two oscillator rates, asymmetric travel and processing,
// quantized timestamps with every combination of capture errors. No inverse formula here.
#[test]
fn mapped_interval_contains_the_true_target_under_declared_rate_and_capture_bounds() {
    let source = Clock::new([3; 16], 1).unwrap();
    let target = Clock::new([4; 16], 2).unwrap();
    let limits = Limits {
        max_round_trip_ns: 100_000,
        max_age_ns: 1_000_000,
        max_uncertainty_ns: 100_000,
        relative_drift_ppm: 100,
        timestamp_error_ns: 3,
    };
    let source_start = 1_000_000_i128;
    let shift =
        |time: i128, sign: u32| u64::try_from(time + if sign == 0 { -2 } else { 2 }).unwrap();
    for rate in [999_900_i128, 1_000_000, 1_000_100] {
        let actual_target = |time: i128| 10_000_000 + time * rate / 1_000_000;
        for (outward, process, returning) in [(10, 20, 800), (800, 50, 10), (1, 900, 1), (0, 0, 0)]
        {
            let target_receive_source_time = source_start + outward;
            let target_send_source_time = target_receive_source_time + process;
            let source_receive = target_send_source_time + returning;
            for flags in 0..32_u32 {
                let input = Exchange {
                    sent: source.at(shift(source_start, flags & 1)),
                    received_at_target: target.at(shift(
                        actual_target(target_receive_source_time),
                        (flags >> 1) & 1,
                    )),
                    sent_at_target: target.at(shift(
                        actual_target(target_send_source_time),
                        (flags >> 2) & 1,
                    )),
                    received: source.at(shift(source_receive, (flags >> 3) & 1)),
                };
                // Numerically reversed measured stamps are rejected, even if errors could explain them.
                if input.sent.nanos > input.received.nanos
                    || input.received_at_target.nanos > input.sent_at_target.nanos
                {
                    continue;
                }
                let map = Mapping::measure(input, limits).unwrap();
                for advance in [10, 1000, 100_000, 900_000] {
                    let true_source = source_receive + advance;
                    let query = source.at(shift(true_source, (flags >> 4) & 1));
                    let window = map.convert(query).unwrap();
                    let truth = u64::try_from(actual_target(true_source)).unwrap();
                    assert!(
                        window.earliest().nanos <= truth && truth <= window.latest().nanos,
                        "rate={rate}, flags={flags}, query={query:?}, window={window:?}, truth={truth}"
                    );
                }
            }
        }
    }
}
