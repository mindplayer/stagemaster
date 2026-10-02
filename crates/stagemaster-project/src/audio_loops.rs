//! Persistent loop authoring and conversion into the independent playback scheduler.
use crate::{AudioTimeline, array};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stagemaster_playback::{LoopPlays, LoopRegion, LoopSchedule};

pub(super) const CAPABILITY: &str = "media.audio-loop-regions";
pub use stagemaster_playback::MAX_LOOP_REGIONS as MAX_AUDIO_LOOP_REGIONS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum AudioLoopPlays {
    Count { count: u32 },
    UntilExit {},
}
impl AudioLoopPlays {
    pub(super) const fn playback(self) -> LoopPlays {
        match self {
            Self::Count { count } => LoopPlays::Count(count),
            Self::UntilExit {} => LoopPlays::UntilExit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioLoopRegion {
    pub id: String,
    pub name: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub plays: AudioLoopPlays,
    pub enabled: bool,
    pub locked: bool,
}

pub struct CompiledAudioLoops {
    /// Indices match `schedule.regions()`, with disabled objects omitted from both.
    pub region_ids: Vec<String>,
    pub schedule: LoopSchedule,
}

impl AudioTimeline {
    /// Compile one immutable run snapshot in a caller-selected integer timebase.
    /// # Errors
    /// Reject zero resolution, invalid regions or ranges lost during quantization.
    pub fn compile_loops(&self, ticks_per_second: u32) -> Result<CompiledAudioLoops, String> {
        validate_regions(self)?;
        if ticks_per_second == 0 {
            return Err("循环时基须大于零".into());
        }
        let ticks = |ms: u64| {
            ms.checked_mul(u64::from(ticks_per_second))
                .map(|v| v / 1000)
                .ok_or_else(|| "循环时间换算超出范围".to_owned())
        };
        let mut region_ids = Vec::new();
        let mut regions = Vec::new();
        for region in self.loop_regions.iter().filter(|r| r.enabled) {
            regions.push(LoopRegion {
                start: ticks(region.start_ms)?,
                end: ticks(region.end_ms)?,
                plays: region.plays.playback(),
            });
            region_ids.push(region.id.clone());
        }
        Ok(CompiledAudioLoops {
            region_ids,
            schedule: LoopSchedule::new(ticks(self.duration_ms())?, regions)?,
        })
    }
}

pub(super) fn validate_regions(track: &AudioTimeline) -> Result<(), String> {
    if track.loop_regions.len() > MAX_AUDIO_LOOP_REGIONS {
        return Err("演出循环最多 128 个区段".into());
    }
    let mut previous_end = 0;
    for region in &track.loop_regions {
        if region.name.trim().is_empty() {
            return Err("循环区段名称不能为空".into());
        }
        if region.start_ms >= region.end_ms || region.end_ms > track.duration_ms() {
            return Err(format!(
                "循环区段“{}”的结束须晚于开始，且位于音乐范围内",
                region.name
            ));
        }
        if region.start_ms < previous_end {
            return Err(format!("循环区段“{}”与前一段重叠，请调整范围", region.name));
        }
        if region.plays == (AudioLoopPlays::Count { count: 0 }) {
            return Err(format!("循环区段“{}”的总播放次数须至少为 1", region.name));
        }
        previous_end = region.end_ms;
    }
    Ok(())
}

pub(super) fn validate(root: &Value, track: Option<&AudioTimeline>) -> Result<(), String> {
    let declared = array(root, "requires")
        .iter()
        .any(|r| r["key"] == CAPABILITY && r["version"] == 1);
    let Some(track) = track else {
        return if declared {
            Err("循环区段能力声明缺少音乐轨道".into())
        } else {
            Ok(())
        };
    };
    if !track.loop_regions.is_empty() && !declared {
        return Err("工程缺少演出循环区段能力声明".into());
    }
    validate_regions(track)
}

pub(super) fn declare_if_needed(root: &mut Value, track: &AudioTimeline) -> Result<(), String> {
    if !track.loop_regions.is_empty()
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == CAPABILITY)
    {
        root["requires"]
            .as_array_mut()
            .ok_or("能力列表无效")?
            .push(json!({"key":CAPABILITY,"version":1}));
    }
    Ok(())
}
