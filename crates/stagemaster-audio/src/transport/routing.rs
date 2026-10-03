use super::{OutputBinding, Transport, output::Output};
use crate::OutputScope;
use rodio::Player;

impl Transport {
    /// Configure cooperative output ownership before mounting a voice. Survives clear/load.
    /// # Errors
    /// Reject changing a route with a mounted output. Prepared requests become stale.
    pub fn set_output_scope(&mut self, scope: OutputScope) -> Result<(), String> {
        if self.output.is_some() || self.player.is_some() || self.output_lease.is_some() {
            return Err("请先停止音乐再更换声音输出占用范围".into());
        }
        self.revision = self.revision.wrapping_add(1);
        self.output_scope = Some(scope);
        Ok(())
    }

    pub(super) fn output_failed(&self) -> bool {
        self.output.as_ref().is_some_and(Output::failed)
            || self.binding.as_ref().is_some_and(OutputBinding::failed)
    }

    pub(super) fn new_player(&mut self) -> Result<Player, String> {
        // A failed first mount must drop its temporary reservation, not block every retry.
        let lease = if self.output_lease.is_none() {
            self.output_scope
                .as_ref()
                .map(OutputScope::reserve)
                .transpose()?
        } else {
            None
        };
        if self.output.as_ref().is_none_or(Output::failed) {
            self.output = Some(match &self.binding {
                Some(binding) => Output::bound(binding)?,
                None => Output::open()?,
            });
        }
        if lease.is_some() {
            self.output_lease = lease;
        }
        Ok(self
            .output
            .as_ref()
            .ok_or("音频设备未就绪")?
            .player(self.volume_percent.unwrap_or(100)))
    }

    pub(super) fn release_output(&mut self) {
        self.player = None;
        self.output = None;
        self.output_lease = None;
    }
}
