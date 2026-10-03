use rodio::mixer::MixerSource;
use serde::{Deserialize, Serialize};
use stagemaster_audio::{OutputBinding, Transport};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum OutputKind {
    SystemDefault,
    Software,
}

pub(super) struct SoftwareOutput {
    source: MixerSource,
    origin: Instant,
    frames: u64,
}
impl SoftwareOutput {
    pub fn prepare(kind: OutputKind) -> (Transport, Option<Self>) {
        match kind {
            OutputKind::SystemDefault => (Transport::default(), None),
            OutputKind::Software => {
                let (mixer, source) = rodio::mixer::mixer(
                    2.try_into().expect("two channels"),
                    48_000.try_into().expect("valid rate"),
                );
                (
                    Transport::with_output(OutputBinding::new(mixer)),
                    Some(Self {
                        source,
                        origin: Instant::now(),
                        frames: 0,
                    }),
                )
            }
        }
    }
    pub fn reset_clock(&mut self) {
        self.origin = Instant::now();
        self.frames = 0;
    }
    pub fn pull(&mut self) -> Result<(), String> {
        let due = u64::try_from(self.origin.elapsed().as_nanos() * 48_000 / 1_000_000_000)
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
