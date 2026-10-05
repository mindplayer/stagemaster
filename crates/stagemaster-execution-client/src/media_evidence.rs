//! One bounded diagnostic for the original media path, not a command or execution authority.
use crate::{MediaAction, Record, validation};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTarget {
    pub host_id: String,
    pub revision: String,
    pub group: String,
    pub generation: String,
    pub action: MediaAction,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LocalMediaProblem {
    InvalidTarget,
    SendPreflight,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseProblem {
    Encoding,
    Connection,
    Incomplete,
    BodyLimit,
    HttpRefused,
    InvalidJson,
    ReceiptMismatch,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaHttpEvidence {
    pub status: Option<u16>,
    pub body_complete: bool,
    pub code: Option<String>,
    pub problem: Option<ResponseProblem>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceOutcome {
    Accepted,
    Rejected,
    Unknown,
    Other,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaReceiptEvidence {
    pub serial: String,
    pub complete: bool,
    pub outcome: Option<EvidenceOutcome>,
    pub code: Option<String>,
    pub media_request: Option<String>,
    pub generation: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaOperationEvidence {
    pub target: Option<MediaTarget>,
    pub serial: Option<String>,
    /// Started a send attempt, not proof that the server received or admitted the request.
    pub attempted: bool,
    pub submission: MediaHttpEvidence,
    pub receipt_read: Option<MediaHttpEvidence>,
    pub receipt: Option<MediaReceiptEvidence>,
    pub not_submitted_reason: Option<LocalMediaProblem>,
}

impl MediaOperationEvidence {
    pub(crate) fn receive(&mut self, record: &Record) {
        if self.serial.as_deref() != Some(&record.serial) {
            return;
        }
        let outcome = record.outcome.as_ref();
        let media = outcome
            .filter(|o| record.status == "complete" && o.kind == "accepted")
            .and_then(|o| o.state.as_ref())
            .and_then(|s| {
                self.target
                    .as_ref()
                    .and_then(|t| s.media.iter().find(|m| m.id == t.group))
            });
        self.receipt = Some(MediaReceiptEvidence {
            serial: record.serial.clone(),
            complete: record.status == "complete",
            outcome: outcome.map(|o| match o.kind.as_str() {
                "accepted" => EvidenceOutcome::Accepted,
                "rejected" => EvidenceOutcome::Rejected,
                "unknown" => EvidenceOutcome::Unknown,
                _ => EvidenceOutcome::Other,
            }),
            code: outcome.and_then(|o| o.code.as_deref()).and_then(known_code),
            media_request: media
                .and_then(|m| m.control.as_ref())
                .filter(|c| c.request != "0")
                .and_then(|c| bounded_decimal(&c.request)),
            generation: media.and_then(|m| bounded_decimal(&m.generation)),
        });
    }
}

fn bounded_decimal(value: &str) -> Option<String> {
    (value.len() <= 20 && validation::decimal(value).is_ok()).then(|| value.into())
}

pub(crate) fn known_code(value: &str) -> Option<String> {
    const WIRE: &[&str] = &[
        "invalid",
        "busy",
        "closed",
        "bodyLimit",
        "sessionLimit",
        "sessionMissing",
        "duplicateConflict",
        "notRetained",
        "sequence",
        "mediaTargetChanged",
        "loopTargetChanged",
        "unknown",
        "noControl",
        "exhausted",
        "workerFailed",
    ];
    const RUNTIME: &[&str] = &[
        "Identity",
        "Clock",
        "Busy",
        "Lease",
        "Sequence",
        "Revision",
        "Exhausted",
        "Mode",
        "Empty",
        "Selection",
        "NotLoaded",
        "Step",
        "State",
        "Budget",
        "Read",
        "Integrity",
        "Package",
        "Allocation",
        "Playback",
        "Permission(Missing)",
        "Permission(Expired)",
        "Permission(UncertainTime)",
        "Permission(Restricted)",
    ];
    const HOST: &[&str] = &[
        "Configuration",
        "NotPrepared",
        "ThreadSpawn",
        "QueueFull",
        "InvalidDeadline",
        "Deadline",
        "Closed",
        "ObservationBusy",
    ];
    let runtime = value
        .strip_prefix("Runtime(")
        .and_then(|v| v.strip_suffix(')'));
    (WIRE.contains(&value)
        || RUNTIME.contains(&value)
        || HOST.contains(&value)
        || runtime.is_some_and(|v| RUNTIME.contains(&v)))
    .then(|| value.into())
}

#[cfg(test)]
#[path = "media_evidence_tests.rs"]
mod tests;
