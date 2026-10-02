//! Prepared list contribution: timing stays in Player, ownership stays in the source.
mod compile;
mod ownership;
use crate::{CompiledOutput, CompiledStep, LiveOutput};
use ownership::{Ownership, StepOwnership};
use stagemaster_engine::live::{Error, Frame, Handle, Layout, LiveMixer};
use stagemaster_playback::{Command, Player, Status};

pub struct LiveSequencePlayer {
    player: Player,
    output: CompiledOutput,
    layout: Layout,
    ownership: Ownership,
    values: Vec<Option<u16>>,
    winners: Vec<Option<Handle>>,
    pub id: String,
    pub revision_id: String,
    pub steps: Vec<CompiledStep>,
}
impl LiveSequencePlayer {
    #[must_use]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }
    #[must_use]
    pub const fn status(&self) -> Status {
        self.player.status()
    }
    #[must_use]
    pub const fn index(&self) -> Option<usize> {
        self.player.index()
    }
    #[must_use]
    pub const fn elapsed_ms(&self) -> u64 {
        self.player.elapsed_ms()
    }
    #[must_use]
    pub fn can_next(&self) -> bool {
        self.player.can_next()
    }

    /// Prepare an encoder outside the live path, bound to this exact engineering snapshot.
    /// # Errors
    /// Reject an inconsistent output mapping.
    pub fn prepare_output(&self) -> Result<LiveOutput, String> {
        LiveOutput::prepare(&self.output, &self.layout)
    }

    /// Apply/advance using one current composition as the baseline for newly acquired values.
    /// The host serializes calls and successful publications across sources.
    /// # Errors
    /// Reject mismatched layouts, old handles, invalid steps or backwards time; no mixer mutation occurs.
    pub fn apply(
        &mut self,
        command: Command,
        now_ms: u64,
        mixer: &LiveMixer,
        handle: Handle,
    ) -> Result<(), String> {
        if mixer.layout().id() != self.layout.id() {
            return Err("场景列表与当前合成工程不一致".into());
        }
        let source = mixer.source(handle).map_err(|e| e.to_string())?;
        mixer
            .render(&mut self.ownership.baseline, &mut self.winners)
            .map_err(|e| e.to_string())?;
        for (index, attribute) in self.layout.attributes().iter().enumerate() {
            self.ownership.winning[index] = self.winners[index] == Some(handle);
            if attribute.intensity {
                // The baseline is post-fader; undo this source's attenuation before it is
                // used as an origin, otherwise the compositor would apply the level twice.
                let value = u32::from(self.ownership.baseline[index]);
                let level = u32::from(source.level);
                let origin = (value * u32::from(u16::MAX) + level / 2)
                    .checked_div(level)
                    .unwrap_or(0);
                self.ownership.baseline[index] = u16::try_from(origin).unwrap_or(u16::MAX);
            }
        }
        self.player
            .apply_observed(command, now_ms, &mut self.ownership)
    }

    /// Publish the final sparse state and pending explicit claims. Failure retains all claims.
    /// # Errors
    /// Return handle, layout, serial or shape rejection without changing the mixer.
    pub fn publish(
        &mut self,
        mixer: &mut LiveMixer,
        handle: Handle,
        serial: u64,
    ) -> Result<(), Error> {
        for (index, owned) in self.ownership.owned.iter().enumerate() {
            self.values[index] = owned.then_some(self.player.values()[index]);
        }
        mixer.publish(
            handle,
            Frame {
                layout: self.layout.id(),
                serial,
                values: &self.values,
                assert: &self.ownership.claims,
            },
        )?;
        self.ownership.claims.fill(false);
        Ok(())
    }
}
