use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use serde::Serialize;
use std::{
    fs::File,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
#[derive(Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub problem: Option<String>,
    pub volume_percent: u8,
}
/// Single local audition voice. Position is software consumption, not calibrated DAC time.
#[derive(Default)]
pub struct Transport {
    player: Option<Player>,
    stream: Option<MixerDeviceSink>,
    failed: Arc<AtomicBool>,
    file: Option<PathBuf>,
    in_ms: u64,
    duration_ms: u64,
    base_ms: u64,
    volume_percent: Option<u8>,
}
impl Transport {
    /// # Errors
    /// Reject an empty, reversed or excessive source range.
    pub fn load(&mut self, file: PathBuf, in_ms: u64, out_ms: u64) -> Result<(), String> {
        if in_ms >= out_ms || out_ms > crate::MAX_DURATION_MS {
            return Err("音乐范围无效".into());
        }
        self.clear();
        self.file = Some(file);
        self.in_ms = in_ms;
        self.duration_ms = out_ms - in_ms;
        Ok(())
    }
    pub fn clear(&mut self) {
        self.player = None;
        self.stream = None;
        self.file = None;
        self.base_ms = 0;
        self.duration_ms = 0;
        self.failed.store(false, Ordering::Relaxed);
    }
    #[must_use]
    pub fn position(&self) -> Position {
        let position = self.player.as_ref().map_or(self.base_ms, |p| {
            if p.empty() {
                self.duration_ms
            } else {
                self.base_ms
                    .saturating_add(u64::try_from(p.get_pos().as_millis()).unwrap_or(u64::MAX))
                    .min(self.duration_ms)
            }
        });
        let failed = self.failed.load(Ordering::Relaxed);
        Position {
            playing: !failed && self.player.as_ref().is_some_and(|p| !p.empty()),
            position_ms: position,
            duration_ms: self.duration_ms,
            volume_percent: self.volume_percent.unwrap_or(100),
            problem: failed.then(|| "音频输出中断，请检查系统输出设备后重新播放".into()),
        }
    }
    /// # Errors
    /// Reject volume values above 100 percent.
    pub fn set_volume(&mut self, percent: u8) -> Result<(), String> {
        if percent > 100 {
            return Err("试听音量必须在 0–100% 之间".into());
        }
        self.volume_percent = Some(percent);
        if let Some(player) = &self.player {
            player.set_volume(f32::from(percent) / 100.0);
        }
        Ok(())
    }
    pub fn pause(&mut self) {
        self.base_ms = self.position().position_ms;
        self.player = None;
    }
    pub fn stop(&mut self) {
        self.player = None;
        self.base_ms = 0;
    }
    /// # Errors
    /// Reject missing files/devices and unsupported seeks. A failure never starts another output route.
    pub fn play(&mut self) -> Result<(), String> {
        if self.position().playing {
            return Ok(());
        }
        if self.base_ms >= self.duration_ms || self.player.as_ref().is_some_and(Player::empty) {
            self.base_ms = 0;
        }
        let file = File::open(self.file.as_ref().ok_or("请先准备音乐")?)
            .map_err(|e| format!("无法打开音乐：{e}"))?;
        let mut source = Decoder::try_from(file).map_err(|e| format!("音乐解码失败：{e}"))?;
        source
            .try_seek(Duration::from_millis(self.in_ms + self.base_ms))
            .map_err(|e| format!("音乐定位失败：{e}"))?;
        if self.failed.swap(false, Ordering::Relaxed) {
            self.stream = None;
        }
        if self.stream.is_none() {
            let failed = self.failed.clone();
            let mut stream = DeviceSinkBuilder::from_default_device()
                .map_err(|e| format!("无法打开系统音频输出：{e}"))?
                .with_error_callback(move |_| {
                    failed.store(true, Ordering::Relaxed);
                })
                .open_stream()
                .map_err(|e| format!("无法打开系统音频输出：{e}"))?;
            stream.log_on_drop(false);
            self.stream = Some(stream);
        }
        let player = Player::connect_new(self.stream.as_ref().ok_or("音频设备未就绪")?.mixer());
        player.set_volume(f32::from(self.volume_percent.unwrap_or(100)) / 100.0);
        player.append(source.take_duration(Duration::from_millis(self.duration_ms - self.base_ms)));
        self.player = Some(player);
        Ok(())
    }
    /// # Errors
    /// Reject positions outside the clip; decoder failure leaves audition paused.
    pub fn seek(&mut self, position_ms: u64) -> Result<(), String> {
        if position_ms > self.duration_ms {
            return Err("播放位置超出音乐范围".into());
        }
        let playing = self.position().playing;
        self.player = None;
        self.base_ms = position_ms;
        if playing && position_ms < self.duration_ms {
            self.play()?;
        }
        Ok(())
    }
}
