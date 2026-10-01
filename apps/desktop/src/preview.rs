//! Desktop-only clock and transport adapter for the independent playback engine.
use serde::{Deserialize, Serialize};
use stagemaster_playback::{Player, RateClock, Status};
use stagemaster_project::{CompiledSequence, Document};
use std::time::Instant;
mod draft;
pub(crate) use draft::{Preparation, Prepared};

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Request {
    Snapshot,
    BeginEffectDraft {
        generation: u32,
        epoch: u32,
        scene_id: String,
        effect: stagemaster_project::SceneEffect,
        illuminate: bool,
    },
    UpdateEffectDraft {
        generation: u32,
        epoch: u32,
        serial: u32,
        effect: stagemaster_project::SceneEffect,
        illuminate: bool,
    },
    EndEffectDraft {
        epoch: u32,
    },
    Load {
        generation: u32,
        sequence_id: String,
    },
    LoadScene {
        generation: u32,
        scene_id: String,
    },
    Control {
        epoch: u32,
        serial: u32,
        command: Command,
    },
}
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Command {
    Execute { step_id: String },
    Next,
    Pause,
    Resume,
    Stop,
    SetRate { percent: u16 },
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    epoch: u32,
    control_serial: u32,
    loaded: Option<LoadedView>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadedView {
    #[serde(skip_serializing_if = "Option::is_none")]
    draft_effect_id: Option<String>,
    sequence_id: String,
    scene_id: Option<String>,
    name: String,
    source_revision: String,
    status: &'static str,
    step_id: Option<String>,
    elapsed_ms: u64,
    rate_percent: u16,
    delay_ms: u64,
    fade_ms: u64,
    wait_ms: Option<u64>,
    stale: bool,
    can_next: bool,
    buffer_bytes: usize,
    effect_buffer_bytes: usize,
    steps: Vec<stagemaster_project::CompiledStep>,
    output: stagemaster_project::PreviewOutput,
}
struct Loaded {
    draft: Option<draft::Draft>,
    player: Player,
    clock: RateClock,
    output: stagemaster_project::CompiledOutput,
    sequence_id: String,
    scene_id: Option<String>,
    name: String,
    source_revision: String,
    steps: Vec<stagemaster_project::CompiledStep>,
    content_version: u64,
}
pub(crate) struct Preview {
    origin: Instant,
    loaded: Option<Loaded>,
    epoch: u32,
    last_serial: u32,
}
pub(crate) struct RenderOutput {
    pub status: &'static str,
    pub output: Option<stagemaster_project::PreviewOutput>,
}
impl Default for Preview {
    fn default() -> Self {
        Self {
            origin: Instant::now(),
            loaded: None,
            epoch: 0,
            last_serial: 0,
        }
    }
}
impl Preview {
    pub(crate) fn render_output(
        &mut self,
        version: u64,
        now: u64,
        master: stagemaster_playback::OutputMaster,
    ) -> Result<RenderOutput, String> {
        let Some(loaded) = &mut self.loaded else {
            return Ok(RenderOutput {
                status: "unloaded",
                output: None,
            });
        };
        if loaded.content_version != version {
            return Ok(RenderOutput {
                status: "stalePlayback",
                output: None,
            });
        }
        loaded.advance(now)?;
        Ok(RenderOutput {
            status: status_name(loaded.player.status()),
            output: Some(
                loaded
                    .output
                    .render_with_master(loaded.player.values(), master)?,
            ),
        })
    }
    pub(crate) fn now(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
    pub(crate) fn clear(&mut self) {
        self.loaded = None;
        self.epoch = self.epoch.wrapping_add(1);
        self.last_serial = 0;
    }
    pub(crate) fn load(
        &mut self,
        document: &Document,
        content_version: u64,
        id: &str,
    ) -> Result<(), String> {
        self.install(document.compile_sequence(id)?, content_version, None);
        Ok(())
    }
    pub(crate) fn load_scene(
        &mut self,
        document: &Document,
        content_version: u64,
        id: &str,
    ) -> Result<(), String> {
        self.install(
            document.compile_scene(id)?,
            content_version,
            Some(id.into()),
        );
        Ok(())
    }
    fn install(
        &mut self,
        compiled: CompiledSequence,
        content_version: u64,
        scene_id: Option<String>,
    ) {
        let CompiledSequence {
            plan,
            output,
            id,
            name,
            revision_id,
            steps,
        } = compiled;
        let now = self.now();
        let player = Player::new(plan, now);
        self.clear();
        self.loaded = Some(Loaded {
            draft: None,
            player,
            clock: RateClock::new(now),
            output,
            sequence_id: if scene_id.is_some() {
                String::new()
            } else {
                id
            },
            scene_id,
            name,
            source_revision: revision_id,
            steps,
            content_version,
        });
    }
    pub(crate) fn control(
        &mut self,
        content_version: u64,
        epoch: u32,
        serial: u32,
        command: Command,
        now: u64,
    ) -> Result<(), String> {
        if epoch != self.epoch {
            return Err("预览计划已更换，请重试".into());
        }
        if serial <= self.last_serial {
            return Err("重复或乱序的预览操作已忽略".into());
        }
        let loaded = self.loaded.as_mut().ok_or("请先载入一个场景列表")?;
        if loaded.content_version != content_version
            && !matches!(command, Command::Stop | Command::Pause)
        {
            return Err("工程已修改，请重新载入预览后执行".into());
        }
        let mut clock = loaded.clock;
        let time = match command {
            Command::SetRate { percent } => clock.set_rate(now, percent)?,
            _ => clock.advance(now)?,
        };
        match command {
            Command::Execute { step_id } => {
                let index = loaded
                    .steps
                    .iter()
                    .position(|s| s.id == step_id)
                    .ok_or("此步骤不在已载入的预览中")?;
                loaded.player.execute(index, time)?;
            }
            Command::Next => loaded.player.next(time)?,
            Command::Pause => loaded.player.pause(time)?,
            Command::Resume => loaded.player.resume(time)?,
            Command::Stop => loaded.player.stop(time)?,
            Command::SetRate { .. } => loaded.player.advance(time)?,
        }
        loaded.clock = clock;
        self.last_serial = serial;
        Ok(())
    }
    pub(crate) fn snapshot(
        &mut self,
        content_version: u64,
        now: u64,
        master: stagemaster_playback::OutputMaster,
    ) -> Result<Snapshot, String> {
        let loaded = if let Some(l) = &mut self.loaded {
            l.advance(now)?;
            let index = l.player.index();
            let step = index.map(|i| &l.player.plan().steps()[i]);
            Some(LoadedView {
                draft_effect_id: l.draft.as_ref().map(|d| d.effect_id.clone()),
                sequence_id: l.sequence_id.clone(),
                scene_id: l.scene_id.clone(),
                name: l.name.clone(),
                source_revision: l.source_revision.clone(),
                status: status_name(l.player.status()),
                step_id: index.map(|i| l.steps[i].id.clone()),
                elapsed_ms: l.player.elapsed_ms(),
                rate_percent: l.clock.percent(),
                delay_ms: step.map_or(0, |s| s.delay_ms),
                fade_ms: step.map_or(0, |s| s.fade_ms),
                wait_ms: step.and_then(|s| s.wait_ms),
                stale: l.content_version != content_version,
                can_next: l.player.can_next(),
                buffer_bytes: l.player.plan().value_buffer_bytes(),
                effect_buffer_bytes: l.player.plan().effect_buffer_bytes(),
                steps: l.steps.clone(),
                output: l.output.render_with_master(l.player.values(), master)?,
            })
        } else {
            None
        };
        Ok(Snapshot {
            epoch: self.epoch,
            control_serial: self.last_serial,
            loaded,
        })
    }
}
impl Loaded {
    fn advance(&mut self, now: u64) -> Result<u64, String> {
        let mut clock = self.clock;
        let time = clock.advance(now)?;
        self.player.advance(time)?;
        self.clock = clock;
        Ok(time)
    }
}
fn status_name(status: Status) -> &'static str {
    match status {
        Status::Idle => "idle",
        Status::Running => "running",
        Status::Paused => "paused",
        Status::Finished => "finished",
    }
}

#[cfg(test)]
mod draft_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod rate_tests;
