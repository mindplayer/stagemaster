use crate::{Command, Key, Session};

/// Same-time controls for ordinary prepared programs; never media or manual layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchCommand {
    Pause,
    Resume,
    Stop,
}
impl BatchCommand {
    const fn command(self) -> Command {
        match self {
            Self::Pause => Command::Pause,
            Self::Resume => Command::Resume,
            Self::Stop => Command::Stop,
        }
    }
}
impl Session {
    /// Preflight every target, advance one authority time, then compose the whole result.
    /// Pausing/resuming inactive programs is a no-op; stop releases only selected contributions.
    /// # Errors
    /// Empty/duplicate/foreign/manual/media targets or >64 keys reject before advancing.
    /// Internal execution failures latch the existing group fault and retract its complete frame.
    pub fn control_batch(
        &mut self,
        keys: &[Key],
        command: BatchCommand,
        now_ms: u64,
    ) -> Result<(), String> {
        self.ready(now_ms)?;
        if keys.is_empty() || keys.len() > 64 {
            return Err("批量节目须为 1—64 项".into());
        }
        let mut seen = [false; 64];
        let mut indices = [0; 64];
        for (slot, &key) in keys.iter().enumerate() {
            let index = self.index(key)?;
            let source = &self.sources[index];
            if seen[index]
                || source.media_group.is_some()
                || source
                    .player
                    .as_ref()
                    .is_none_or(crate::player::Player::is_audio)
            {
                return Err("批量目标重复或不是独立普通节目".into());
            }
            seen[index] = true;
            indices[slot] = index;
        }
        self.tick(now_ms)?;
        let result = indices[..keys.len()]
            .iter()
            .try_for_each(|&index| {
                let entry = &mut self.sources[index];
                entry.player.as_mut().ok_or("批量节目缺少播放器")?.apply(
                    command.command(),
                    now_ms,
                    &self.mixer,
                    entry.handle,
                )
            })
            .and_then(|()| self.compose(now_ms));
        self.finish(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn final_batch_composition_failure_retracts_every_valid_frame() {
        let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        raw["entryPoints"] = serde_json::json!([]);
        let doc =
            stagemaster_project::Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
        let spec = crate::SourceSpec {
            id: [1; 16],
            priority: 0,
            playback: Some(
                stagemaster_project::PackageSelection::Scene {
                    id: doc.view().scenes[0].id.clone(),
                }
                .into(),
            ),
        };
        for source_fault in [false, true] {
            let mut s = Session::prepare(&doc, [9; 16], std::slice::from_ref(&spec), 0).unwrap();
            let key = s.key([1; 16]).unwrap();
            s.control(key, Command::Execute(0), 0).unwrap();
            if source_fault {
                s.sources[0].serial = u64::MAX - 1;
            } else {
                s.sequence = u64::MAX - 1;
            }
            assert!(s.control_batch(&[key], BatchCommand::Stop, 10).is_err());
            assert!(s.fault().is_some());
            assert!(s.frame().is_none() && s.values().is_none() && s.winner(0).is_none());
            assert!(s.control_batch(&[key], BatchCommand::Resume, 11).is_err());
        }
    }
}
