use crate::{Catalog, Selection, State, validation::decimal};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Idle,
    Delay,
    Fade,
    Wait,
    Hold,
    Finished,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub phase: Phase,
    pub elapsed_ms: String,
    pub phase_elapsed_ms: String,
    pub phase_duration_ms: Option<String>,
    pub next_step: Option<String>,
    pub next_wrap: bool,
}

pub(crate) fn validate(catalog: &Catalog, state: &State) -> Result<(), String> {
    let declared = catalog.capabilities.iter().any(|c| c == "sourceProgress");
    for source in &state.sources {
        let entry = catalog
            .sources
            .iter()
            .find(|s| s.id == source.id)
            .ok_or("后台进度来源不存在")?;
        let program = matches!(
            entry.selection,
            Selection::Scene { .. } | Selection::Sequence { .. }
        );
        let Some(p) = &source.progress else {
            if declared && program {
                return Err("后台缺少已声明的节目进度".into());
            }
            continue;
        };
        let elapsed = decimal(&p.elapsed_ms)?;
        let phase_elapsed = decimal(&p.phase_elapsed_ms)?;
        let duration = p.phase_duration_ms.as_deref().map(decimal).transpose()?;
        let current = source
            .step
            .as_ref()
            .and_then(|id| entry.steps.iter().position(|s| &s.id == id));
        let next = p
            .next_step
            .as_ref()
            .and_then(|id| entry.steps.iter().position(|s| &s.id == id));
        let timing_valid = match p.phase {
            Phase::Idle => {
                source.status.as_deref() == Some("Idle")
                    && source.step.is_none()
                    && elapsed == 0
                    && phase_elapsed == 0
                    && duration.is_none()
                    && next.is_none()
            }
            Phase::Finished => {
                source.status.as_deref() == Some("Finished")
                    && current.is_some_and(|i| i + 1 == entry.steps.len())
                    && phase_elapsed == 0
                    && duration.is_none()
                    && next.is_none()
            }
            Phase::Delay | Phase::Fade | Phase::Wait | Phase::Hold => {
                matches!(source.status.as_deref(), Some("Running" | "Paused"))
                    && current.is_some()
                    && if p.phase == Phase::Hold {
                        duration.is_none()
                    } else {
                        duration.is_some_and(|d| d > 0 && d <= 86_400_000 && phase_elapsed < d)
                    }
            }
        };
        let next_valid = match (current, next, p.next_wrap) {
            (Some(i), Some(n), false) => n == i + 1,
            (Some(i), Some(0), true) => i + 1 == entry.steps.len(),
            (_, None, false) => {
                p.next_step.is_none()
                    && (matches!(p.phase, Phase::Idle | Phase::Finished)
                        || current.is_some_and(|i| i + 1 == entry.steps.len()))
            }
            _ => false,
        };
        if !declared || !program || phase_elapsed > elapsed || !timing_valid || !next_valid {
            return Err("后台节目进度与状态、时间或步骤不一致".into());
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "progress_tests.rs"]
mod tests;
