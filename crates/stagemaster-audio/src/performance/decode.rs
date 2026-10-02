use rodio::{ChannelCount, Decoder, SampleRate, Source};
use std::{
    fs::File,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub(super) struct Format {
    pub channels: ChannelCount,
    pub rate: SampleRate,
}

impl Format {
    pub fn read(source: &impl Source) -> Result<Self, String> {
        let channels = source.channels();
        let rate = source.sample_rate();
        if channels.get() > 2 || !(8_000..=192_000).contains(&rate.get()) {
            return Err("演出音乐须为单／双声道，采样率须在 8–192 kHz 之间".into());
        }
        Ok(Self { channels, rate })
    }

    pub fn frames(self, ms: u64) -> Result<u64, String> {
        ms.checked_mul(u64::from(self.rate.get()))
            .map(|value| value / 1_000)
            .ok_or_else(|| "音频范围超出帧计数上限".into())
    }

    pub fn frame(self, source: &mut impl Source) -> Result<[f32; 2], String> {
        let mut frame = [0.0; 2];
        for sample in frame.iter_mut().take(usize::from(self.channels.get())) {
            *sample = source.next().ok_or("音乐实际样本少于编排范围")?;
            if !sample.is_finite()
                || source.channels() != self.channels
                || source.sample_rate() != self.rate
            {
                return Err("音乐样本无效或中途改变了声道／采样率".into());
            }
        }
        Ok(frame)
    }
}

pub(super) fn open(path: &Path) -> Result<Decoder<std::io::BufReader<File>>, String> {
    let file = File::open(path).map_err(|e| format!("无法读取演出音乐：{e}"))?;
    if file
        .metadata()
        .map_err(|e| format!("无法检查音乐文件：{e}"))?
        .len()
        > crate::MAX_FILE_BYTES
    {
        return Err("音乐文件不能超过 512 MiB".into());
    }
    Decoder::try_from(file).map_err(|e| format!("演出音乐解码失败：{e}"))
}

pub(super) fn checkpoint(cancel: &AtomicBool, started: Instant) -> Result<(), String> {
    if cancel.load(Ordering::Acquire) {
        return Err("音乐准备已取消".into());
    }
    if started.elapsed() >= Duration::from_mins(2) {
        return Err("音乐准备超时，请检查文件或缩短范围".into());
    }
    Ok(())
}
