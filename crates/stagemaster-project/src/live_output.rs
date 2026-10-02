//! Prepared, snapshot-bound output encoding for the host compositor.
use crate::LiveScenePlayer;
use stagemaster_engine::live::{Handle, LayoutId, LiveMixer};

pub struct LiveOutput {
    identity: LayoutId,
    output: stagemaster_package::Output,
    values: Vec<u16>,
    winners: Vec<Option<Handle>>,
}
impl LiveScenePlayer {
    /// Prepare the encoder and diagnostic buffers outside the live scheduling path.
    /// # Errors
    /// Reject an internally inconsistent output mapping.
    pub fn prepare_output(&self) -> Result<LiveOutput, String> {
        let output = self.output.portable_output()?;
        Ok(LiveOutput {
            identity: self.layout.id(),
            values: vec![0; output.mappings.len()],
            winners: vec![None; output.mappings.len()],
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
        if mixer.layout().id() != self.identity {
            return Err("输出编码器与当前合成工程不一致".into());
        }
        mixer
            .render(&mut self.values, &mut self.winners)
            .map_err(|e| e.to_string())?;
        self.output
            .render(&self.values, slots)
            .map_err(|e| e.to_string())?;
        Ok(self.output.universe)
    }
    /// Historical values from the last successful composition, not physical lamp feedback.
    #[must_use]
    pub fn values(&self) -> &[u16] {
        &self.values
    }
    #[must_use]
    pub fn winners(&self) -> &[Option<Handle>] {
        &self.winners
    }
}
