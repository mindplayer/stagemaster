use super::*;
use crate::{Outcome, SourceState, State};
use serde_json::json;
const HOST: &str = "66666666-0000-4000-8000-000000000001";
const SOURCE: &str = "66666666-0000-4000-8000-000000000002";
const STEP: &str = "66666666-0000-4000-8000-000000000003";
const MAX: &str = "18446744073709551615";
fn catalog() -> Catalog {
    serde_json::from_value(json!({"protocol":2,"execution":"sourceGroup","mode":"software","physicalOutput":false,"projectId":HOST,"layout":"layout","sources":[{"id":SOURCE,"name":"测试场景","priority":0,"selection":{"kind":"scene","id":STEP},"steps":[{"id":STEP,"name":"开幕","number":"1"}]}]})).unwrap()
}
fn evidence() -> SourceOperationEvidence {
    SourceOperationEvidence {
        target: SourceOperationEvidence::target(
            &catalog(),
            HOST,
            MAX,
            SOURCE,
            &Action::Start { step: STEP.into() },
        ),
        serial: Some(MAX.into()),
        attempted: true,
        ..SourceOperationEvidence::default()
    }
}
fn record(kind: &str) -> Record {
    Record {
        serial: MAX.into(),
        status: "complete".into(),
        outcome: Some(Outcome {
            kind: kind.into(),
            code: Some("Runtime(Revision)".into()),
            message: Some("Bearer private-marker-Authorization".repeat(100_000)),
            state: Some(State {
                boot: HOST.into(),
                layout: "layout".into(),
                revision: MAX.into(),
                observed_ms: MAX.into(),
                sources: vec![SourceState {
                    id: SOURCE.into(),
                    level: 65535,
                    status: Some("Paused".into()),
                    step: Some(STEP.into()),
                    progress: None,
                    held: None,
                    held_values: None,
                }],
                fault: false,
                owner: None,
                media: vec![],
                audio: None,
                output: None,
            }),
        }),
    }
}
#[test]
fn bounded_evidence_never_copies_private_message_layout_or_arbitrary_target() {
    let mut value = evidence();
    value.receive(&record("applied"), &catalog());
    let text = serde_json::to_string(&value).unwrap();
    assert!(text.len() < 4096);
    for secret in [
        "Bearer",
        "Authorization",
        "private-marker",
        "private-layout",
    ] {
        assert!(!text.contains(secret));
    }
    for bad in ["01", "-1", "18446744073709551616", &"x".repeat(100_000)] {
        assert!(
            SourceOperationEvidence::target(&catalog(), HOST, bad, SOURCE, &Action::Pause {})
                .is_none()
        );
    }
    assert!(
        SourceOperationEvidence::target(
            &catalog(),
            HOST,
            MAX,
            SOURCE,
            &Action::Start {
                step: "x".repeat(100_000)
            }
        )
        .is_none()
    );
}
#[test]
fn rejected_unknown_pending_wrong_host_step_and_serial_never_borrow_state() {
    for kind in ["rejected", "unknown", "other", "accepted"] {
        let mut value = evidence();
        value.receive(&record(kind), &catalog());
        assert!(value.receipt.unwrap().source_state.is_none());
    }
    let mut value = evidence();
    let mut receipt = record("applied");
    receipt.status = "pending".into();
    value.receive(&receipt, &catalog());
    assert!(value.receipt.as_ref().unwrap().source_state.is_none());
    assert!(value.receipt.as_ref().unwrap().outcome.is_none());
    receipt.status = "complete".into();
    receipt.serial = "1".into();
    value.receipt = None;
    value.receive(&receipt, &catalog());
    assert!(value.receipt.is_none());
    receipt.serial = MAX.into();
    receipt
        .outcome
        .as_mut()
        .unwrap()
        .state
        .as_mut()
        .unwrap()
        .boot = SOURCE.into();
    value.receive(&receipt, &catalog());
    assert!(value.receipt.as_ref().unwrap().source_state.is_none());
    receipt
        .outcome
        .as_mut()
        .unwrap()
        .state
        .as_mut()
        .unwrap()
        .boot = HOST.into();
    receipt
        .outcome
        .as_mut()
        .unwrap()
        .state
        .as_mut()
        .unwrap()
        .layout = "private-layout".into();
    value.receive(&receipt, &catalog());
    assert!(value.receipt.as_ref().unwrap().source_state.is_none());
    receipt
        .outcome
        .as_mut()
        .unwrap()
        .state
        .as_mut()
        .unwrap()
        .layout = "layout".into();
    receipt
        .outcome
        .as_mut()
        .unwrap()
        .state
        .as_mut()
        .unwrap()
        .sources[0]
        .step = Some("private-step".repeat(100_000));
    value.receive(&receipt, &catalog());
    assert!(value.receipt.unwrap().source_state.is_none());
}
#[test]
fn invalid_status_code_and_revision_are_not_copied_as_source_receipt_state() {
    let mut value = evidence();
    let mut receipt = record("applied");
    let outcome = receipt.outcome.as_mut().unwrap();
    outcome.code = Some("private-code".repeat(100_000));
    outcome.state.as_mut().unwrap().sources[0].status = Some("private-status".repeat(100_000));
    value.receive(&receipt, &catalog());
    let raw = serde_json::to_value(&value).unwrap();
    assert!(raw["receipt"]["code"].is_null());
    assert!(raw["receipt"]["sourceState"].is_null());
    let state = receipt.outcome.as_mut().unwrap().state.as_mut().unwrap();
    state.sources[0].status = Some("Paused".into());
    state.revision = "01".into();
    value.receive(&receipt, &catalog());
    assert!(value.receipt.unwrap().source_state.is_none());
}
