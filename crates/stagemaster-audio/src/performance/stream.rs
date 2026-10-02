use super::{control::Failure, decode, prepare::Data};
use std::{
    ops::Range,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

const BLOCK_FRAMES: usize = 1_024;
const QUEUE_BLOCKS: usize = 128;
const WARM_BLOCKS: usize = 8;
static WORKERS: AtomicUsize = AtomicUsize::new(0);

struct Permit;
impl Permit {
    fn take() -> Result<Self, String> {
        WORKERS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < 2).then_some(n + 1)
            })
            .map(|_| Self)
            .map_err(|_| "上一条音乐解码仍在收尾，请稍后重试".into())
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        WORKERS.fetch_sub(1, Ordering::AcqRel);
    }
}

struct Block {
    start: u64,
    frames: usize,
    samples: [f32; BLOCK_FRAMES * 2],
}

pub(super) struct Feed {
    receiver: Option<Receiver<Block>>,
    current: Option<Block>,
    failed: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
}

impl Feed {
    pub fn prepare(
        data: Arc<Data>,
        start: u64,
        cancelled: Arc<AtomicBool>,
        external: &AtomicBool,
    ) -> Result<Self, String> {
        let started = Instant::now();
        decode::checkpoint(external, started)?;
        let segments = segments(&data, start);
        let (sender, receiver) = mpsc::sync_channel(QUEUE_BLOCKS);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let failed = Arc::new(AtomicBool::new(false));
        if !segments.is_empty() {
            let permit = Permit::take()?;
            let worker_cancel = cancelled.clone();
            let worker_failed = failed.clone();
            thread::Builder::new()
                .name("stage-audio-decode".into())
                .spawn(move || {
                    let _permit = permit;
                    let result = produce(&data, &segments, &sender, &ready_tx, &worker_cancel);
                    if let Err(error) = result {
                        worker_failed.store(true, Ordering::Release);
                        let _ = ready_tx.try_send(Err(error));
                    }
                })
                .map_err(|e| format!("无法启动音乐解码：{e}"))?;
            loop {
                if let Err(error) = decode::checkpoint(external, started) {
                    cancelled.store(true, Ordering::Release);
                    return Err(error);
                }
                match ready_rx.recv_timeout(Duration::from_millis(10)) {
                    Ok(result) => {
                        result?;
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        return Err("音乐解码准备意外结束".into());
                    }
                }
            }
        }
        Ok(Self {
            receiver: Some(receiver),
            current: None,
            failed,
            cancelled,
        })
    }

    pub fn frame(&mut self, tick: u64, channels: usize) -> Result<[f32; 2], Failure> {
        if self.failed.load(Ordering::Acquire) {
            return Err(Failure::Decode);
        }
        if self
            .current
            .as_ref()
            .is_none_or(|b| tick == b.start + b.frames as u64)
        {
            self.current = Some(
                self.receiver
                    .as_ref()
                    .ok_or(Failure::Cancelled)?
                    .try_recv()
                    .map_err(|error| match error {
                        mpsc::TryRecvError::Empty => Failure::Underflow,
                        mpsc::TryRecvError::Disconnected => Failure::Decode,
                    })?,
            );
        }
        let block = self.current.as_ref().ok_or(Failure::Sequence)?;
        let offset = tick
            .checked_sub(block.start)
            .and_then(|v| usize::try_from(v).ok())
            .filter(|&v| v < block.frames)
            .ok_or(Failure::Sequence)?;
        let mut frame = [0.0; 2];
        frame[..channels]
            .copy_from_slice(&block.samples[offset * channels..(offset + 1) * channels]);
        if offset + 1 == block.frames {
            self.current = None;
        }
        Ok(frame)
    }

    pub fn cancel(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        self.receiver = None;
    }

    pub fn has_failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }
}

impl Drop for Feed {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn segments(data: &Data, start: u64) -> Vec<Range<u64>> {
    let mut result = Vec::new();
    let mut at = start;
    for (region, cache) in data.schedule.regions().iter().zip(&data.cached) {
        if cache.is_none() || region.end <= at {
            continue;
        }
        if region.start > at {
            result.push(at..region.start);
        }
        at = region.end;
    }
    if at < data.schedule.duration() {
        result.push(at..data.schedule.duration());
    }
    result
}

fn produce(
    data: &Data,
    segments: &[Range<u64>],
    sender: &SyncSender<Block>,
    ready: &SyncSender<Result<(), String>>,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let mut decoder = decode::open(&data.path)?;
    let format = data.format;
    let channels = usize::from(format.channels.get());
    let mut absolute = 0;
    let mut sent = 0;
    let started = Instant::now();
    for segment in segments {
        while absolute < data.source_start + segment.start {
            if absolute % 1_024 == 0 {
                check_worker(cancel, started, sent)?;
            }
            format.frame(&mut decoder)?;
            absolute += 1;
        }
        let mut at = segment.start;
        while at < segment.end {
            check_worker(cancel, started, sent)?;
            let frames = usize::try_from((segment.end - at).min(BLOCK_FRAMES as u64))
                .map_err(|_| "音乐预读块超出范围")?;
            let mut block = Block {
                start: at,
                frames,
                samples: [0.0; BLOCK_FRAMES * 2],
            };
            for samples in block.samples[..frames * channels].chunks_exact_mut(channels) {
                let frame = format.frame(&mut decoder)?;
                samples.copy_from_slice(&frame[..channels]);
            }
            sender.send(block).map_err(|_| "音乐音源已释放")?;
            absolute += frames as u64;
            at += frames as u64;
            sent += 1;
            if sent == WARM_BLOCKS {
                let _ = ready.try_send(Ok(()));
            }
        }
    }
    if sent < WARM_BLOCKS {
        let _ = ready.try_send(Ok(()));
    }
    Ok(())
}

fn check_worker(cancel: &AtomicBool, started: Instant, sent: usize) -> Result<(), String> {
    if sent < WARM_BLOCKS {
        return decode::checkpoint(cancel, started);
    }
    if cancel.load(Ordering::Acquire) {
        return Err("音乐播放已取消".into());
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn worker_count() -> usize {
    WORKERS.load(Ordering::Acquire)
}

#[cfg(test)]
#[path = "stream_tests.rs"]
mod tests;
