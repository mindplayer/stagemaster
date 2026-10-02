use stagemaster_playback::{
    LoopPlayback, LoopPlays, LoopPosition, LoopRegion, LoopSchedule, MAX_LOOP_REGIONS,
};

fn region(start: u64, end: u64, plays: LoopPlays) -> LoopRegion {
    LoopRegion { start, end, plays }
}

fn player(duration: u64, regions: Vec<LoopRegion>, start: u64) -> LoopPlayback {
    LoopPlayback::new(LoopSchedule::new(duration, regions).unwrap(), start).unwrap()
}

#[test]
fn validates_the_whole_schedule_before_starting() {
    use LoopPlays::{Count, UntilExit};
    assert!(LoopSchedule::new(0, vec![]).is_err());
    for regions in [
        vec![region(3, 3, Count(1))],
        vec![region(4, 3, Count(1))],
        vec![region(1, 11, Count(1))],
        vec![region(0, 1, Count(0))],
        vec![region(3, 5, Count(2)), region(2, 3, UntilExit)],
        vec![region(3, 5, Count(2)), region(4, 6, UntilExit)],
    ] {
        assert!(LoopSchedule::new(10, regions).is_err());
    }
    let regions: Vec<_> = (0..128).map(|i| region(i, i + 1, Count(1))).collect();
    assert_eq!(regions.len(), MAX_LOOP_REGIONS);
    assert!(LoopSchedule::new(129, regions.clone()).is_ok());
    let mut excess = regions;
    excess.push(region(128, 129, Count(1)));
    assert!(LoopSchedule::new(129, excess).is_err());
    assert!(LoopPlayback::new(LoopSchedule::new(10, vec![]).unwrap(), 11).is_err());
}

#[test]
fn exact_endpoints_include_adjacent_regions_but_not_gaps() {
    let mut p = player(
        10,
        vec![
            region(2, 4, LoopPlays::Count(2)),
            region(4, 6, LoopPlays::Count(1)),
            region(8, 10, LoopPlays::Count(3)),
        ],
        0,
    );
    assert_eq!(p.position().region, None);
    assert_eq!(p.advance(2).unwrap().region, Some(0));
    assert_eq!(p.advance(2).unwrap().tick, 2);
    assert_eq!(p.position().pass, Some(2));
    assert_eq!(p.advance(2).unwrap().region, Some(1));
    assert_eq!(p.position().pass, Some(1));
    assert_eq!(p.advance(2).unwrap().tick, 6);
    assert_eq!(p.position().region, None);
    assert_eq!(
        p.advance(8).unwrap(),
        LoopPosition {
            tick: 10,
            region: None,
            pass: None,
            exit_requested: false,
            ended: true,
        }
    );
    assert_eq!(p.advance(u64::MAX).unwrap(), p.position());
}

#[test]
fn boundary_exit_is_cancellable_idempotent_and_bound_to_the_current_region() {
    let mut p = player(
        10,
        vec![
            region(1, 4, LoopPlays::UntilExit),
            region(4, 8, LoopPlays::Count(5)),
        ],
        0,
    );
    assert!(p.set_exit_at_end(0, true).is_err());
    p.advance(8).unwrap(); // second tick of pass 3
    assert_eq!(p.position().tick, 2);
    assert_eq!(p.position().pass, Some(3));
    p.set_exit_at_end(0, true).unwrap();
    let requested = p.position();
    p.set_exit_at_end(0, true).unwrap();
    assert_eq!(p.position(), requested);
    assert!(p.set_exit_at_end(1, false).is_err());
    assert_eq!(p.position(), requested);
    p.set_exit_at_end(0, false).unwrap();
    assert_eq!(p.advance(2).unwrap().pass, Some(4));
    p.set_exit_at_end(0, true).unwrap();
    assert_eq!(p.advance(2).unwrap().tick, 3);
    assert_eq!(p.position().region, Some(0));
    assert_eq!(p.advance(1).unwrap().region, Some(1));
    assert!(!p.position().exit_requested);
    let next = p.position();
    assert!(p.set_exit_at_end(0, true).is_err());
    assert_eq!(p.position(), next);
    p.set_exit_at_end(1, true).unwrap();
    assert_eq!(p.advance(4).unwrap().tick, 8); // finite region also exits early
    assert_eq!(p.position().region, None);
}

