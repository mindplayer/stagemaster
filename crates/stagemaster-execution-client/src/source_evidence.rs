//! One bounded original ordinary-program control diagnostic; never execution authority.
use crate::{Action, Catalog, MediaHttpEvidence, Record, Selection, validation};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SourceCommand {
    Start { step: String },
    Pause {},
    Resume {},
    Next {},
    Stop {},
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTarget {
    pub host_id: String,
    pub revision: String,
    pub source: String,
    pub action: SourceCommand,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LocalSourceProblem {
    InvalidTarget,
    SendPreflight,
}
#[derive(Clone, Debug, Serialize)]
pub struct SourceReceiptState {
    pub revision: String,
    pub status: String,
    pub step: Option<String>,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceOutcome {
    Applied,
    Rejected,
    Unknown,
    Other,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceReceiptEvidence {
    pub serial: String,
    pub complete: bool,
    pub outcome: Option<SourceOutcome>,
    pub code: Option<String>,
    pub source_state: Option<SourceReceiptState>,
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceOperationEvidence {
    pub target: Option<SourceTarget>,
    pub serial: Option<String>,
    pub attempted: bool,
    pub submission: MediaHttpEvidence,
    pub receipt_read: Option<MediaHttpEvidence>,
    pub receipt: Option<SourceReceiptEvidence>,
    pub not_submitted_reason: Option<LocalSourceProblem>,
}
impl SourceOperationEvidence {
    pub(crate) fn target(
        catalog: &Catalog,
        host: &str,
        revision: &str,
        source: &str,
        action: &Action,
    ) -> Option<SourceTarget> {
        validation::identity(host).ok()?;
        validation::decimal(revision).ok()?;
        let entry = catalog.sources.iter().find(|s| {
            s.id == source
                && matches!(
                    s.selection,
                    Selection::Scene { .. } | Selection::Sequence { .. }
                )
        })?;
        let action = match action {
            Action::Start { step } if entry.steps.iter().any(|s| s.id == *step) => {
                SourceCommand::Start { step: step.clone() }
            }
            Action::Pause {} => SourceCommand::Pause {},
            Action::Resume {} => SourceCommand::Resume {},
            Action::Next {} => SourceCommand::Next {},
            Action::Stop {} => SourceCommand::Stop {},
            _ => return None,
        };
        Some(SourceTarget {
            host_id: host.into(),
            revision: revision.into(),
            source: source.into(),
            action,
        })
    }
    pub(crate) fn receive(&mut self, record: &Record, catalog: &Catalog) {
        if self.serial.as_deref() != Some(&record.serial) {
            return;
        }
        let outcome = record
            .outcome
            .as_ref()
            .filter(|_| record.status == "complete");
        let source_state = (|| {
            let target = self.target.as_ref()?;
            let state = outcome
                .filter(|o| record.status == "complete" && o.kind == "applied")?
                .state
                .as_ref()?;
            if state.boot != target.host_id || state.layout != catalog.layout {
                return None;
            }
            validation::decimal(&state.revision).ok()?;
            let entry = catalog.sources.iter().find(|s| s.id == target.source)?;
            let source = state.sources.iter().find(|s| s.id == target.source)?;
            let status = source.status.as_ref()?;
            if !matches!(status.as_str(), "Idle" | "Running" | "Paused" | "Finished")
                || source
                    .step
                    .as_ref()
                    .is_some_and(|step| !entry.steps.iter().any(|s| s.id == *step))
            {
                return None;
            }
            Some(SourceReceiptState {
                revision: state.revision.clone(),
                status: status.clone(),
                step: source.step.clone(),
            })
        })();
        self.receipt = Some(SourceReceiptEvidence {
            serial: record.serial.clone(),
            complete: record.status == "complete",
            outcome: outcome.map(|o| match o.kind.as_str() {
                "applied" => SourceOutcome::Applied,
                "rejected" => SourceOutcome::Rejected,
                "unknown" => SourceOutcome::Unknown,
                _ => SourceOutcome::Other,
            }),
            code: outcome
                .and_then(|o| o.code.as_deref())
                .and_then(crate::media_evidence::known_code),
            source_state,
        });
    }
}

#[cfg(test)]
#[path = "source_evidence_tests.rs"]
mod tests;
