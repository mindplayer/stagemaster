//! Prepared, snapshot-bound output encoding for the host compositor.
use crate::{CompiledOutput, LiveScenePlayer};
use stagemaster_engine::live::{Handle, Layout, LayoutId, LiveMixer};
use stagemaster_playback::OutputMaster;

pub struct LiveOutput {
    identity: LayoutId,
    output: stagemaster_package::Output,
    values: Vec<u16>,
    winners: Vec<Option<Handle>>,
    intensity: Vec<bool>,
}
impl LiveScenePlayer {
    /// Prepare the encoder and diagnostic buffers outside the live scheduling path.
    /// # Errors
    /// Reject an internally inconsistent output mapping.
    pub fn prepare_output(&self) -> Result<LiveOutput, String> {
        LiveOutput::prepare(&self.output, &self.layout)
    }
}
impl LiveOutput {
    pub(crate) fn prepare(compiled: &CompiledOutput, layout: &Layout) -> Result<Self, String> {
        let output = compiled.portable_output()?;
        Ok(LiveOutput {
            identity: layout.id(),
            values: vec![0; output.mappings.len()],
            winners: vec![None; output.mappings.len()],
            intensity: layout.attributes().iter().map(|a| a.intensity).collect(),
            output,
        })
    }
}
impl LiveOutput {
    /// Compose then encode against the same immutable snapshot; no per-frame allocation.
    /// Returns the logical universe, never physical completion. Forward only on success.
    /// # Errors
    /// Reject a different snapshot or invalid shape before changing output slots.
    pub fn render(&mut self, mixer: &LiveMixer, slots: &mut [u8; 512]) -> Result<u16, String> {
        self.render_with_master(mixer, slots, OutputMaster::default())
    }
    /// Apply final intensity attenuation once, after mixing and before channel encoding.
    /// Ownership and source contributions remain unchanged; no per-frame allocation.
    /// # Errors
    /// Reject a different snapshot or invalid shape before changing output slots.
    pub fn render_with_master(
        &mut self,
        mixer: &LiveMixer,
        slots: &mut [u8; 512],
        master: OutputMaster,
    ) -> Result<u16, String> {
        if mixer.layout().id() != self.identity {
            return Err("输出编码器与当前合成工程不一致".into());
        }
        mixer
            .render(&mut self.values, &mut self.winners)
            .map_err(|e| e.to_string())?;
        for (value, intensity) in self.values.iter_mut().zip(&self.intensity) {
            if *intensity {
                *value = master.scale(*value);
            }
        }
        self.output
            .render(&self.values, slots)
            .map_err(|e| e.to_string())?;
        Ok(self.output.universe)
    }
    /// Historical post-master values from the last successful composition, not lamp feedback.
    #[must_use]
    pub fn values(&self) -> &[u16] {
        &self.values
    }
    #[must_use]
    pub fn winners(&self) -> &[Option<Handle>] {
        &self.winners
    }
}
