use crate::{PlaybackSelection, player::Player};
use crate::{Session, SourceSpec, source::Entry};
use stagemaster_engine::live::{Kind, LiveMixer, MAX_SOURCES, Source};
use stagemaster_project::Document;
use stagemaster_project::LiveSourceBudget;

impl Session {
    /// Prepare all sources from one immutable snapshot. No output or playback is started.
    /// Total targets/effect channels/keyframes share the single-plan limits across the group.
    /// # Errors
    /// Reject empty, duplicate, unknown, oversized or manual-only groups, or preparation failure.
    pub fn prepare(
        doc: &Document,
        boot: [u8; 16],
        specs: &[SourceSpec],
        now_ms: u64,
    ) -> Result<Self, String> {
        if specs
            .iter()
            .any(|s| matches!(s.playback, Some(PlaybackSelection::AudioTimeline)))
        {
            return Err("音乐轨道必须绑定明确的媒体同步组".into());
        }
        Self::prepare_sources(doc, boot, specs, now_ms)
    }
    pub(super) fn prepare_sources(
        doc: &Document,
        boot: [u8; 16],
        specs: &[SourceSpec],
        now_ms: u64,
    ) -> Result<Self, String> {
        if boot == [0; 16] || !(1..=MAX_SOURCES).contains(&specs.len()) {
            return Err("执行实例身份或来源数量无效".into());
        }
        for (i, spec) in specs.iter().enumerate() {
            if spec.id == [0; 16] || specs[..i].iter().any(|s| s.id == spec.id) {
                return Err("来源身份无效或重复".into());
            }
        }
        let mut prepared = Vec::with_capacity(specs.len());
        let mut budget = LiveSourceBudget::default();
        for spec in specs {
            let player = spec
                .playback
                .as_ref()
                .map(|selection| Player::prepare(doc, selection, now_ms))
                .transpose()?;
            if let Some(player) = &player {
                budget.add(player.budget())?;
            }

            prepared.push(player);
        }
        let first = prepared
            .iter()
            .flatten()
            .next()
            .ok_or("来源组至少需要一个播放来源")?;
        let output = first.prepare_output()?;
        let layout = first.layout().clone();
        if prepared.iter().flatten().any(|p| p.layout() != &layout) {
            return Err("来源组工程属性布局不一致".into());
        }
        let count = layout.attributes().len();
        let mut mixer = LiveMixer::new(boot, layout, specs.len()).map_err(|e| e.to_string())?;
        let mut sources = Vec::with_capacity(specs.len());
        for (spec, player) in specs.iter().zip(prepared) {
            let kind = if player.is_some() {
                Kind::Playback
            } else {
                Kind::Programmer
            };
            let handle = mixer
                .open(
                    Source { id: spec.id, kind },
                    spec.priority,
                    mixer.layout().id(),
                )
                .map_err(|e| e.to_string())?;
            sources.push(Entry {
                id: spec.id,
                handle,
                serial: 0,
                level: u16::MAX,
                player,
                selection: spec.playback.clone(),
                media_group: None,
                sampled_at_ms: now_ms,
                reassert_at_ms: None,
                values: vec![None; count],
                times: vec![None; count],
                assertions: vec![false; count],
            });
        }
        Ok(Self {
            boot,
            clock: stagemaster_time::Clock::new(boot, 0).map_err(|e| e.to_string())?,
            mixer,
            output,
            sources,
            claims: Vec::with_capacity(count * specs.len()),
            now_ms,
            sequence: 0,
            frame: None,
            fault: None,
            media: Vec::new(),
        })
    }
}
