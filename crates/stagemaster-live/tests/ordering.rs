mod support;
use stagemaster_live::{Change, Command, Session};
use support::*;

#[test]
fn actual_activation_order_and_same_time_tie_do_not_depend_on_preparation_order() {
    for delay in [70, 80] {
        let doc = decode(&fixture(delay));
        for reverse in [false, true] {
            let mut specs = specs();
            if reverse {
                specs.reverse();
            }
            let mut s = Session::prepare(&doc, [9; 16], &specs, 0).unwrap();
            for n in [1, 2] {
                s.control(s.key([n; 16]).unwrap(), Command::Execute(0), 0)
                    .unwrap();
            }
            s.tick(100).unwrap();
            assert_eq!(s.winner(1), Some([if delay == 70 { 1 } else { 2 }; 16]));
            assert_eq!(
                s.values().unwrap()[1],
                if delay == 70 { 30_000 } else { 20_000 }
            );
            assert_eq!(s.values().unwrap()[0], 25_000);
            let manual = s.key([3; 16]).unwrap();
            s.patch(
                manual,
                &[Change {
                    attribute: 1,
                    value: Some(40_000),
                }],
                100,
            )
            .unwrap();
            s.tick(150).unwrap();
            assert_eq!(s.winner(1), Some([3; 16]));
            s.patch(
                manual,
                &[Change {
                    attribute: 1,
                    value: None,
                }],
                150,
            )
            .unwrap();
            assert_eq!(s.winner(1), Some([if delay == 70 { 1 } else { 2 }; 16]));
        }
    }
}
#[test]
fn tracked_values_do_not_steal_but_explicit_jump_does_and_stop_is_local() {
    let doc = decode(&fixture(50));
    let mut s = Session::prepare(&doc, [9; 16], &specs(), 0).unwrap();
    let (a, b) = (s.key([1; 16]).unwrap(), s.key([2; 16]).unwrap());
    s.control(a, Command::Execute(0), 0).unwrap();
    s.control(b, Command::Execute(0), 0).unwrap();
    s.tick(65).unwrap();
    assert_eq!(s.winner(1), Some([2; 16])); // A's step at 60 only owns new intensity.
    s.control(a, Command::Execute(1), 65).unwrap();
    assert_eq!(s.winner(1), Some([1; 16]));
    s.control(a, Command::Pause, 70).unwrap();
    s.tick(100).unwrap();
    assert_eq!(s.winner(1), Some([1; 16]));
    s.control(a, Command::Stop, 100).unwrap();
    assert_eq!(s.winner(1), Some([2; 16]));
    assert_eq!(s.values().unwrap()[0], 0);
    s.control(b, Command::Stop, 100).unwrap();
    assert_eq!(s.winner(1), None);
    assert_eq!(s.values().unwrap(), [0; 4]);
}
#[test]
fn skipped_cycles_use_each_attributes_last_claim_instead_of_sample_time() {
    let mut raw = fixture(999_985);
    raw["lighting"]["scenes"][2]["assignments"] = serde_json::json!([set("blue", 15_000)]);
    raw["lighting"]["sequences"][0] = list(
        20,
        &[step(10, 1, 20, Some(30)), step(11, 3, 0, Some(50))],
        true,
    );
    for reverse in [false, true] {
        let mut specs = specs();
        if reverse {
            specs.reverse();
        }
        let mut s = Session::prepare(&decode(&raw), [9; 16], &specs, 0).unwrap();
        for n in [1, 2] {
            s.control(s.key([n; 16]).unwrap(), Command::Execute(0), 0)
                .unwrap();
        }
        // Current first step is still delayed. Last red claim was 999_920; blue was 999_950.
        s.tick(1_000_005).unwrap();
        assert_eq!(s.winner(1), Some([2; 16]));
        assert_eq!(s.winner(3), Some([1; 16]));
        assert_eq!(s.values().unwrap()[3], 15_000);
        s.tick(1_000_020).unwrap();
        assert_eq!(s.winner(1), Some([1; 16]));
        assert_eq!(s.winner(3), None);
    }
}
