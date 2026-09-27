//! Desktop-only clock and transport adapter for the independent playback engine.
use serde::{Deserialize, Serialize};
use stagemaster_playback::{Player, Status};
use stagemaster_project::{CompiledSequence, Document};
use std::time::Instant;

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Request {
    Snapshot,
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
    sequence_id: String,
    scene_id: Option<String>,
    name: String,
    source_revision: String,
    status: &'static str,
    step_id: Option<String>,
    elapsed_ms: u64,
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
    player: Player,
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
    pub(crate) fn render_output(&mut self, version: u64, now: u64) -> Result<RenderOutput, String> {
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
        loaded.player.advance(now)?;
        Ok(RenderOutput {
            status: status_name(loaded.player.status()),
            output: Some(loaded.output.render(loaded.player.values())?),
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
        let player = Player::new(plan, self.now());
        self.clear();
        self.loaded = Some(Loaded {
            player,
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
        match command {
            Command::Execute { step_id } => {
                let index = loaded
                    .steps
                    .iter()
                    .position(|s| s.id == step_id)
                    .ok_or("此步骤不在已载入的预览中")?;
                loaded.player.execute(index, now)?;
            }
            Command::Next => loaded.player.next(now)?,
            Command::Pause => loaded.player.pause(now)?,
            Command::Resume => loaded.player.resume(now)?,
            Command::Stop => loaded.player.stop(now)?,
        }
        self.last_serial = serial;
        Ok(())
    }
    pub(crate) fn snapshot(&mut self, content_version: u64, now: u64) -> Result<Snapshot, String> {
        let loaded = if let Some(l) = &mut self.loaded {
            l.player.advance(now)?;
            let index = l.player.index();
            let step = index.map(|i| &l.player.plan().steps()[i]);
            Some(LoadedView {
                sequence_id: l.sequence_id.clone(),
                scene_id: l.scene_id.clone(),
                name: l.name.clone(),
                source_revision: l.source_revision.clone(),
                status: status_name(l.player.status()),
                step_id: index.map(|i| l.steps[i].id.clone()),
                elapsed_ms: l.player.elapsed_ms(),
                delay_ms: step.map_or(0, |s| s.delay_ms),
                fade_ms: step.map_or(0, |s| s.fade_ms),
                wait_ms: step.and_then(|s| s.wait_ms),
                stale: l.content_version != content_version,
                can_next: l.player.can_next(),
                buffer_bytes: l.player.plan().value_buffer_bytes(),
                effect_buffer_bytes: l.player.plan().effect_buffer_bytes(),
                steps: l.steps.clone(),
                output: l.output.render(l.player.values())?,
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
fn status_name(status: Status) -> &'static str {
    match status {
        Status::Idle => "idle",
        Status::Running => "running",
        Status::Paused => "paused",
        Status::Finished => "finished",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn document() -> Document {
        let mut input: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        input["entryPoints"] = serde_json::json!([]);
        Document::decode(&serde_json::to_vec(&input).unwrap()).unwrap()
    }
    #[test]
    fn scene_effect_renderer_and_monitor_share_clock_serial_and_epoch() {
        let mut doc = document();
        let view = doc.view();
        let scene = &view.scenes[0].id;
        let fixture = &view.fixtures[0].id;
        doc.edit(
            serde_json::from_value(
                serde_json::json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{
            "id":"29999999-0000-4000-8000-000000000001","name":"测试呼吸","enabled":true,
            "fixtureIds":[fixture],"periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,
            "reverse":false,"waveform":"triangle","dutyPercent":50,
            "channels":[{"attribute":"dimmer","low":0,"high":65535}]}}}),
            )
            .unwrap(),
        )
        .unwrap();
        let mut p = Preview::default();
        p.load_scene(&doc, 1, scene).unwrap();
        let now = p.now();
        let epoch = p.epoch;
        p.control(
            1,
            epoch,
            1,
            Command::Execute {
                step_id: scene.clone(),
            },
            now,
        )
        .unwrap();
        let rendered =
            serde_json::to_value(p.render_output(1, now + 250).unwrap().output.unwrap()).unwrap();
        let monitored = p.snapshot(1, now + 250).unwrap();
        assert_eq!(monitored.control_serial, 1);
        let loaded = monitored.loaded.unwrap();
        assert_eq!(loaded.scene_id.as_ref(), Some(scene));
        assert!(loaded.sequence_id.is_empty());
        assert_eq!(rendered, serde_json::to_value(loaded.output).unwrap());
        p.control(1, epoch, 2, Command::Pause, now + 250).unwrap();
        p.control(1, epoch, 3, Command::Resume, now + 10_000)
            .unwrap();
        let frame = p.snapshot(1, now + 10_250).unwrap();
        assert_eq!(frame.loaded.unwrap().elapsed_ms, 500);
        p.load(&doc, 1, &view.sequences[0].id).unwrap();
        assert!(p.control(1, epoch, 4, Command::Stop, p.now()).is_err());
        let frame = p.snapshot(1, p.now()).unwrap();
        assert_eq!(frame.control_serial, 0);
        assert!(frame.loaded.unwrap().scene_id.is_none());
    }
    #[test]
    fn renderer_polling_or_disconnect_cannot_own_the_show_clock() {
        let doc = document();
        let id = doc.view().sequences[0].id.clone();
        let mut observed = Preview::default();
        let mut disconnected = Preview::default();
        observed.load(&doc, 1, &id).unwrap();
        disconnected.load(&doc, 1, &id).unwrap();
        let now = observed.now().max(disconnected.now());
        observed
            .control(1, observed.epoch, 1, Command::Next, now)
            .unwrap();
        disconnected
            .control(1, disconnected.epoch, 1, Command::Next, now)
            .unwrap();
        for tick in [10, 100, 500, 1100] {
            observed.render_output(1, now + tick).unwrap();
        }
        let a = serde_json::to_value(observed.snapshot(1, now + 1500).unwrap()).unwrap();
        let b = serde_json::to_value(disconnected.snapshot(1, now + 1500).unwrap()).unwrap();
        assert_eq!(a, b);
        // A render-version failure also cannot reset, stop or advance the player independently.
        assert!(
            observed
                .render_output(2, now + 2000)
                .unwrap()
                .output
                .is_none()
        );
        let a = serde_json::to_value(observed.snapshot(1, now + 2500).unwrap()).unwrap();
        let b = serde_json::to_value(disconnected.snapshot(1, now + 2500).unwrap()).unwrap();
        assert_eq!(a, b);
    }
    #[test]
    fn stale_plans_reject_new_execution_but_allow_pause_and_release() {
        let doc = document();
        let id = doc.view().sequences[0].id.clone();
        let mut p = Preview::default();
        p.load(&doc, 3, &id).unwrap();
        let now = p.now();
        let step = p.loaded.as_ref().unwrap().steps[0].id.clone();
        p.control(
            3,
            p.epoch,
            1,
            Command::Execute {
                step_id: step.clone(),
            },
            now,
        )
        .unwrap();
        assert!(
            p.control(4, p.epoch, 2, Command::Execute { step_id: step }, now + 500)
                .is_err()
        );
        p.control(4, p.epoch, 3, Command::Pause, now + 750).unwrap();
        let snapshot = p.snapshot(4, now + 1000).unwrap().loaded.unwrap();
        assert!(snapshot.stale);
        assert_eq!(snapshot.status, "paused");
        assert_eq!(snapshot.elapsed_ms, 750);
        assert!(
            p.control(4, p.epoch, 4, Command::Resume, now + 1000)
                .is_err()
        );
        p.control(4, p.epoch, 5, Command::Stop, now + 1000).unwrap();
        assert_eq!(
            p.snapshot(4, now + 1000)
                .unwrap()
                .loaded
                .unwrap()
                .output
                .slots[0],
            0
        );
    }
    #[test]
    fn repeated_and_wrong_epoch_controls_are_rejected_and_failed_load_preserves_plan() {
        let doc = document();
        let id = doc.view().sequences[0].id.clone();
        let mut p = Preview::default();
        p.load(&doc, 1, &id).unwrap();
        let now = p.now();
        let epoch = p.epoch;
        p.control(1, epoch, 1, Command::Next, now).unwrap();
        assert!(p.control(1, epoch, 1, Command::Next, now).is_err());
        assert_eq!(p.loaded.as_ref().unwrap().player.index(), Some(0));
        assert!(p.load(&doc, 1, "missing").is_err());
        assert_eq!(p.epoch, epoch);
        assert_eq!(p.loaded.as_ref().unwrap().player.index(), Some(0));
        p.load(&doc, 1, &id).unwrap();
        assert!(p.control(1, epoch, 2, Command::Next, p.now()).is_err());
        p.clear();
        assert!(p.snapshot(1, p.now()).unwrap().loaded.is_none());
    }
}
