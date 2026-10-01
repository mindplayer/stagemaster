use crate::looping::{LoopBuffer, LoopRange, LoopRequest, PreparedLoop};
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
    pub loop_range: Option<LoopRange>,
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
    loop_buffer: Option<LoopBuffer>,
    revision: u64,
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
        self.revision = self.revision.wrapping_add(1);
        self.loop_buffer = None;
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
            if let Some(buffer) = &self.loop_buffer {
                buffer.position(self.base_ms, p.get_pos())
            } else if p.empty() {
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
            loop_range: self.loop_buffer.as_ref().map(LoopBuffer::range),
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
        self.revision = self.revision.wrapping_add(1);
        self.base_ms = self.position().position_ms;
        self.player = None;
    }
    pub fn stop(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.player = None;
        self.base_ms = self.loop_buffer.as_ref().map_or(0, |b| b.range().start_ms);
    }
    /// # Errors
    /// Reject missing files/devices and unsupported seeks. A failure never starts another output route.
    pub fn play(&mut self) -> Result<(), String> {
        self.revision = self.revision.wrapping_add(1);
        if self.position().playing {
            return Ok(());
        }
        if self.base_ms >= self.duration_ms || self.player.as_ref().is_some_and(Player::empty) {
            self.base_ms = 0;
        }
        let source: Box<dyn Source + Send> = if let Some(buffer) = &self.loop_buffer {
            Box::new(buffer.source(self.base_ms))
        } else {
            let file = File::open(self.file.as_ref().ok_or("请先准备音乐")?)
                .map_err(|e| format!("无法打开音乐：{e}"))?;
            let mut source = Decoder::try_from(file).map_err(|e| format!("音乐解码失败：{e}"))?;
            source
                .try_seek(Duration::from_millis(self.in_ms + self.base_ms))
                .map_err(|e| format!("音乐定位失败：{e}"))?;
            Box::new(source.take_duration(Duration::from_millis(self.duration_ms - self.base_ms)))
        };
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
        player.append(source);
        self.player = Some(player);
        Ok(())
    }
    /// # Errors
    /// Reject positions outside the clip; decoder failure leaves audition paused.
    pub fn seek(&mut self, position_ms: u64) -> Result<(), String> {
        if position_ms > self.duration_ms {
            return Err("播放位置超出音乐范围".into());
        }
        self.revision = self.revision.wrapping_add(1);
        let playing = self.position().playing;
        if self
            .loop_buffer
            .as_ref()
            .is_some_and(|b| !b.range().contains(position_ms))
        {
            self.loop_buffer = None;
        }
        self.player = None;
        self.base_ms = position_ms;
        if playing && position_ms < self.duration_ms {
            self.play()?;
        }
        Ok(())
    }
    /// # Errors
    /// Reject configuring while playing or an invalid range.
    pub fn loop_request(&self, range: Option<LoopRange>) -> Result<LoopRequest, String> {
        if self.position().playing {
            return Err("请暂停音乐后调整循环范围".into());
        }
        if let Some(value) = range {
            value.validate(self.duration_ms)?;
        }
        Ok(LoopRequest {
            revision: self.revision,
            file: self.file.clone().ok_or("请先准备音乐")?,
            in_ms: self.in_ms,
            range,
        })
    }
    /// # Errors
    /// Reject stale preparation without replacing the previous range or cursor.
    pub fn apply_loop(&mut self, prepared: PreparedLoop) -> Result<(), String> {
        if prepared.revision != self.revision
            || self.position().playing
            || self.file.as_ref() != Some(&prepared.file)
            || self.in_ms != prepared.in_ms
        {
            return Err("播放状态已变化，请重新设置循环".into());
        }
        let position = self.position().position_ms;
        self.player = None;
        self.base_ms = prepared.buffer.as_ref().map_or(position, |b| {
            if b.range().contains(position) {
                position
            } else {
                b.range().start_ms
            }
        });
        self.loop_buffer = prepared.buffer;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
}
