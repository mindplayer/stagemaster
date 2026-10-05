use crate::{
    Client, LocalMediaProblem, MediaAction, MediaOperationEvidence, MediaTarget, View, validation,
};
use serde_json::json;

impl Client {
    /// Submit to the original authority. An accepted receipt is not media execution completion.
    pub async fn apply_media(
        &mut self,
        host: &str,
        revision: &str,
        group: &str,
        generation: &str,
        action: MediaAction,
    ) -> Result<View, String> {
        let checked = self.check_media(host, revision, group, generation, &action);
        let new_evidence = !self.pending;
        if new_evidence {
            self.media_operation = Some(MediaOperationEvidence {
                target: checked.as_ref().ok().map(|()| MediaTarget {
                    host_id: host.into(),
                    revision: revision.into(),
                    group: group.into(),
                    generation: generation.into(),
                    action: action.clone(),
                }),
                not_submitted_reason: checked
                    .as_ref()
                    .err()
                    .map(|_| LocalMediaProblem::InvalidTarget),
                ..MediaOperationEvidence::default()
            });
        }
        checked?;
        let result = self
            .send(
                json!({"kind":"submit","expectedRevision":revision,"action":{
                    "kind":"media","group":group,"generation":generation,"action":action
                }}),
            )
            .await;
        if new_evidence
            && result.is_err()
            && let Some(evidence) = &mut self.media_operation
            && !evidence.attempted
        {
            evidence.not_submitted_reason = Some(LocalMediaProblem::SendPreflight);
        }
        result
    }

    fn check_media(
        &self,
        host: &str,
        revision: &str,
        group: &str,
        generation: &str,
        action: &MediaAction,
    ) -> Result<(), String> {
        let catalog = self.catalog.audio.as_ref().ok_or("后台没有音乐来源")?;
        if host != self.transport.discovery.host_id || group != catalog.group {
            return Err("音乐对应的后台或同步组已更换，请重新选择".into());
        }
        validation::decimal(revision)?;
        validation::decimal(generation)?;
        if let MediaAction::ExitLoop {
            instance,
            region,
            pass,
            ..
        } = action
            && (!catalog.performance_loops
                || *region >= 128
                || validation::decimal(instance)? == 0
                || validation::decimal(pass)? == 0)
        {
            return Err("当前后台不支持循环控制或循环目标无效".into());
        }
        if let MediaAction::Recover { position_ms } = action
            && (!catalog.provider_recovery || *position_ms >= catalog.duration_ms)
        {
            return Err("当前后台不支持音乐恢复或准备位置超出范围".into());
        }
        if let MediaAction::Seek { position_ms, .. } = action
            && (*position_ms > catalog.duration_ms
                || (*position_ms == catalog.duration_ms && !catalog.seek_includes_end))
        {
            return Err(if catalog.seek_includes_end {
                "定位超出音乐范围"
            } else {
                "定位须在音乐范围内；当前后台尚不支持直接定位到末尾"
            }
            .into());
        }
        Ok(())
    }
}
