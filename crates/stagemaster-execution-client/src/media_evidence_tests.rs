use super::*;
use crate::{MediaCompletion, MediaControl, MediaState, MediaStatus, Outcome, State};
use serde_json::json;
const ID: &str = "66666666-0000-4000-8000-000000000002";
const MAX: &str = "18446744073709551615";

fn evidence() -> MediaOperationEvidence {
    MediaOperationEvidence {
        target: Some(MediaTarget {
            host_id: ID.into(),
            revision: MAX.into(),
            group: ID.into(),
            generation: MAX.into(),
            action: MediaAction::ExitLoop {
                instance: MAX.into(),
                region: 127,
                pass: MAX.into(),
                requested: true,
            },
        }),
        serial: Some(MAX.into()),
        attempted: true,
        submission: MediaHttpEvidence {
            status: Some(200),
            body_complete: true,
            code: None,
            problem: None,
        },
        receipt_read: Some(MediaHttpEvidence {
            status: Some(200),
            body_complete: true,
            code: None,
            problem: None,
        }),
        ..MediaOperationEvidence::default()
    }
}
fn record(group: &str, request: &str, generation: &str) -> Record {
    Record {
        serial: MAX.into(),
        status: "complete".into(),
        outcome: Some(Outcome {
            kind: "accepted".into(),
            code: Some("Runtime(Permission(UncertainTime))".into()),
            message: Some("Bearer secret-marker-Authorization".repeat(100_000)),
            state: Some(State {
                boot: ID.into(),
                layout: "private-layout-marker".into(),
                revision: MAX.into(),
                observed_ms: MAX.into(),
                sources: vec![],
                fault: false,
                owner: None,
                audio: None,
                output: None,
                media: vec![MediaState {
                    id: group.into(),
                    generation: generation.into(),
                    status: MediaStatus::Paused,
                    position_ms: 0,
                    control: Some(MediaControl {
                        request: request.into(),
                        status: MediaCompletion::Pending,
                        problem: Some("private-provider-marker".into()),
                    }),
                    termination: None,
                }],
            }),
        }),
    }
}

#[test]
fn media_evidence_maximum_legal_fields_remain_bounded_and_do_not_copy_private_state() {
    let mut value = evidence();
    value.receive(&record(ID, MAX, MAX));
    let bytes = serde_json::to_vec(&value).unwrap();
    assert!(bytes.len() < 4096);
    let text = String::from_utf8(bytes).unwrap();
    for secret in [
        "Bearer",
        "Authorization",
        "secret-marker",
        "private-layout-marker",
        "private-provider-marker",
    ] {
        assert!(!text.contains(secret));
    }
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["receipt"]["mediaRequest"], MAX);
    assert_eq!(json["receipt"]["generation"], MAX);
    assert_eq!(
        json["receipt"]["code"],
        "Runtime(Permission(UncertainTime))"
    );
}

#[test]
fn diagnostic_codes_are_known_literals_not_free_error_strings() {
    for code in [
        "loopTargetChanged",
        "mediaTargetChanged",
        "notRetained",
        "Deadline",
        "Runtime(Revision)",
    ] {
        assert_eq!(known_code(code).as_deref(), Some(code));
    }
    for code in [
        "Bearer secret",
        "Authorization",
        "http://private.invalid",
        "未审阅的自由文本",
        "Runtime(unknown-private)",
    ] {
        assert!(known_code(code).is_none());
    }
    assert!(known_code(&"f".repeat(64)).is_none());
    assert!(known_code(&"x".repeat(100_000)).is_none());
}

#[test]
fn unmatched_serial_or_group_cannot_supply_the_original_media_identity() {
    let mut value = evidence();
    let mut other = record(ID, "1", "1");
    other.serial = "1".into();
    value.receive(&other);
    assert!(value.receipt.is_none());
    value.receive(&record("other", "1", "1"));
    let raw = serde_json::to_value(&value).unwrap();
    assert_eq!(raw["receipt"]["mediaRequest"], json!(null));
    assert_eq!(raw["receipt"]["generation"], json!(null));
}

#[test]
fn unknown_outcome_and_invalid_counts_are_redacted_without_inventing_confirmation() {
    for count in ["0", "01", "+1", "18446744073709551616"] {
        let mut value = evidence();
        let mut returned = record(ID, count, "01");
        let outcome = returned.outcome.as_mut().unwrap();
        outcome.kind = "Bearer secret-marker".into();
        outcome.code = Some("secret-marker".into());
        value.receive(&returned);
        let raw = serde_json::to_value(&value).unwrap();
        assert_eq!(raw["receipt"]["outcome"], "other");
        assert_eq!(raw["receipt"]["code"], json!(null));
        assert_eq!(raw["receipt"]["mediaRequest"], json!(null));
        assert_eq!(raw["receipt"]["generation"], json!(null));
        assert!(!raw.to_string().contains("secret-marker"));
    }
}

#[test]
fn a_refusal_or_incomplete_receipt_cannot_borrow_a_prior_media_control_from_state() {
    for (status, kind) in [
        ("complete", "rejected"),
        ("complete", "unknown"),
        ("complete", "other"),
        ("pending", "accepted"),
    ] {
        let mut value = evidence();
        let mut returned = record(ID, "7", "5");
        returned.status = status.into();
        returned.outcome.as_mut().unwrap().kind = kind.into();
        value.receive(&returned);
        let raw = serde_json::to_value(&value).unwrap();
        assert_eq!(
            raw["receipt"]["mediaRequest"],
            json!(null),
            "{kind}/{status}"
        );
        assert_eq!(raw["receipt"]["generation"], json!(null), "{kind}/{status}");
    }
}

#[test]
fn accepted_media_count_projection_rejects_noncanonical_or_overflowing_values() {
    for count in ["0", "01", "+1", "18446744073709551616"] {
        let mut value = evidence();
        value.receive(&record(ID, count, "5"));
        assert!(value.receipt.unwrap().media_request.is_none(), "{count}");
    }
    for count in ["01", "+1", "18446744073709551616"] {
        let mut value = evidence();
        value.receive(&record(ID, "7", count));
        assert!(value.receipt.unwrap().generation.is_none(), "{count}");
    }
}
