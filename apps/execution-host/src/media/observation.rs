use super::Runner;
use stagemaster_audio::PerformanceObservation;
use stagemaster_live::media::{Sample, Status};
use stagemaster_live_host::{
    State,
    media::{MediaState, Termination},
};
use stagemaster_runtime::Code;
use stagemaster_time::Mapping;
use std::sync::atomic::Ordering;

impl Runner {
    pub(super) fn state(&self) -> Result<Option<State>, String> {
        let Ok(observed) = self.observer.read() else {
            return Ok(None);
        };
        if observed.phase != stagemaster_runtime_host::Phase::Running {
            return Err("灯光执行宿主已停止".into());
        }
        Ok(observed.snapshot.map(|s| s.state))
    }
    pub(super) fn media(&self, state: &State) -> Result<MediaState, String> {
        state
            .media
            .iter()
            .flatten()
            .find(|m| m.group.id == self.port.initial().id)
            .copied()
            .ok_or("同步组不存在".into())
    }
    pub(super) fn native(&self) -> Result<Option<PerformanceObservation>, String> {
        match self.transport.performance_observation() {
            Err(e) if e.contains("正在更新") || e.contains("忙") => Ok(None),
            value => value,
        }
    }
    pub(super) fn sample(
        &self,
        native: PerformanceObservation,
    ) -> Result<Option<(Sample, Mapping)>, String> {
        if let Some(problem) = native.snapshot.problem {
            return Err(problem.into());
        }
        let Some(render) = native.snapshot.render else {
            return Ok(None);
        };
        let (at, mapping) = self.mapping.map(render.at).map_err(|e| e.to_string())?;
        Ok(Some((
            Sample {
                at,
                sequence: render.sequence,
                position_ms: native.snapshot.position.tick * 1000 / u64::from(render.sample_rate),
                playing: render.applied.playing,
                progress: Some(stagemaster_live::media::MediaProgress {
                    instance: native.instance,
                    sample_rate: render.sample_rate,
                    position_ticks: native.snapshot.position.tick,
                    consumed_ticks: native.snapshot.consumption.map_or(0, |c| c.frames),
                    repeated_ticks: native.snapshot.position.repeated_ticks,
                }),
            },
            mapping,
        )))
    }
    pub(super) fn pull(&mut self) -> Result<(), String> {
        if self.failed {
            return Ok(());
        }
        if let Some(output) = &mut self.software {
            output.pull()?;
        }
        if let Some(native) = self.native()? {
            let position = self.transport.position();
            let loop_state = self.loop_state(native);
            if let Ok(mut view) = self.view.lock() {
                view.position_ms = position.position_ms;
                view.instance = Some(native.instance.to_string());
                view.loop_state = loop_state;
                view.frames = native
                    .snapshot
                    .consumption
                    .map_or(0, |c| c.frames)
                    .to_string();
            }
        }
        Ok(())
    }
    pub(super) fn tick(&mut self) -> Result<(), String> {
        if let Some((key, reason)) = self.pending_end {
            self.finish(key, reason)?;
            return Ok(());
        }
        self.pull()?;
        if self.staged.is_some() {
            self.cancel_job();
            if self.staged.is_some() {
                return Ok(());
            }
        }
        let Some(state) = self.state()? else {
            return Ok(());
        };
        let media = self.media(&state)?;
        if let Some(serial) = self.terminal {
            if media.termination.is_some_and(|r| r.serial == serial) {
                self.terminal = None;
            } else {
                return Ok(());
            }
        }
        if self.observe_control(media)? {
            return Ok(());
        }
        let Some(key) = self.active else {
            return Ok(());
        };
        if media.group.key != key || media.group.status == Status::Lost {
            return Err("媒体同步组已失效，音源已停止，请重新准备播放".into());
        }
        let Some(native) = self.native()? else {
            return Ok(());
        };
        if let Some(problem) = native.snapshot.problem {
            return Err(problem.into());
        }
        if native.snapshot.position.ended {
            self.finish(key, Termination::Ended)?;
            if let Ok(mut view) = self.view.lock() {
                view.status = "ended";
                view.position_ms = view.duration_ms;
            }
            return Ok(());
        }
        if native.snapshot.stopped {
            return Err("音源已意外停止".into());
        }
        if self.pending_sample.is_none() {
            self.pending_sample = self.sample(native)?;
        }
        if let Some((sample, mapping)) = self.pending_sample
            && state.observed_ms * 1_000_000
                >= mapping
                    .convert(sample.at)
                    .map_err(|e| e.to_string())?
                    .latest()
                    .nanos
        {
            if sample.sequence > self.last_sample {
                match self.port.publish(key, sample, mapping) {
                    Ok(_) => {}
                    Err(Code::Busy) => return Ok(()),
                    Err(e) => return Err(e.to_string()),
                }
                self.last_sample = sample.sequence;
            }
            self.pending_sample = None;
        }
        Ok(())
    }
    pub(super) fn finish(
        &mut self,
        key: stagemaster_live::media::GroupKey,
        reason: Termination,
    ) -> Result<(), String> {
        self.transport.stop();
        self.pending_end = Some((key, reason));
        let serial = match self.port.terminate(key, reason) {
            Ok(serial) => serial,
            Err(Code::Busy) => return Ok(()),
            Err(e) => return Err(e.to_string()),
        };
        self.pending_end = None;
        self.terminal = Some(serial);
        self.active = None;
        self.pending_sample = None;
        Ok(())
    }
    pub(super) fn fail(&mut self, problem: String) {
        self.failed = true;
        self.cancel_job();
        self.transport.stop();
        if let Some(output) = &mut self.software {
            output.reset_clock();
        }
        if let Some(request) = self.request.take() {
            let _ = self.port.complete_control(request.ticket, Err(Code::Read));
        }
        if let Some(key) = self.active {
            let _ = self.finish(key, Termination::Failed(Code::Read));
        }
        if let Ok(mut view) = self.view.lock() {
            view.status = "failed";
            view.loop_state = None;
            view.problem = Some(problem);
        }
        if self.state().is_err() {
            self.cancel.store(true, Ordering::Release);
        }
    }
    pub(super) fn cancel_job(&mut self) {
        if let Some(staged) = self.staged.take() {
            match self.port.reclaim(staged, true) {
                Err(Code::Busy) => self.staged = Some(staged),
                Ok(reclaimed) if reclaimed.result == Some(Ok(())) => {
                    if let Ok(Some(state)) = self.state()
                        && let Ok(media) = self.media(&state)
                        && reclaimed.prepared.key().generation().checked_add(1)
                            == Some(media.group.key.generation())
                    {
                        self.active = Some(media.group.key);
                        self.last_sample = media.group.sequence;
                    }
                }
                _ => {}
            }
        }
        self.transport.pause();
        self.pending_sample = None;
    }
}
