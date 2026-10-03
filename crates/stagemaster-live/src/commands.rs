use crate::{Change, Command, Key, Session};
use stagemaster_engine::live::MAX_ATTRIBUTES;

impl Session {
    /// Apply a playback command after advancing all automatic boundaries to the same time.
    /// Stop also clears a manual layer. Pause/finish hold ownership; stop releases only this source.
    /// # Errors
    /// Input preflight is atomic. Internal execution failures invalidate the complete frame.
    pub fn control(&mut self, key: Key, command: Command, now_ms: u64) -> Result<(), String> {
        self.ready(now_ms)?;
        let index = self.index(key)?;
        if self.sources[index].media_group.is_some() {
            return Err("媒体跟随来源须通过所属同步组控制".into());
        }
        match (&self.sources[index].player, command) {
            (Some(p), Command::Execute(i)) if i >= p.step_count() => {
                return Err("所选步骤不存在".into());
            }
            (None, c) if c != Command::Stop => return Err("手动层不支持场景列表命令".into()),
            _ => {}
        }
        self.tick(now_ms)?;
        let entry = &mut self.sources[index];
        let result = if let Some(player) = &mut entry.player {
            player.apply(command, now_ms, &self.mixer, entry.handle)
        } else {
            entry.values.fill(None);
            entry.times.fill(None);
            Ok(())
        }
        .and_then(|()| self.compose(now_ms));
        self.finish(result)
    }

    /// Sparse explicit manual edits; each Some reasserts even if its numeric value is unchanged.
    /// Values must already be translated/validated by the trusted fixture semantic adapter.
    /// # Errors
    /// Reject wrong source, duplicate/out-of-range attributes or backwards time before advancing.
    pub fn patch(&mut self, key: Key, changes: &[Change], now_ms: u64) -> Result<(), String> {
        self.ready(now_ms)?;
        let index = self.index(key)?;
        let entry = &self.sources[index];
        if entry.player.is_some() {
            return Err("只有手动层可以接收属性修改".into());
        }
        if changes.len() > entry.values.len() {
            return Err("手动修改数量超出属性范围".into());
        }
        let mut used = [false; MAX_ATTRIBUTES];
        for change in changes {
            if change.attribute >= entry.values.len() || used[change.attribute] {
                return Err("手动属性不存在或重复".into());
            }
            used[change.attribute] = true;
        }
        self.tick(now_ms)?;
        let entry = &mut self.sources[index];
        for change in changes {
            entry.values[change.attribute] = change.value;
            entry.times[change.attribute] = change.value.map(|_| now_ms);
        }
        let result = self.compose(now_ms);
        self.finish(result)
    }

    /// Ordinary source fader scales intensity only; zero is not release or an LTP reassertion.
    /// # Errors
    /// Reject wrong instance or backwards time. Internal failures invalidate the frame.
    pub fn set_level(&mut self, key: Key, level: u16, now_ms: u64) -> Result<(), String> {
        self.ready(now_ms)?;
        let index = self.index(key)?;
        self.tick(now_ms)?;
        let result = self
            .change_level(index, level)
            .and_then(|()| self.compose(now_ms));
        self.finish(result)
    }
    fn change_level(&mut self, index: usize, level: u16) -> Result<(), String> {
        let entry = &mut self.sources[index];
        let serial = entry.next_serial()?;
        self.mixer
            .set_level(entry.handle, serial, level)
            .map_err(|e| e.to_string())?;
        entry.level = level;
        Ok(())
    }
}
