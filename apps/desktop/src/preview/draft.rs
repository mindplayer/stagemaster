//! One temporary effect overlay. It never mutates a Document or owns a second clock.
use super::{Loaded, Preview};
use stagemaster_playback::{Player, Status};
use stagemaster_project::{
    CompiledSequence, Document, EditCommand, EffectEdit, SceneEffect, ValueMode,
};

pub(super) struct Draft {
    pub effect_id: String,
    pub baseline: CompiledSequence,
    pub serial: u32,
}

pub(crate) fn compile(
    document: &Document,
    scene_id: &str,
    effect: SceneEffect,
    illuminate: bool,
) -> Result<CompiledSequence, String> {
    let mut commands = Vec::new();
    if illuminate {
        let view = document.view();
        for id in &effect.fixture_ids {
            if view
                .fixtures
                .iter()
                .any(|f| &f.id == id && f.attributes.iter().any(|a| a.key == "dimmer"))
            {
                commands.push(EditCommand::SetSceneValue {
                    scene_id: scene_id.into(),
                    fixture_id: id.clone(),
                    attribute: "dimmer".into(),
                    mode: ValueMode::Literal,
                    value: u16::MAX,
                });
            }
        }
    }
    commands.push(EditCommand::Effect {
        command: EffectEdit::Put {
            scene_id: scene_id.into(),
            effect,
        },
    });
    if commands.len() > 256 {
        return Err("本次修改超过容量，请减少灯具数量".into());
    }
    let mut temporary = document.clone();
    temporary.edit(EditCommand::Batch { commands })?;
    temporary.compile_scene(scene_id)
}

pub(crate) struct Preparation {
    pub document: Document,
    pub generation: u32,
    pub version: u64,
    pub epoch: u32,
    pub scene_id: String,
    pub effect: SceneEffect,
    pub illuminate: bool,
    pub serial: Option<u32>,
}
pub(crate) struct Prepared {
    pub generation: u32,
    version: u64,
    epoch: u32,
    scene_id: String,
    effect_id: String,
    baseline: Option<CompiledSequence>,
    compiled: CompiledSequence,
    serial: Option<u32>,
}
impl Preparation {
    /// Expensive validation and compilation runs without the shared session lock.
    pub(crate) fn compile(self) -> Result<Prepared, String> {
        let baseline = if self.serial.is_none() {
            Some(self.document.compile_scene(&self.scene_id)?)
        } else {
            None
        };
        let effect_id = self.effect.id.clone();
        let compiled = compile(&self.document, &self.scene_id, self.effect, self.illuminate)?;
        Ok(Prepared {
            generation: self.generation,
            version: self.version,
            epoch: self.epoch,
            scene_id: self.scene_id,
            effect_id,
            baseline,
            compiled,
            serial: self.serial,
        })
    }
}
impl Preview {
    pub(crate) fn guard_epoch(&self, epoch: u32) -> Result<(), String> {
        if epoch != self.epoch {
            return Err("预演内容已更换，请重新开启即时预演".into());
        }
        Ok(())
    }
    pub(crate) fn draft_target(
        &self,
        version: u64,
        epoch: u32,
        serial: u32,
        effect_id: &str,
    ) -> Result<String, String> {
        self.guard_epoch(epoch)?;
        let loaded = self.loaded.as_ref().ok_or("即时预演已结束")?;
        let draft = loaded.draft.as_ref().ok_or("当前播放不是效果草稿")?;
        if loaded.content_version != version || draft.effect_id != effect_id {
            return Err("即时预演的编辑目标已变化".into());
        }
        if serial <= draft.serial {
            return Err("过时的效果草稿已忽略".into());
        }
        loaded
            .scene_id
            .clone()
            .ok_or_else(|| "草稿场景不存在".into())
    }
    pub(crate) fn install_draft(
        &mut self,
        prepared: Prepared,
        version: u64,
    ) -> Result<bool, String> {
        self.guard_epoch(prepared.epoch)?;
        if version != prepared.version {
            return Err("工程已修改，请重新开启即时预演".into());
        }
        if let Some(serial) = prepared.serial {
            self.draft_target(version, prepared.epoch, serial, &prepared.effect_id)?;
            let now = self.now();
            let loaded = self.loaded.as_mut().expect("validated draft");
            replace_at_time(loaded, prepared.compiled, now)?;
            loaded.draft.as_mut().expect("validated draft").serial = serial;
            Ok(false)
        } else {
            let baseline = prepared.baseline.ok_or("缺少原场景预演")?;
            self.install(prepared.compiled, version, Some(prepared.scene_id));
            let now = self.now();
            let loaded = self.loaded.as_mut().expect("installed scene");
            loaded.player.execute(0, now)?;
            loaded.draft = Some(Draft {
                effect_id: prepared.effect_id,
                baseline,
                serial: 0,
            });
            Ok(true)
        }
    }
    pub(crate) fn end_draft(&mut self, epoch: u32) -> Result<(), String> {
        // Closing an old editor must never stop the new owner's playback.
        if self.epoch != epoch {
            return Ok(());
        }
        let now = self.now();
        let Some(loaded) = &mut self.loaded else {
            return Ok(());
        };
        let Some(draft) = loaded.draft.take() else {
            return Ok(());
        };
        replace_at_time(loaded, draft.baseline, now)?;
        self.epoch = self.epoch.wrapping_add(1);
        self.last_serial = 0;
        Ok(())
    }
    pub(crate) fn clear_draft(&mut self) {
        if self.loaded.as_ref().is_some_and(|l| l.draft.is_some()) {
            self.clear();
        }
    }
}

fn replace_at_time(
    loaded: &mut Loaded,
    compiled: CompiledSequence,
    now: u64,
) -> Result<(), String> {
    loaded.player.advance(now)?;
    let status = loaded.player.status();
    let elapsed = loaded.player.elapsed_ms();
    let mut player = Player::new(compiled.plan, 0);
    if status == Status::Idle {
        player.advance(now)?;
    } else {
        player.execute(0, 0)?;
        player.advance(elapsed)?;
        player.pause(elapsed)?;
        if status == Status::Running {
            player.resume(now)?;
        } else {
            player.advance(now)?;
        }
    }
    loaded.player = player;
    loaded.output = compiled.output;
    loaded.name = compiled.name;
    loaded.steps = compiled.steps;
    Ok(())
}
