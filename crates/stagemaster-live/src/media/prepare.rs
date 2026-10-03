use super::{Group, GroupSpec, Status, nanos};
use crate::{Session, SourceSpec};
use stagemaster_project::Document;

impl Session {
    /// Prepare a fixed subset of sources to follow identified external media cursors.
    /// All compilation and allocations happen before ownership passes to a scheduling host.
    /// # Errors
    /// Reject invalid budgets, duplicate/unknown/manual members or shared source membership.
    pub fn prepare_with_media(
        doc: &Document,
        boot: [u8; 16],
        sources: &[SourceSpec],
        groups: &[GroupSpec],
        now_ms: u64,
    ) -> Result<Self, String> {
        nanos(now_ms)?;
        if groups.len() > sources.len() {
            return Err("同步组数量超出来源数量".into());
        }
        let mut session = Self::prepare(doc, boot, sources, now_ms)?;
        for (index, spec) in groups.iter().enumerate() {
            let limits = spec.limits;
            if spec.id == [0; 16]
                || groups[..index].iter().any(|g| g.id == spec.id)
                || spec.clock.id() == session.host_clock().id()
                || spec.sources.is_empty()
                || spec.sources.len() > sources.len()
                || limits.max_age_ns == 0
                || limits.max_uncertainty_ns == 0
                || limits.max_gap_ms == 0
                || !(1..=400).contains(&limits.max_rate_percent)
                || limits.position_tolerance_ms > 1000
            {
                return Err("同步组身份、成员或观测预算无效".into());
            }
            nanos(limits.max_gap_ms)?;
            let mut members = Vec::with_capacity(spec.sources.len());
            for id in &spec.sources {
                let source = session
                    .sources
                    .iter()
                    .position(|s| s.id == *id)
                    .ok_or("同步组来源不存在")?;
                let entry = &mut session.sources[source];
                if entry.player.is_none() || entry.media_group.is_some() {
                    return Err("手动来源或重复来源不能加入同步组".into());
                }
                let steps = entry
                    .player
                    .as_ref()
                    .ok_or("同步组缺少播放器")?
                    .plan()
                    .steps();
                if steps
                    .iter()
                    .take(steps.len().saturating_sub(1))
                    .any(|step| step.wait_ms.is_none())
                {
                    return Err("媒体跟随列表的中间步骤不能等待人工推进".into());
                }
                entry.media_group = Some(index);
                members.push(source);
            }
            session.media.push(Group {
                spec: spec.clone(),
                members,
                generation: 0,
                status: Status::Ready,
                last: None,
                observed_host_ns: 0,
            });
        }
        Ok(session)
    }
}
