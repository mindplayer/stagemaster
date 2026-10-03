use super::Transport;
use crate::PerformanceSnapshot;

/// Non-serialized, process-local observation for a trusted execution adapter.
#[derive(Clone, Copy, Debug)]
pub struct PerformanceObservation {
    pub instance: u64,
    /// Host intent, NOT confirmation of audible output or the callback applying a pause.
    pub requested_playing: bool,
    /// Exact source request generation; compare with snapshot.render.applied before confirmation.
    pub request: crate::PlaybackRequest,
    pub snapshot: PerformanceSnapshot,
}

impl Transport {
    /// Observe a prepared performance voice without refreshing cached consumption timestamps.
    /// None means no current performance voice; ordinary audition is not exposed by this API.
    /// Call from the media adapter, never while holding the engine's scheduling ownership.
    /// # Errors
    /// Reject output failure or busy/inconsistent native observations; no cache is substituted.
    pub fn performance_observation(&self) -> Result<Option<PerformanceObservation>, String> {
        let Some(voice) = self.performance.as_ref().and_then(|p| p.voice.as_ref()) else {
            return Ok(None);
        };
        if self.output_failed() {
            return Err("音频输出已中断，消费观测不可用于继续同步".into());
        }
        let snapshot = voice.control.try_snapshot()?;
        let request = voice.control.requested_playback();
        Ok(Some(PerformanceObservation {
            instance: voice.control.instance(),
            requested_playing: request.playing,
            request,
            snapshot,
        }))
    }
}
