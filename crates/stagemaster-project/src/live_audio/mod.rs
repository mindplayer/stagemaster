//! Prepared authored audio lighting, independent of the decoder, clock, UI and storage.
mod budget;
mod compile;
mod sampler;
use crate::{AudioTimeline, CompiledOutput, LiveOutput};
pub use budget::LiveSourceBudget;
use sampler::Sampler;
use stagemaster_engine::live::{Error, Layout};
use stagemaster_playback::{Command, Status};

pub struct LiveAudioTimeline {
    track: AudioTimeline,
    defaults: Sampler,
    segments: Vec<(String, Sampler)>,
    output: CompiledOutput,
    layout: Layout,
    owned: Vec<bool>,
    budget: LiveSourceBudget,
    active: Option<usize>,
    position: u64,
    status: Status,
    claims: Vec<Option<u64>>,
}
impl LiveAudioTimeline {
    #[must_use]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }
    #[must_use]
    pub const fn budget(&self) -> LiveSourceBudget {
        self.budget
    }
    #[must_use]
    pub const fn duration_ms(&self) -> u64 {
        self.track.duration_ms()
    }
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }
    #[must_use]
    pub fn index(&self) -> Option<usize> {
        (self.status != Status::Idle).then_some(self.active.map_or(0, |i| i + 1))
    }
    /// # Errors
    /// Reject an inconsistent prepared output mapping.
    pub fn prepare_output(&self) -> Result<LiveOutput, String> {
        LiveOutput::prepare(&self.output, &self.layout)
    }
    /// Advance only from authoritative media positions; explicit backward seeks prepare a new generation.
    /// # Errors
    /// Reject out-of-range/backwards positions or unsupported commands before changing contribution.
    pub fn apply_timeline(&mut self, command: Command, position: u64) -> Result<(), String> {
        if command == Command::Stop {
            self.status = Status::Idle;
            self.claims.fill(None);
            return Ok(());
        }
        if position > self.track.duration_ms() || position < self.position {
            return Err("音乐灯光位置越界或倒退，须重新准备".into());
        }
        match command {
            Command::Execute(0) => {
                self.sample(position, true)?;
                self.status = Status::Running;
            }
            Command::Advance | Command::Pause => {
                if self.status == Status::Running {
                    self.sample(position, false)?;
                }
                if command == Command::Pause && self.status == Status::Running {
                    self.status = Status::Paused;
                }
            }
            Command::Resume => {
                if self.status == Status::Paused {
                    if position != self.position {
                        return Err("音乐继续位置与暂停位置不一致".into());
                    }
                    self.status = Status::Running;
                }
            }
            _ => return Err("音乐灯光轨道须按媒体位置控制".into()),
        }
        Ok(())
    }
    /// Sample an admitted media loop boundary using the existing immutable plans.
    /// The owning media group validates consumption continuity before calling this method.
    /// # Errors
    /// Reject inactive sources or positions outside the authored track before changing output.
    pub fn repeat_at(&mut self, position: u64) -> Result<(), String> {
        if self.status == Status::Idle || position > self.duration_ms() {
            return Err("音乐循环来源未运行或位置越界".into());
        }
        self.sample(position, true)
    }
    fn sample(&mut self, position: u64, reassert: bool) -> Result<(), String> {
        let reference = self.track.lighting_at(position);
        let origin = reference.map_or(0, |r| r.start_ms);
        let active = reference
            .map(|r| {
                self.segments
                    .iter()
                    .position(|(id, _)| id == r.id)
                    .ok_or("音乐段落未准备")
            })
            .transpose()?;
        let sampler = active.map_or(&mut self.defaults, |i| &mut self.segments[i].1);
        sampler.sample(position - origin)?;
        // Gaps share the default sampler, but crossing an entire enabled clip still
        // changes ownership once. Disabled clips and repeated gap samples do not.
        let crossed_end = self.track.lighting_clips.as_ref().is_some_and(|clips| {
            clips
                .iter()
                .any(|clip| clip.enabled && self.position < clip.end_ms && clip.end_ms <= position)
        });
        if reassert || active != self.active || crossed_end {
            for (index, owned) in self.owned.iter().enumerate() {
                if *owned {
                    self.claims[index] = Some(position);
                }
            }
        }
        self.active = active;
        self.position = position;
        Ok(())
    }
    /// # Errors
    /// Reject destination shape before writing sparse values or pending claims.
    pub fn copy_contribution(
        &self,
        values: &mut [Option<u16>],
        times: &mut [Option<u64>],
    ) -> Result<(), Error> {
        if values.len() != self.owned.len() || times.len() != self.owned.len() {
            return Err(Error::Shape);
        }
        let source = self
            .active
            .map_or(&self.defaults, |i| &self.segments[i].1)
            .values();
        for (index, value) in values.iter_mut().enumerate() {
            *value = (self.status != Status::Idle && self.owned[index]).then_some(source[index]);
        }
        times.copy_from_slice(&self.claims);
        Ok(())
    }
    pub fn acknowledge_contribution(&mut self) {
        self.claims.fill(None);
    }
}
