//! Integer cyclic curves; sampling needs no clock, allocation or device knowledge.
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Curve {
    Smooth,
    Triangle,
    Pulse,
    Keyframes(Vec<Keyframe>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition {
    Hold,
    Linear,
    Smooth,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keyframe {
    pub phase: u16,
    pub value: u16,
    pub transition: Transition,
}

fn smooth(weight: u64) -> u64 {
    weight * weight * (3 * 65_536 - 2 * weight) / (65_536 * 65_536)
}
fn blend(from: u16, to: u16, weight: u64) -> u16 {
    u16::try_from((u64::from(from) * (65_536 - weight) + u64::from(to) * weight + 32_768) / 65_536)
        .expect("bounded curve")
}

fn sample_keyframes(frames: &[Keyframe], phase: u64) -> u16 {
    // Validated plans start at phase zero and contain strictly increasing positions.
    let index = frames.partition_point(|frame| u64::from(frame.phase) <= phase) - 1;
    let from = &frames[index];
    let next = frames.get(index + 1);
    let end = next.map_or(65_536, |frame| u64::from(frame.phase));
    let weight = (phase - u64::from(from.phase)) * 65_536 / (end - u64::from(from.phase));
    let weight = match from.transition {
        Transition::Hold => 0,
        Transition::Linear => weight,
        Transition::Smooth => smooth(weight),
    };
    blend(from.value, next.unwrap_or(&frames[0]).value, weight)
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
    pub(crate) fn sample(&self, elapsed_ms: u64, offset_ms: u64) -> u16 {
        let period = u64::from(self.period_ms);
        let elapsed = (elapsed_ms % period + offset_ms % period) % period;
        let phase = (elapsed * 65_536 / period + 65_536 - u64::from(self.phase)) % 65_536;
        if let Curve::Keyframes(frames) = &self.curve {
            return sample_keyframes(frames, phase);
        }
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
                smooth(linear)
            } else {
                linear
            }
        };
        blend(self.low, self.high, weight)
    }
}
