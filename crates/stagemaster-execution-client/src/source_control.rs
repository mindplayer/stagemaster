use crate::{Action, Client, LocalSourceProblem, Selection, SourceOperationEvidence, View};
use serde_json::json;

impl Client {
    pub async fn apply(
        &mut self,
        host: &str,
        revision: &str,
        source: &str,
        action: Action,
    ) -> Result<View, String> {
        let entry = self.catalog.sources.iter().find(|s| s.id == source);
        let diagnose = matches!(
            action,
            Action::Start { .. }
                | Action::Pause {}
                | Action::Resume {}
                | Action::Next {}
                | Action::Stop {}
        ) && entry.is_none_or(|s| {
            matches!(
                s.selection,
                Selection::Scene { .. } | Selection::Sequence { .. }
            )
        });
        let new_evidence = diagnose && !self.pending;
        if new_evidence {
            let target = (host == self.transport.discovery.host_id)
                .then(|| {
                    SourceOperationEvidence::target(&self.catalog, host, revision, source, &action)
                })
                .flatten();
            self.source_operation = Some(SourceOperationEvidence {
                target,
                ..SourceOperationEvidence::default()
            });
        }
        // Preserve original business validation and wire behavior; diagnostic checks don't authorize.
        if host != self.transport.discovery.host_id || entry.is_none() {
            if new_evidence && let Some(evidence) = &mut self.source_operation {
                evidence.not_submitted_reason = Some(LocalSourceProblem::InvalidTarget);
            }
            return Err("操作对应的后台或节目已更换，请重新选择".into());
        }
        if entry.is_some_and(|s| matches!(s.selection, Selection::AudioTimeline {}))
            && !matches!(action, Action::Level { .. })
        {
            return Err("音乐请使用播放、暂停、停止或定位操作".into());
        }
        if let Action::Patch { changes } = &action {
            crate::manual_validation::edits(&self.catalog, source, changes)?;
        }
        let command = json!({"kind":"submit","expectedRevision":revision,"action":{"kind":"source","source":source,"action":action}});
        let result = self.send_original(command, true, diagnose).await;
        if new_evidence
            && result.is_err()
            && let Some(evidence) = &mut self.source_operation
            && !evidence.attempted
            && evidence.not_submitted_reason.is_none()
        {
            evidence.not_submitted_reason = Some(LocalSourceProblem::SendPreflight);
        }
        result
    }
}
