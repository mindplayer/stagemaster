use crate::CompiledOutput;
use stagemaster_engine::live::{Error, Frame, Handle, Layout, LiveMixer};
use stagemaster_playback::{Player, Status};

/// Prepared scene source for a trusted host compositor. Not a network or hardware controller.
pub struct LiveScenePlayer {
    pub(crate) player: Player,
    pub(crate) output: CompiledOutput,
    pub(crate) layout: Layout,
    pub(crate) owned: Vec<bool>,
    pub(crate) values: Vec<Option<u16>>,
    pub(crate) claims: Vec<bool>,
    pub(crate) asserting: bool,
    pub id: String,
    pub revision_id: String,
}
impl LiveScenePlayer {
    #[must_use]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }
    #[must_use]
    pub fn output(&self) -> &CompiledOutput {
        &self.output
    }
    #[must_use]
    pub const fn status(&self) -> Status {
        self.player.status()
    }
    /// Execute/reassert this scene without stealing unrelated attributes.
    /// # Errors
    /// Reject backwards time; the source contribution remains unchanged until publish.
    pub fn start(&mut self, now_ms: u64) -> Result<(), String> {
        self.player.execute(0, now_ms)?;
        self.asserting = true;
        Ok(())
    }
    /// # Errors
    /// Reject backwards time without advancing the player.
    pub fn tick(&mut self, now_ms: u64) -> Result<(), String> {
        self.player.advance(now_ms)
    }
    /// # Errors
    /// Reject backwards time. Pausing holds ownership and does not reassert.
    pub fn pause(&mut self, now_ms: u64) -> Result<(), String> {
        self.player.pause(now_ms)
    }
    /// # Errors
    /// Reject backwards time. Resuming continues the same source order.
    pub fn resume(&mut self, now_ms: u64) -> Result<(), String> {
        self.player.resume(now_ms)
    }
    /// # Errors
    /// Reject backwards time. Next successful publish withdraws all scene attributes.
    pub fn stop(&mut self, now_ms: u64) -> Result<(), String> {
        self.player.stop(now_ms)?;
        self.asserting = false;
        Ok(())
    }
    /// Publish values separately from explicit reassertion; failed batches retain the intent.
    /// The host owns command serialization, source lifetime, scheduling and fault handling.
    /// # Errors
    /// Return the mixer's layout, handle, ordering or budget rejection without changing it.
    pub fn publish(
        &mut self,
        mixer: &mut LiveMixer,
        handle: Handle,
        serial: u64,
    ) -> Result<(), Error> {
        let active = self.player.status() != Status::Idle;
        for (index, &owned) in self.owned.iter().enumerate() {
            self.values[index] = if active && owned {
                Some(self.player.values()[index])
            } else {
                None
            };
            self.claims[index] = active && owned && self.asserting;
        }
        mixer.publish(
            handle,
            Frame {
                layout: self.layout.id(),
                serial,
                values: &self.values,
                assert: &self.claims,
            },
        )?;
        self.asserting = false;
        Ok(())
    }
}
