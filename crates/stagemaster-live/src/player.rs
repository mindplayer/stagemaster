use crate::{Command, PlaybackSelection};
use stagemaster_engine::live::{Error, Handle, Layout, LiveMixer};
use stagemaster_playback::Status;
use stagemaster_project::{
    Document, LiveAudioTimeline, LiveOutput, LiveSequencePlayer, LiveSourceBudget,
};

pub(super) enum Player {
    Program(LiveSequencePlayer),
    Audio(LiveAudioTimeline),
}
impl Player {
    pub fn prepare(
        doc: &Document,
        selection: &PlaybackSelection,
        now: u64,
    ) -> Result<Self, String> {
        match selection {
            PlaybackSelection::Program(p) => doc.compile_live_source(p, now).map(Self::Program),
            PlaybackSelection::AudioTimeline => doc.compile_live_audio().map(Self::Audio),
        }
    }
    pub fn layout(&self) -> &Layout {
        match self {
            Self::Program(p) => p.layout(),
            Self::Audio(p) => p.layout(),
        }
    }
    pub fn prepare_output(&self) -> Result<LiveOutput, String> {
        match self {
            Self::Program(p) => p.prepare_output(),
            Self::Audio(p) => p.prepare_output(),
        }
    }
    pub fn status(&self) -> Status {
        match self {
            Self::Program(p) => p.status(),
            Self::Audio(p) => p.status(),
        }
    }
    pub fn index(&self) -> Option<usize> {
        match self {
            Self::Program(p) => p.index(),
            Self::Audio(p) => p.index(),
        }
    }
    pub fn budget(&self) -> LiveSourceBudget {
        match self {
            Self::Program(p) => LiveSourceBudget::from_plan(p.plan()),
            Self::Audio(p) => p.budget(),
        }
    }
    pub fn supports_position(&self, position: u64) -> bool {
        match self {
            Self::Program(_) => position <= stagemaster_playback::MAX_TIME_MS,
            Self::Audio(p) => position <= p.duration_ms(),
        }
    }
    pub fn step_count(&self) -> usize {
        match self {
            Self::Program(p) => p.plan().steps().len(),
            Self::Audio(_) => 0,
        }
    }
    pub const fn is_audio(&self) -> bool {
        matches!(self, Self::Audio(_))
    }
    pub fn continuous(&self) -> bool {
        match self {
            Self::Audio(_) => true,
            Self::Program(p) => {
                let steps = p.plan().steps();
                !steps
                    .iter()
                    .take(steps.len().saturating_sub(1))
                    .any(|s| s.wait_ms.is_none())
            }
        }
    }
    pub fn apply(
        &mut self,
        command: Command,
        now: u64,
        mixer: &LiveMixer,
        handle: Handle,
    ) -> Result<(), String> {
        match self {
            Self::Program(p) => p.apply(command, now, mixer, handle),
            Self::Audio(_) => Err("音乐轨道须通过所属同步组控制".into()),
        }
    }
    pub fn apply_timeline(&mut self, command: Command, position: u64) -> Result<(), String> {
        match self {
            Self::Program(p) => p.apply_timeline(command, position),
            Self::Audio(p) => p.apply_timeline(command, position),
        }
    }
    pub fn copy_contribution(
        &self,
        values: &mut [Option<u16>],
        times: &mut [Option<u64>],
    ) -> Result<(), Error> {
        match self {
            Self::Program(p) => p.copy_contribution(values, times),
            Self::Audio(p) => p.copy_contribution(values, times),
        }
    }
    pub fn acknowledge_contribution(&mut self) {
        match self {
            Self::Program(p) => p.acknowledge_contribution(),
            Self::Audio(p) => p.acknowledge_contribution(),
        }
    }
}
