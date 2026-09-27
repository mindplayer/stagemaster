//! Integer two-point cyclic curves; no clock, allocation or device knowledge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curve {
    Smooth,
    Triangle,
    Pulse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectChannel {
    pub index: usize,
    pub low: u16,
    pub high: u16,
    pub period_ms: u32,
    /// A full turn is 65536. Positive phase delays this channel.
    pub phase: u16,
    pub curve: Curve,
    pub duty_percent: u8,
}

impl EffectChannel {
    #[must_use]
    pub(crate) fn sample(&self, elapsed_ms: u64) -> u16 {
        let period = u64::from(self.period_ms);
        let phase =
            ((elapsed_ms % period) * 65_536 / period + 65_536 - u64::from(self.phase)) % 65_536;
        let weight = if self.curve == Curve::Pulse {
            if phase * 100 < u64::from(self.duty_percent) * 65_536 {
                65_536
            } else {
                0
            }
        } else {
            let linear = if phase <= 32_768 {
                phase * 2
            } else {
                (65_536 - phase) * 2
            };
            if self.curve == Curve::Smooth {
                // Q16 smoothstep, exact endpoints, with all intermediates bounded in u64.
                linear * linear * (3 * 65_536 - 2 * linear) / (65_536 * 65_536)
            } else {
                linear
            }
        };
        u16::try_from(
            (u64::from(self.low) * (65_536 - weight) + u64::from(self.high) * weight + 32_768)
                / 65_536,
        )
        .expect("bounded curve")
    }
}
