mod support;
use serde_json::json;
use stagemaster_live::{Change, Command, Session};
use stagemaster_project::PackageSelection;
use support::*;

#[test]
fn aggregate_plan_budget_rejects_many_individually_valid_programs() {
    let mut r = fixture(70);
    let original = r["lighting"]["fixtures"][0].clone();
    let patch = r["lighting"]["patches"][0].clone();
    for n in 1..32 {
        let mut fixture = original.clone();
        fixture["id"] = json!(id(1000 + n));
        let mut patched = patch.clone();
        patched["fixtureId"] = fixture["id"].clone();
        patched["address"] = json!(1 + n * 4);
        r["lighting"]["fixtures"]
            .as_array_mut()
            .unwrap()
            .push(fixture);
        r["lighting"]["patches"]
            .as_array_mut()
            .unwrap()
            .push(patched);
    }
    r["lighting"]["sequences"] = json!([list(
        20,
        &(0..129)
            .map(|n| step(2000 + n, 1, 0, None))
            .collect::<Vec<_>>(),
        false
    )]);
    let doc = decode(&r);
    let specs: Vec<_> = (1..=16)
        .map(|n| playback(n, PackageSelection::Sequence { id: id(20) }))
        .collect();
    assert!(Session::prepare(&doc, [9; 16], &specs[..15], 0).is_ok());
    let error = Session::prepare(&doc, [9; 16], &specs, 0).err().unwrap();
    assert!(error.contains("累计"), "{error}");
}
#[test]
fn command_processes_due_automatic_claims_before_manual_intent_and_priority_still_wins() {
    let doc = decode(&fixture(80));
    let mut s = Session::prepare(&doc, [9; 16], &specs(), 0).unwrap();
    for n in [1, 2] {
        s.control(s.key([n; 16]).unwrap(), Command::Execute(0), 0)
            .unwrap();
    }
    let m = s.key([3; 16]).unwrap();
    let change = [Change {
        attribute: 1,
        value: Some(40_000),
    }];
    s.patch(m, &change, 80).unwrap();
    assert_eq!(s.winner(1), Some([3; 16]));
    s.control(s.key([1; 16]).unwrap(), Command::Execute(2), 80)
        .unwrap();
    assert_eq!(s.winner(1), Some([1; 16]));
    s.patch(m, &change, 80).unwrap();
    assert_eq!(s.winner(1), Some([3; 16])); // Explicit same-value edit is still an assertion.
    let mut specs = specs();
    specs[1].priority = 1;
    let mut s = Session::prepare(&doc, [10; 16], &specs, 0).unwrap();
    s.control(s.key([2; 16]).unwrap(), Command::Execute(0), 0)
        .unwrap();
    s.patch(s.key([3; 16]).unwrap(), &change, 80).unwrap();
    assert_eq!(s.winner(1), Some([2; 16]));
}
