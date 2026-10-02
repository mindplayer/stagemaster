use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, mixer::Mixer};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(super) struct Output {
    mixer: Mixer,
    _device: Option<MixerDeviceSink>,
    failed: Arc<AtomicBool>,
}

impl Output {
    pub fn open() -> Result<Self, String> {
        let failed = Arc::new(AtomicBool::new(false));
        let signal = failed.clone();
        let mut device = DeviceSinkBuilder::from_default_device()
            .map_err(|e| format!("无法打开系统音频输出：{e}"))?
            .with_error_callback(move |_| signal.store(true, Ordering::Release))
            .open_stream()
            .map_err(|e| format!("无法打开系统音频输出：{e}"))?;
        device.log_on_drop(false);
        Ok(Self {
            mixer: device.mixer().clone(),
            _device: Some(device),
            failed,
        })
    }

    pub fn failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }

    pub fn player(&self, volume: u8) -> Player {
        let player = Player::connect_new(&self.mixer);
        player.pause();
        player.set_volume(f32::from(volume) / 100.0);
        player
    }

    #[cfg(test)]
    pub fn virtual_device() -> (Self, rodio::mixer::MixerSource) {
        let (mixer, source) = rodio::mixer::mixer(2.try_into().unwrap(), 8_000.try_into().unwrap());
        (
            Self {
                mixer,
                _device: None,
                failed: Arc::new(AtomicBool::new(false)),
            },
            source,
        )
    }
}
