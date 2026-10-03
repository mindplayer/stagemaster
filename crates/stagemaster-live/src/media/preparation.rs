use super::{GroupKey, Prepared};
use crate::{Command, PlaybackSelection, Session, player::Player};
use stagemaster_engine::live::Layout;
use stagemaster_playback::MAX_TIME_MS;
use stagemaster_project::Document;

/// Immutable preparation capability, separable from the running Session and its thread.
/// Keep one per group; callers bound concurrent jobs and dispose results off the scheduler.
pub struct Preparer {
    boot: [u8; 16],
    index: usize,
    layout: Layout,
    members: Vec<(usize, PlaybackSelection)>,
}
impl Session {
    /// Export immutable preparation metadata before moving the session to its scheduling owner.
    /// The returned object neither holds a session lock nor reads ongoing playback state.
    /// # Errors
    /// Reject wrong execution/group keys or missing prepared sources.
    pub fn media_preparer(&self, key: GroupKey) -> Result<Preparer, String> {
        let index = self.media_index(key)?;
        let members = self.media[index]
            .members
            .iter()
            .map(|&source| {
                let selection = self.sources[source]
                    .selection
                    .clone()
                    .ok_or("同步组缺少来源计划")?;
                Ok((source, selection))
            })
            .collect::<Result<_, String>>()?;
        Ok(Preparer {
            boot: self.boot,
            index,
            layout: self.mixer.layout().clone(),
            members,
        })
    }
}
impl Preparer {
    /// Compile a start/seek replacement off the scheduler, without any running Session access.
    /// Use the current group key from an authoritative observation. Activation checks its age.
    /// # Errors
    /// Reject other groups/projects, invalid positions/deadlines or compilation failure.
    pub fn prepare(
        &self,
        key: GroupKey,
        doc: &Document,
        position_ms: u64,
        playing: bool,
        deadline_ms: u64,
    ) -> Result<Prepared, String> {
        if key.boot != self.boot || key.index != self.index {
            return Err("准备器不属于当前同步组".into());
        }
        if position_ms > MAX_TIME_MS || deadline_ms == 0 {
            return Err("同步组准备位置或截止时间无效".into());
        }
        let mut players = Vec::with_capacity(self.members.len());
        for (source, selection) in &self.members {
            let mut player = Player::prepare(doc, selection, 0)?;
            if player.layout() != &self.layout {
                return Err("同步组准备使用了不同的工程版本".into());
            }
            player.apply_timeline(Command::Execute(0), 0)?;
            player.apply_timeline(Command::Advance, position_ms)?;
            if !playing {
                player.apply_timeline(Command::Pause, position_ms)?;
            }
            players.push((*source, player));
        }
        Ok(Prepared {
            key,
            deadline_ms,
            position_ms,
            playing,
            players,
            restarted_provider: None,
            looping: self
                .members
                .iter()
                .all(|(_, s)| matches!(s, PlaybackSelection::AudioTimeline))
                && doc
                    .audio_timeline()
                    .is_some_and(|t| t.loop_regions.iter().any(|r| r.enabled)),
        })
    }
}
