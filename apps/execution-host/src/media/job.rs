use super::Runner;
use stagemaster_live::media::{GroupKey, Sample};
use stagemaster_live_host::media::{ControlRequest, MediaCommand};
use stagemaster_runtime::Code;
use stagemaster_time::Mapping;
use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

pub(super) enum Failure {
    Superseded,
    Rejected(Code),
    Problem(String),
}
impl From<String> for Failure {
    fn from(value: String) -> Self {
        Self::Problem(value)
    }
}
impl Runner {
    pub(super) fn guard(&mut self, request: ControlRequest) -> Result<(), Failure> {
        loop {
            if self.cancel.load(Ordering::Acquire) {
                return Err(Failure::Superseded);
            }
            if self
                .clock
                .at_ms(Instant::now())
                .map_err(|e| e.to_string())?
                >= request.deadline_ms
            {
                return Err(
                    if matches!(request.command, MediaCommand::ExitLoop { .. }) {
                        // An expired loop intent must not stop the already-running source.
                        Failure::Superseded
                    } else {
                        Failure::Problem("音乐操作准备超时".into())
                    },
                );
            }
            match self.port.control_state() {
                Ok(Some(current)) if current.request == request && current.result.is_none() => {
                    return Ok(());
                }
                Err(Code::Busy) => {
                    self.pull()?;
                    thread::sleep(Duration::from_millis(5));
                }
                _ => return Err(Failure::Superseded),
            }
        }
    }
    pub(super) fn wait<T>(
        &mut self,
        request: ControlRequest,
        mut read: impl FnMut(&mut Self) -> Result<Option<T>, String>,
    ) -> Result<T, Failure> {
        loop {
            self.guard(request)?;
            self.pull()?;
            if let Some(value) = read(self)? {
                return Ok(value);
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn confirmed_sample(
        &mut self,
        request: ControlRequest,
        minimum_sequence: u64,
    ) -> Result<(Sample, Mapping), Failure> {
        let (sample, mapping) = self.wait(request, |r| {
            let Some(native) = r.native()? else {
                return Ok(None);
            };
            if native.snapshot.stopped || native.snapshot.position.ended {
                return Err("音源已结束，无法确认当前播放操作".into());
            }
            if native.snapshot.render.is_none_or(|render| {
                render.applied != native.request || render.sequence <= minimum_sequence
            }) {
                return Ok(None);
            }
            r.sample(native)
        })?;
        let nanos = mapping
            .convert(sample.at)
            .map_err(|e| e.to_string())?
            .latest()
            .nanos;
        self.wait(request, |r| {
            Ok(r.state()?.filter(|s| s.observed_ms * 1_000_000 >= nanos))
        })?;
        Ok((sample, mapping))
    }
    pub(super) fn prepare_at(
        &mut self,
        request: ControlRequest,
        key: GroupKey,
        position: u64,
        restarted: Option<stagemaster_time::Clock>,
    ) -> Result<(), Failure> {
        self.transport.pause();
        let source = self
            .transport
            .seek_preparation(position, false)?
            .ok_or_else(|| "后台正式音源未准备".to_string())?
            .prepare(&self.cancel)?;
        self.guard(request)?;
        self.transport.apply_seek(source)?;
        self.activate_prepared(request, key, restarted)
    }
    pub(super) fn activate_prepared(
        &mut self,
        request: ControlRequest,
        key: GroupKey,
        restarted: Option<stagemaster_time::Clock>,
    ) -> Result<(), Failure> {
        self.guard(request)?;
        self.transport.prime_performance()?;
        let (actual, _) = self.confirmed_sample(request, 0)?;
        let mut prepared = self.prepare.prepare(
            key,
            &self.doc,
            actual.position_ms,
            false,
            request.deadline_ms,
        )?;
        if let Some(provider) = restarted {
            prepared = prepared.with_restarted_provider(provider)?;
        }
        let (sample, mapping) = self.confirmed_sample(request, actual.sequence)?;
        self.guard(request)?;
        let activation = self
            .port
            .stage_requested(request.ticket, prepared, sample, mapping)
            .map_err(|e| e.to_string())?;
        self.staged = Some(activation);
        let result = self.wait(request, |r| match r.port.reclaim(activation, false) {
            Ok(result) => Ok(Some(result)),
            Err(Code::Busy) => Ok(None),
            Err(e) => Err(e.to_string()),
        })?;
        self.staged = None;
        result
            .result
            .ok_or_else(|| "媒体准备未激活".to_string())?
            .map_err(|e| e.to_string())?;
        let next = self.wait(request, |r| {
            Ok(r.state()?
                .and_then(|s| r.media(&s).ok())
                .filter(|m| m.group.key != key))
        })?;
        self.active = Some(next.group.key);
        self.last_sample = sample.sequence;
        self.pending_sample = None;
        Ok(())
    }
    pub(super) fn execute(
        &mut self,
        request: ControlRequest,
        key: GroupKey,
    ) -> Result<(), Failure> {
        self.guard(request)?;
        if self.failed && !matches!(request.command, MediaCommand::Recover { .. }) {
            return Err(Failure::Problem("音源需要重新准备后才能操作".into()));
        }
        match request.command {
            MediaCommand::ExitLoop { .. } => self.exit_loop(request)?,
            MediaCommand::Recover { position_ms } => self.recover(request, key, position_ms)?,
            command if self.is_end_seek(command) => self.seek_end(request)?,
            MediaCommand::Stop => {
                self.transport.stop();
                self.active = None;
                self.pending_sample = None;
            }
            command => {
                let position = self.transport.position();
                let resume = self.active == Some(key)
                    && position.problem.is_none()
                    && !position.performance.as_ref().is_some_and(|p| p.ended);
                let playing = match command {
                    MediaCommand::Play => {
                        if !resume {
                            self.prepare_at(request, key, 0, None)?;
                        }
                        true
                    }
                    MediaCommand::Pause => {
                        if !resume {
                            return Err(Failure::Problem("当前音源未处于可暂停状态".into()));
                        }
                        false
                    }
                    MediaCommand::Seek {
                        position_ms,
                        playing,
                    } => {
                        self.prepare_at(request, key, position_ms, None)?;
                        playing
                    }
                    MediaCommand::Stop
                    | MediaCommand::ExitLoop { .. }
                    | MediaCommand::Recover { .. } => unreachable!(),
                };
                self.guard(request)?;
                if playing {
                    self.transport.play()?;
                } else {
                    self.transport.pause();
                }
                let (sample, mapping) = self.confirmed_sample(request, self.last_sample)?;
                let active = self.active.ok_or_else(|| "媒体组尚未激活".to_string())?;
                let serial =
                    self.wait(request, |r| match r.port.publish(active, sample, mapping) {
                        Ok(serial) => Ok(Some(serial)),
                        Err(Code::Busy) => Ok(None),
                        Err(e) => Err(e.to_string()),
                    })?;
                let receipt = self.wait(request, |r| {
                    Ok(r.state()?
                        .and_then(|s| r.media(&s).ok())
                        .and_then(|m| m.observation)
                        .filter(|r| r.serial == serial))
                })?;
                receipt.result.map_err(|e| e.to_string())?;
                self.last_sample = sample.sequence;
            }
        }
        self.guard(request)?;
        self.complete_request(request, Ok(()))?;
        // Completion is asynchronous. Keep the provider free to process the next explicit intent.
        if let Ok(mut view) = self.view.lock() {
            view.status = if matches!(request.command, MediaCommand::Stop) {
                view.position_ms = 0;
                view.loop_state = None;
                "stopped"
            } else if self.is_end_seek(request.command) {
                view.position_ms = view.duration_ms;
                "ended"
            } else if self.transport.position().playing {
                "playing"
            } else {
                "paused"
            };
        }
        Ok(())
    }
}
