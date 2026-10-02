use super::position::PublishedPosition;
use stagemaster_playback::{LoopPlayback, LoopPosition};
use std::sync::{
    Arc, Mutex, TryLockError,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

#[derive(Clone, Copy, Debug)]
pub struct PerformanceSnapshot {
    pub position: LoopPosition,
    /// Accepted intent, not yet applied at an audio frame boundary.
    pub pending_exit: Option<LoopExitIntent>,
    pub control_problem: Option<&'static str>,
    pub problem: Option<&'static str>,
    pub stopped: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopExitIntent {
    pub region: usize,
    pub pass: u64,
    pub requested: bool,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub(super) enum Failure {
    Underflow = 1,
    Decode = 2,
    Sequence = 3,
    Cursor = 4,
    Cancelled = 5,
    Control = 6,
}

impl Failure {
    fn message(code: u8) -> Option<&'static str> {
        match code {
            0 => None,
            1 => Some("音乐预读不足，播放已停止"),
            2 => Some("音乐解码失败或样本不足，播放已停止"),
            3 => Some("音乐样本与编排位置不一致，播放已停止"),
            4 => Some("音乐循环计数超出范围，播放已停止"),
            5 => Some("音乐播放已取消"),
            _ => Some("音乐运行控制异常，播放已停止"),
        }
    }
}

pub(super) struct Shared {
    position: PublishedPosition,
    pending: Mutex<Option<LoopExitIntent>>,
    control_rejected: AtomicBool,
    failure: AtomicU8,
    pub stopped: AtomicBool,
    pub cancelled: Arc<AtomicBool>,
}

impl Shared {
    pub fn new(position: LoopPosition) -> Arc<Self> {
        Arc::new(Self {
            position: PublishedPosition::new(position),
            pending: Mutex::new(None),
            control_rejected: AtomicBool::new(false),
            failure: AtomicU8::new(0),
            stopped: AtomicBool::new(false),
            cancelled: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn publish(&self, position: LoopPosition) {
        self.position.publish(position);
    }

    pub fn fail(&self, failure: Failure) {
        self.failure.store(failure as u8, Ordering::Release);
    }

    pub fn apply(&self, playback: &mut LoopPlayback) -> Result<(), Failure> {
        match self.pending.try_lock() {
            Ok(mut pending) => {
                if let Some(intent) = pending.take() {
                    // Reject stale control without constructing the core's diagnostic String
                    // in the audio callback. Only the already-validated target reaches it.
                    let position = playback.position();
                    let rejected = position.region != Some(intent.region)
                        || position.pass != Some(intent.pass);
                    if !rejected {
                        playback
                            .set_exit_at_end(intent.region, intent.requested)
                            .map_err(|_| Failure::Control)?;
                    }
                    self.control_rejected.store(rejected, Ordering::Release);
                    self.publish(playback.position());
                }
            }
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Poisoned(_)) => return Err(Failure::Control),
        }
        Ok(())
    }
}

/// Handle belongs to exactly one source; hosts must also validate their playback-instance ID.
#[derive(Clone)]
pub struct PerformanceControl {
    pub(super) shared: Arc<Shared>,
}

impl PerformanceControl {
    /// # Errors
    /// Reject a busy snapshot or a poisoned control state.
    pub fn snapshot(&self) -> Result<PerformanceSnapshot, String> {
        let pending = self
            .shared
            .pending
            .lock()
            .map_err(|_| "音乐运行控制不可用")?;
        Ok(PerformanceSnapshot {
            position: self.shared.position.read()?,
            pending_exit: *pending,
            control_problem: self
                .shared
                .control_rejected
                .load(Ordering::Acquire)
                .then_some("当前循环或播放遍数已变化，之前的退出操作未执行"),
            problem: Failure::message(self.shared.failure.load(Ordering::Acquire)),
            stopped: self.shared.stopped.load(Ordering::Acquire),
        })
    }

    /// Last desired state wins until applied; cancelling a pending exit is supported.
    /// # Errors
    /// Reject a stopped/failed source or a target different from the current region/pass.
    pub fn request_exit(&self, region: usize, pass: u64, requested: bool) -> Result<(), String> {
        let mut pending = self
            .shared
            .pending
            .lock()
            .map_err(|_| "音乐运行控制不可用")?;
        let position = self.shared.position.read()?;
        if self.shared.stopped.load(Ordering::Acquire)
            || self.shared.cancelled.load(Ordering::Acquire)
            || self.shared.failure.load(Ordering::Acquire) != 0
            || position.region != Some(region)
            || position.pass != Some(pass)
        {
            return Err("当前循环或播放遍数已变化，请重新确认退出目标".into());
        }
        *pending = Some(LoopExitIntent {
            region,
            pass,
            requested,
        });
        self.shared.control_rejected.store(false, Ordering::Release);
        Ok(())
    }

    pub fn cancel(&self) {
        self.shared.cancelled.store(true, Ordering::Release);
    }
}
