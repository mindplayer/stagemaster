use super::{OutputKind, Runner, job::Failure, output::SoftwareOutput};
use stagemaster_audio::Transport;
use stagemaster_live::media::GroupKey;
use stagemaster_live_host::media::{ControlRequest, LocalClock};
use stagemaster_time::Clock;
use std::{path::PathBuf, time::Instant};

pub(super) struct Reload {
    pub path: PathBuf,
    pub output: OutputKind,
}
impl Runner {
    pub(super) fn recover(
        &mut self,
        request: ControlRequest,
        key: GroupKey,
        position: u64,
    ) -> Result<(), Failure> {
        let state = self
            .state()?
            .ok_or_else(|| "后台状态暂不可用".to_string())?;
        let previous = self.media(&state)?.group.provider;
        let epoch = previous
            .epoch()
            .checked_add(1)
            .ok_or_else(|| "媒体时钟重启代次已耗尽".to_string())?;
        let provider = Clock::new(previous.id(), epoch).map_err(|e| e.to_string())?;
        let target = self
            .mapping
            .map(Instant::now())
            .map_err(|e| e.to_string())?
            .1
            .target();
        let mapping = LocalClock::new(self.clock, provider, target, 5_000_000_000)
            .map_err(|e| e.to_string())?;
        // Dispose the old decoder, cached loop and output before allocating the replacement.
        self.transport.clear();
        self.transport = Transport::default();
        self.software = None;
        let track = self
            .doc
            .audio_timeline()
            .ok_or_else(|| "工程没有音乐轨道".to_string())?;
        stagemaster_audio::verify(&self.reload.path, &track.asset.digest, &self.cancel)
            .map_err(|_| "后台音乐资源校验失败，请关闭后台后从原工程重新载入".to_string())?;
        let (mut transport, software) = SoftwareOutput::prepare(self.reload.output)?;
        let prepared = transport
            .load_performance_request(
                self.reload.path.clone(),
                track.in_ms,
                track.out_ms,
                Some(track.compile_loops(1000)?.schedule),
            )?
            .prepare(&self.cancel)?;
        self.guard(request)?;
        transport.apply_load(prepared)?;
        self.transport = transport;
        self.software = software;
        self.mapping = mapping;
        self.failed = false;
        if position == 0 {
            // apply_load already prepared a new, paused zero-position voice. Do not decode it twice.
            self.activate_prepared(request, key, Some(provider))?;
        } else {
            self.prepare_at(request, key, position, Some(provider))?;
        }
        Ok(())
    }
}
