use rodio::mixer::MixerSource;
use serde::{Deserialize, Serialize};
use stagemaster_audio::{OutputBinding, Transport};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum OutputKind {
    SystemDefault,
    Software,
}

pub(super) struct SoftwareOutput {
    cancel: Arc<AtomicBool>,
    problem: Arc<Mutex<Option<String>>>,
    thread: Option<JoinHandle<()>>,
    binding: OutputBinding,
}
struct Consumer {
    source: MixerSource,
    origin: Instant,
    frames: u64,
}
impl SoftwareOutput {
    pub fn prepare(kind: OutputKind) -> Result<(Transport, Option<Self>), String> {
        match kind {
            OutputKind::SystemDefault => Ok((Transport::default(), None)),
            OutputKind::Software => {
                let (mixer, source) = rodio::mixer::mixer(
                    2.try_into().expect("two channels"),
                    48_000.try_into().expect("valid rate"),
                );
                let binding = OutputBinding::new(mixer);
                let output = Self::start(
                    Consumer {
                        source,
                        origin: Instant::now(),
                        frames: 0,
                    },
                    binding.clone(),
                )?;
                Ok((Transport::with_output(binding), Some(output)))
            }
        }
    }
    fn start(mut consumer: Consumer, binding: OutputBinding) -> Result<Self, String> {
        let cancel = Arc::new(AtomicBool::new(false));
        let problem = Arc::new(Mutex::new(None));
        let stopped = cancel.clone();
        let failure = problem.clone();
        let output = binding.clone();
        let thread = thread::Builder::new()
            .name("stage-software-audio".into())
            .spawn(move || {
                while !stopped.load(Ordering::Acquire) {
                    if let Err(error) = consumer.pull() {
                        output.report_failure();
                        if let Ok(mut slot) = failure.lock() {
                            *slot = Some(error);
                        }
                        break;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
            })
            .map_err(|e| format!("无法启动软件音频消费：{e}"))?;
        Ok(Self {
            cancel,
            problem,
            thread: Some(thread),
            binding,
        })
    }
    pub fn check(&self) -> Result<(), String> {
        let problem = self.problem.lock().map_err(|_| "软件音频故障状态不可用")?;
        if let Some(problem) = &*problem {
            return Err(problem.clone());
        }
        if self.thread.as_ref().is_none_or(JoinHandle::is_finished) {
            self.binding.report_failure();
            return Err("软件音频消费线程已停止".into());
        }
        Ok(())
    }
}
impl Drop for SoftwareOutput {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.binding.report_failure();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Consumer {
    fn pull(&mut self) -> Result<(), String> {
        self.pull_elapsed(self.origin.elapsed())
    }
    fn pull_elapsed(&mut self, elapsed: Duration) -> Result<(), String> {
        let due = u64::try_from(elapsed.as_nanos() * 48_000 / 1_000_000_000)
            .map_err(|_| "软件音频时钟耗尽")?;
        let count = due.saturating_sub(self.frames);
        if count > 24_000 {
            return Err("软件音频消费超过 500 毫秒未运行".into());
        }
        for _ in 0..count * 2 {
            // Like a device callback, an empty mixer renders silence and remains available.
            // No source frame or health observation is invented while it has no queued voice.
            let _ = self.source.next();
        }
        self.frames = due;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/software_output.rs"]
mod tests;
