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
    Problem(String),
}
impl From<String> for Failure {
    fn from(value: String) -> Self {
        Self::Problem(value)
    }
}
impl Runner {
    fn guard(&mut self, request: ControlRequest) -> Result<(), Failure> {
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
                return Err(Failure::Problem("音乐操作准备超时".into()));
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
    fn wait<T>(
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
    fn prepare_at(
        &mut self,
        request: ControlRequest,
        key: GroupKey,
        position: u64,
    ) -> Result<(), Failure> {
        self.transport.pause();
        let source = self
            .transport
            .seek_preparation(position, false)?
            .ok_or_else(|| "后台正式音源未准备".to_string())?
            .prepare(&self.cancel)?;
        let prepared =
            self.prepare
                .prepare(key, &self.doc, position, false, request.deadline_ms)?;
        self.guard(request)?;
        self.transport.apply_seek(source)?;
        if let Some(output) = &mut self.software {
            output.reset_clock();
        }
        self.transport.prime_performance()?;
        let (sample, mapping) = self.confirmed_sample(request, 0)?;
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
        match request.command {
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
                            self.prepare_at(request, key, 0)?;
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
                        self.prepare_at(request, key, position_ms)?;
                        playing
                    }
                    MediaCommand::Stop => unreachable!(),
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
        self.wait(request, |r| {
            match r.port.complete_control(request.ticket, Ok(())) {
                Ok(()) => Ok(Some(())),
                Err(Code::Busy) => Ok(None),
                Err(e) => Err(e.to_string()),
            }
        })?;
        // Completion is asynchronous. Keep the provider free to process the next explicit intent.
        if let Ok(mut view) = self.view.lock() {
            view.status = if matches!(request.command, MediaCommand::Stop) {
                view.position_ms = 0;
                "stopped"
            } else if self.transport.position().playing {
                "playing"
            } else {
                "paused"
            };
        }
        Ok(())
    }
}