#[test]
fn seek_resets_local_pass_and_exit_while_no_consumption_preserves_both() {
    let mut p = player(
        20,
        vec![
            region(1, 5, LoopPlays::UntilExit),
            region(10, 15, LoopPlays::Count(3)),
        ],
        0,
    );
    p.advance(100).unwrap();
    p.set_exit_at_end(0, true).unwrap();
    let paused = p.position();
    for _ in 0..10 {
        assert_eq!(p.advance(0).unwrap(), paused);
    }
    assert!(p.seek(21).is_err());
    assert_eq!(p.position(), paused);
    assert_eq!(p.seek(3).unwrap().pass, Some(1));
    assert!(!p.position().exit_requested);
    assert_eq!(p.seek(5).unwrap().region, None);
    assert_eq!(p.seek(12).unwrap().pass, Some(1));
    assert_eq!(p.advance(3).unwrap().tick, 10);
    assert_eq!(p.position().pass, Some(2));
    assert!(p.seek(20).unwrap().ended);
    assert_eq!(p.seek(0).unwrap().tick, 0);
}

#[test]
fn huge_advances_skip_repeats_and_numeric_failure_is_atomic() {
    let mut finite = player(3, vec![region(1, 2, LoopPlays::Count(u32::MAX))], 0);
    assert!(finite.advance(u64::MAX).unwrap().ended);
    let mut wide = player(u64::MAX, vec![region(0, u64::MAX, LoopPlays::Count(2))], 0);
    assert_eq!(wide.advance(u64::MAX).unwrap().pass, Some(2));
    assert!(wide.advance(u64::MAX).unwrap().ended);
    let mut infinite = player(1, vec![region(0, 1, LoopPlays::UntilExit)], 0);
    assert_eq!(infinite.advance(u64::MAX - 1).unwrap().pass, Some(u64::MAX));
    let before = infinite.position();
    assert!(infinite.advance(1).is_err());
    assert_eq!(infinite.position(), before);
    infinite.set_exit_at_end(0, true).unwrap();
    assert!(infinite.advance(1).unwrap().ended);
}

// Independent per-tick oracle: deliberately no quotient/modulo or multi-pass skipping.
fn reference_tick(state: &mut LoopPosition, duration: u64, regions: &[LoopRegion]) {
    if state.ended {
        return;
    }
    state.tick += 1;
    if let Some(index) = state.region {
        let r = regions[index];
        if state.tick == r.end {
            let pass = state.pass.unwrap();
            if !state.exit_requested
                && match r.plays {
                    LoopPlays::UntilExit => true,
                    LoopPlays::Count(n) => pass < u64::from(n),
                }
            {
                state.tick = r.start;
                state.pass = Some(pass + 1);
                return;
            }
            state.exit_requested = false;
            state.region = None;
            state.pass = None;
        }
    }
    if state.region.is_none() {
        state.region = regions
            .iter()
            .position(|r| (r.start..r.end).contains(&state.tick));
        state.pass = state.region.map(|_| 1);
    }
    state.ended = state.tick == duration;
}

#[test]
fn sparse_advances_match_tick_by_tick_oracle_across_mixed_schedules() {
    let options = [
        LoopPlays::Count(1),
        LoopPlays::Count(2),
        LoopPlays::Count(5),
        LoopPlays::UntilExit,
    ];
    for first in options {
        for second in options {
            for third in options {
                let regions = vec![
                    region(1, 4, first),
                    region(4, 6, second),
                    region(8, 11, third),
                ];
                for start in 0..=12 {
                    let mut p = player(12, regions.clone(), start);
                    let active = regions
                        .iter()
                        .position(|r| (r.start..r.end).contains(&start));
                    let mut expected = LoopPosition {
                        tick: start,
                        region: active,
                        pass: active.map(|_| 1),
                        exit_requested: false,
                        ended: start == 12,
                    };
                    assert_eq!(p.position(), expected);
                    for i in 0..120 {
                        if let Some(index) = expected.region {
                            let requested = i % 11 == 10;
                            expected.exit_requested = requested;
                            p.set_exit_at_end(index, requested).unwrap();
                        }
                        let delta = [0, 1, 2, 3, 4, 17, 31][i % 7];
                        for _ in 0..delta {
                            reference_tick(&mut expected, 12, &regions);
                        }
                        assert_eq!(
                            p.advance(delta).unwrap(),
                            expected,
                            "start {start}, step {i}, regions {regions:?}"
                        );
                    }
                }
            }
        }
    }
}
