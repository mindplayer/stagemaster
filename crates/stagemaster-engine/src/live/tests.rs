use super::*;
use stagemaster_domain::{MixMode, NormalizedValue};

fn layout() -> Layout {
    Layout::new(
        [3; 32],
        vec![
            Attribute {
                default: 40_000,
                mix: MixMode::HighestTakesPrecedence,
                intensity: true,
                discrete: false,
            },
            Attribute {
                default: 1_000,
                mix: MixMode::LatestTakesPrecedence,
                intensity: false,
                discrete: false,
            },
            Attribute {
                default: 0,
                mix: MixMode::LatestTakesPrecedence,
                intensity: false,
                discrete: true,
            },
        ],
    )
    .unwrap()
}
fn source(id: u8) -> Source {
    Source {
        id: [id; 16],
        kind: Kind::Playback,
    }
}
fn open(m: &mut LiveMixer, id: u8, priority: i16) -> Handle {
    m.open(source(id), priority, m.layout().id()).unwrap()
}
fn publish(m: &mut LiveMixer, h: Handle, serial: u64, values: &[Option<u16>], assert: &[bool]) {
    m.publish(
        h,
        Frame {
            layout: m.layout.id,
            serial,
            values,
            assert,
        },
    )
    .unwrap();
}
fn render(m: &LiveMixer) -> ([u16; 3], [Option<Handle>; 3]) {
    let mut values = [0; 3];
    let mut winners = [None; 3];
    m.render(&mut values, &mut winners).unwrap();
    (values, winners)
}
#[test]
fn intensity_level_never_scales_position_or_discrete_function_and_zero_keeps_ownership() {
    let mut m = LiveMixer::new([1; 16], layout(), 3).unwrap();
    let base = open(&mut m, 1, 0);
    let top = open(&mut m, 2, 10);
    publish(
        &mut m,
        base,
        1,
        &[Some(50_000), Some(30_000), Some(50_000)],
        &[false; 3],
    );
    publish(
        &mut m,
        top,
        1,
        &[Some(20_000), Some(60_000), Some(2_570)],
        &[false; 3],
    );
    m.set_level(top, 2, 32_768).unwrap();
    assert_eq!(
        render(&m).0,
        [
            NormalizedValue::from_raw(20_000)
                .scale(NormalizedValue::from_raw(32_768))
                .raw(),
            60_000,
            2_570
        ]
    );
    m.set_level(top, 3, 0).unwrap();
    assert_eq!(render(&m).0, [0, 60_000, 2_570]);
    m.close(top, 4).unwrap();
    assert_eq!(render(&m).0, [50_000, 30_000, 50_000]);
    m.close(base, 2).unwrap();
    assert_eq!(render(&m), ([40_000, 1_000, 0], [None; 3]));
}
#[test]
fn htp_takes_highest_value_but_defaults_never_compete_with_explicit_zero() {
    let mut m = LiveMixer::new([1; 16], layout(), 2).unwrap();
    let a = open(&mut m, 1, 0);
    let b = open(&mut m, 2, 0);
    publish(&mut m, a, 1, &[Some(0), None, None], &[false; 3]);
    assert_eq!(render(&m).0[0], 0);
    publish(&mut m, b, 1, &[Some(20_000), None, None], &[false; 3]);
    assert_eq!(render(&m).1[0], Some(b));
    m.set_level(b, 2, 0).unwrap();
    assert_eq!(render(&m).0[0], 0);
    assert_eq!(render(&m).0[1], 1_000);
}
#[test]
fn sampling_does_not_steal_ltp_and_assertions_are_per_attribute() {
    let mut m = LiveMixer::new([1; 16], layout(), 2).unwrap();
    let a = open(&mut m, 1, 0);
    let b = open(&mut m, 2, 0);
    publish(&mut m, a, 1, &[None, Some(10_000), Some(100)], &[false; 3]);
    publish(&mut m, b, 1, &[None, Some(20_000), Some(200)], &[false; 3]);
    publish(&mut m, a, 2, &[None, Some(30_000), Some(300)], &[false; 3]);
    assert_eq!(render(&m).0, [40_000, 20_000, 200]);
    publish(
        &mut m,
        a,
        3,
        &[None, Some(30_000), Some(300)],
        &[false, true, false],
    );
    assert_eq!(render(&m).0, [40_000, 30_000, 200]);
    publish(&mut m, a, 4, &[None, None, Some(301)], &[false; 3]);
    assert_eq!(render(&m).0, [40_000, 20_000, 200]);
    m.close(b, 2).unwrap();
    assert_eq!(render(&m).0, [40_000, 1_000, 301]);
}
#[test]
fn invalid_batches_leave_values_order_and_command_serial_unchanged() {
    let mut m = LiveMixer::new([1; 16], layout(), 1).unwrap();
    let h = open(&mut m, 1, 0);
    publish(&mut m, h, 1, &[Some(10_000), None, None], &[false; 3]);
    let before = render(&m);
    let state = m.source(h).unwrap();
    let order = m.order;
    let values = [Some(20_000), None, None];
    for (frame, expected) in [
        (
            Frame {
                layout: [9; 32],
                serial: 2,
                values: &values,
                assert: &[false; 3],
            },
            Error::Layout,
        ),
        (
            Frame {
                layout: [3; 32],
                serial: 1,
                values: &values,
                assert: &[false; 3],
            },
            Error::Sequence,
        ),
        (
            Frame {
                layout: [3; 32],
                serial: 2,
                values: &values[..2],
                assert: &[false; 3],
            },
            Error::Shape,
        ),
        (
            Frame {
                layout: [3; 32],
                serial: 2,
                values: &values,
                assert: &[false, true, false],
            },
            Error::Assertion,
        ),
    ] {
        assert_eq!(m.publish(h, frame), Err(expected));
        assert_eq!(render(&m), before);
        assert_eq!(m.source(h).unwrap(), state);
        assert_eq!(m.order, order);
    }
    let mut out = [7; 2];
    let mut winners = [Some(h); 3];
    assert_eq!(m.render(&mut out, &mut winners), Err(Error::Shape));
    assert_eq!(out, [7; 2]);
    assert_eq!(winners, [Some(h); 3]);
}
#[test]
fn reused_slots_reject_old_handles_and_full_source_budget_is_explicit() {
    let mut m = LiveMixer::new([1; 16], layout(), 1).unwrap();
    let old = open(&mut m, 1, 0);
    assert_eq!(m.open(source(1), 0, [3; 32]), Err(Error::Identity));
    assert_eq!(m.open(source(2), 0, [3; 32]), Err(Error::Budget));
    m.close(old, 1).unwrap();
    let new = open(&mut m, 1, 0);
    assert_ne!(old, new);
    assert_eq!(m.close(old, 2), Err(Error::Handle));
    assert_eq!(m.set_level(old, 2, 0), Err(Error::Handle));
    let mut other = LiveMixer::new([2; 16], layout(), 1).unwrap();
    open(&mut other, 1, 0);
    assert_eq!(other.set_level(new, 1, 0), Err(Error::Handle));
}
#[test]
fn exhausted_order_rejects_assertion_atomically_but_allows_release() {
    let mut m = LiveMixer::new([1; 16], layout(), 1).unwrap();
    let h = open(&mut m, 1, 0);
    publish(&mut m, h, 1, &[Some(10), None, None], &[false; 3]);
    m.order = u64::MAX;
    assert_eq!(
        m.publish(
            h,
            Frame {
                layout: [3; 32],
                serial: 2,
                values: &[Some(20), None, None],
                assert: &[true, false, false]
            }
        ),
        Err(Error::Exhausted)
    );
    assert_eq!(render(&m).0[0], 10);
    assert_eq!(m.source(h).unwrap().serial, 1);
    m.close(h, 2).unwrap();
    assert_eq!(m.open(source(1), 0, [3; 32]), Err(Error::Exhausted));
    assert_eq!(render(&m).0[0], 40_000);
}
#[test]
fn prepared_buffers_and_order_stay_stable_during_continuous_updates() {
    let mut m = LiveMixer::new([1; 16], layout(), 4).unwrap();
    let h = open(&mut m, 1, 0);
    publish(&mut m, h, 1, &[Some(0), Some(0), None], &[false; 3]);
    let reserved = m.reserved_bytes();
    let order = m.order;
    let ptr = m.slots[0].cells.as_ptr();
    for n in 2..=10_000_u16 {
        publish(
            &mut m,
            h,
            u64::from(n),
            &[Some(n), Some(n), None],
            &[false; 3],
        );
        assert_eq!(render(&m).0, [n, n, 0]);
        assert_eq!(m.reserved_bytes(), reserved);
        assert_eq!(m.order, order);
        assert_eq!(m.slots[0].cells.as_ptr(), ptr);
    }
}
#[test]
fn discrete_rules_and_invalid_budgets_are_rejected_at_preparation() {
    let a = Attribute {
        default: 0,
        mix: MixMode::HighestTakesPrecedence,
        intensity: false,
        discrete: true,
    };
    assert_eq!(Layout::new([1; 32], vec![a]), Err(Error::Discrete));
    assert!(matches!(
        LiveMixer::new([1; 16], layout(), MAX_SOURCES + 1),
        Err(Error::Budget)
    ));
    assert!(matches!(
        LiveMixer::new([0; 16], layout(), 2),
        Err(Error::Identity)
    ));
}

#[test]
fn final_command_serial_is_reserved_for_release() {
    let mut m = LiveMixer::new([1; 16], layout(), 1).unwrap();
    let h = open(&mut m, 1, 0);
    publish(&mut m, h, u64::MAX - 1, &[Some(1), None, None], &[false; 3]);
    assert_eq!(m.set_level(h, u64::MAX, 0), Err(Error::Exhausted));
    assert_eq!(
        m.publish(
            h,
            Frame {
                layout: [3; 32],
                serial: u64::MAX,
                values: &[Some(2), None, None],
                assert: &[false; 3]
            }
        ),
        Err(Error::Exhausted)
    );
    assert_eq!(render(&m).0[0], 1);
    m.close(h, u64::MAX).unwrap();
    assert_eq!(render(&m).0[0], 40_000);
    let fresh = open(&mut m, 1, 0);
    assert_ne!(fresh, h);
}
