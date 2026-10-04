use super::*;
use serde_json::{Value, json};
fn fixture() -> (Catalog, Value) {
    let catalog = serde_json::from_value(json!({
        "protocol":2,"execution":"sourceGroup","mode":"softwareOutput","physicalOutput":false,
        "projectId":"project","layout":"layout","capabilities":["sourceProgress"],
        "sources":[{"id":"source","name":"列表","priority":0,"selection":{"kind":"sequence","id":"list"},
            "steps":[{"id":"a","name":"第一步","number":"1"},{"id":"b","name":"第二步","number":"2"}]}]
    })).unwrap();
    let state = json!({"boot":"boot","layout":"layout","revision":"1","observedMs":"500","fault":false,"owner":null,
    "sources":[{"id":"source","level":65535,"status":"Running","step":"a","progress":{
        "phase":"fade","elapsedMs":"150","phaseElapsedMs":"50","phaseDurationMs":"200","nextStep":"b","nextWrap":false
    }}]});
    (catalog, state)
}
#[test]
fn legacy_state_without_progress_is_accepted_only_without_declaration() {
    let (mut catalog, mut raw) = fixture();
    raw["sources"][0]
        .as_object_mut()
        .unwrap()
        .remove("progress");
    let state: State = serde_json::from_value(raw).unwrap();
    assert!(validate(&catalog, &state).is_err());
    catalog.capabilities.clear();
    validate(&catalog, &state).unwrap();
    assert!(
        serde_json::to_value(&state).unwrap()["sources"][0]
            .get("progress")
            .is_none()
    );
}
#[test]
fn unknown_steps_invalid_time_and_inconsistent_phase_are_rejected() {
    let (catalog, raw) = fixture();
    validate(&catalog, &serde_json::from_value(raw.clone()).unwrap()).unwrap();
    for (path, value) in [
        (
            "/sources/0/progress/elapsedMs",
            json!("18446744073709551616"),
        ),
        ("/sources/0/progress/elapsedMs", json!("+150")),
        ("/sources/0/progress/phaseElapsedMs", json!("151")),
        ("/sources/0/progress/phaseDurationMs", json!("0")),
        ("/sources/0/progress/phaseDurationMs", json!("50")),
        ("/sources/0/progress/phaseDurationMs", Value::Null),
        ("/sources/0/progress/nextStep", json!("missing")),
        ("/sources/0/progress/nextStep", json!("a")),
        ("/sources/0/progress/nextStep", Value::Null),
        ("/sources/0/progress/nextWrap", json!(true)),
        ("/sources/0/progress/phase", json!("idle")),
        ("/sources/0/step", json!("missing")),
        ("/sources/0/status", json!("Finished")),
    ] {
        let mut bad = raw.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(
            validate(&catalog, &serde_json::from_value(bad).unwrap()).is_err(),
            "{path}"
        );
    }
}
#[test]
fn paused_hold_can_use_full_u64_and_loop_to_first_step() {
    let (catalog, mut raw) = fixture();
    raw["sources"][0]["status"] = json!("Paused");
    raw["sources"][0]["step"] = json!("b");
    raw["sources"][0]["progress"] = json!({"phase":"hold","elapsedMs":u64::MAX.to_string(),
        "phaseElapsedMs":u64::MAX.to_string(),"phaseDurationMs":null,"nextStep":"a","nextWrap":true});
    validate(&catalog, &serde_json::from_value(raw).unwrap()).unwrap();
}
