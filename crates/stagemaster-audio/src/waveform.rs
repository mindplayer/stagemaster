use rodio::{Decoder, Source};
use serde::Serialize;
use std::{
    fs::File,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
pub const MAX_DURATION_MS: u64 = 3_600_000;
pub const BUCKET_MS: u64 = 10;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Waveform {
    pub duration_ms: u64,
    pub bucket_ms: u64,
    /// One interleaved min/max envelope per channel. Never decoded PCM.
    pub channels: Vec<Vec<f32>>,
}
/// Stream decoded samples into bounded peak buckets, never retain the full PCM.
/// # Errors
/// Reject invalid streams, excess duration/channels/rate, cancellation or excessive decode time.
pub fn analyze(path: &Path, cancelled: &AtomicBool) -> Result<Waveform, String> {
    let mut source = Decoder::try_from(File::open(path).map_err(|e| format!("无法读取音乐：{e}"))?)
        .map_err(|e| format!("无法解析音乐：{e}"))?;
    let rate = source.sample_rate().get();
    let channels = source.channels().get();
    if channels > 2 || !(8_000..=192_000).contains(&rate) {
        return Err("当前支持单声道／双声道、8–192 kHz 的音乐".into());
    }
    if source
        .total_duration()
        .is_some_and(|d| d.as_millis() > u128::from(MAX_DURATION_MS))
    {
        return Err("音乐最长支持 1 小时，请先裁切源文件".into());
    }
    let per_second = u64::from(rate) * u64::from(channels);
    let samples_per_bucket = per_second * BUCKET_MS / 1000;
    let maximum = per_second * MAX_DURATION_MS / 1000;
    let mut envelopes = Vec::with_capacity(usize::from(channels));
    for _ in 0..channels {
        let mut envelope = Vec::new();
        envelope
            .try_reserve_exact(720_000)
            .map_err(|_| "波形缓存内存不足")?;
        envelopes.push(envelope);
    }
    let mut count = 0_u64;
    let mut low = [f32::INFINITY; 2];
    let mut high = [f32::NEG_INFINITY; 2];
    let begin = Instant::now();
    while let Some(sample) = source.next() {
        if count.is_multiple_of(samples_per_bucket) {
            if cancelled.load(Ordering::Relaxed) {
                return Err("音乐准备已取消".into());
            }
            if begin.elapsed().as_secs() >= 120 {
                return Err("音乐解析超时，请使用较短或重新导出的文件".into());
            }
            if source.sample_rate().get() != rate || source.channels().get() != channels {
                return Err("暂不支持中途改变采样率或声道数的音乐".into());
            }
        }
        if count >= maximum {
            return Err("音乐最长支持 1 小时".into());
        }
        if !sample.is_finite() {
            return Err("音乐包含无效采样值".into());
        }
        let channel = usize::try_from(count % u64::from(channels)).unwrap_or(0);
        count += 1;
        low[channel] = low[channel].min(sample.clamp(-1.0, 1.0));
        high[channel] = high[channel].max(sample.clamp(-1.0, 1.0));
        // Bucket boundaries use source frames so 11.025/22.05 kHz never split a stereo frame.
        if count.is_multiple_of(u64::from(channels))
            && (count * 1000 / per_second) / BUCKET_MS > envelopes[0].len() as u64 / 2
        {
            for (channel, envelope) in envelopes.iter_mut().enumerate() {
                envelope.extend([low[channel], high[channel]]);
            }
            low = [f32::INFINITY; 2];
            high = [f32::NEG_INFINITY; 2];
        }
    }
    if count == 0 {
        return Err("音乐没有可播放的采样".into());
    }
    if !count.is_multiple_of(u64::from(channels)) {
        return Err("音乐末尾的采样帧不完整".into());
    }
    if low[0].is_finite() {
        for (channel, envelope) in envelopes.iter_mut().enumerate() {
            envelope.extend([low[channel], high[channel]]);
        }
    }
    Ok(Waveform {
        duration_ms: (count * 1000 / per_second).max(1),
        bucket_ms: BUCKET_MS,
        channels: envelopes,
    })
}
