//! Stable, deterministic value types shared by the editing and live engines.

#![forbid(unsafe_code)]

use std::fmt;

macro_rules! numeric_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub u64);
    };
}

numeric_id!(FixtureId);
numeric_id!(GroupId);
numeric_id!(PresetId);
numeric_id!(SequenceId);
numeric_id!(CueId);
numeric_id!(PlaybackId);

/// DMX-independent attributes understood by the show model.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Attribute {
    Intensity,
    Pan,
    Tilt,
    Red,
    Green,
    Blue,
    White,
    Amber,
    Shutter,
    Gobo(u8),
    Focus,
    Zoom,
    Iris,
    BladeInsertion(u8),
    BladeRotation(u8),
    Custom(u16),
}

impl Attribute {
    #[must_use]
    pub const fn default_mix_mode(self) -> MixMode {
        match self {
            Self::Intensity => MixMode::HighestTakesPrecedence,
            _ => MixMode::LatestTakesPrecedence,
        }
    }
}

/// Fixed-point normalized value. Using an integer keeps show evaluation reproducible.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NormalizedValue(u16);

impl NormalizedValue {
    pub const ZERO: Self = Self(0);
    pub const FULL: Self = Self(u16::MAX);

    #[must_use]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    #[must_use]
    pub fn from_percent(percent: u8) -> Self {
        let clamped = u32::from(percent.min(100));
        Self::from_u32_saturated((clamped * u32::from(u16::MAX) + 50) / 100)
    }

    #[must_use]
    pub fn scale(self, weight: Self) -> Self {
        let product = u32::from(self.0) * u32::from(weight.0);
        Self::from_u32_saturated((product + u32::from(u16::MAX) / 2) / u32::from(u16::MAX))
    }

    #[must_use]
    pub fn blend(self, target: Self, weight: Self) -> Self {
        let inverse = u32::from(u16::MAX - weight.0);
        let mixed = u32::from(self.0) * inverse + u32::from(target.0) * u32::from(weight.0);
        Self::from_u32_saturated((mixed + u32::from(u16::MAX) / 2) / u32::from(u16::MAX))
    }

    fn from_u32_saturated(value: u32) -> Self {
        match u16::try_from(value) {
            Ok(value) => Self(value),
            Err(_) => Self::FULL,
        }
    }
}

impl fmt::Display for NormalizedValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:.1}%",
            f64::from(self.0) * 100.0 / f64::from(u16::MAX)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MixMode {
    HighestTakesPrecedence,
    LatestTakesPrecedence,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AttributeAddress {
    pub fixture: FixtureId,
    pub attribute: Attribute,
}

impl AttributeAddress {
    #[must_use]
    pub const fn new(fixture: FixtureId, attribute: Attribute) -> Self {
        Self { fixture, attribute }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttributeDescriptor {
    pub default: NormalizedValue,
    pub mix_mode: MixMode,
}

#[cfg(test)]
mod tests {
    use super::NormalizedValue;

    #[test]
    fn fixed_point_scaling_and_blending_are_deterministic() {
        assert_eq!(NormalizedValue::from_percent(100), NormalizedValue::FULL);
        assert_eq!(NormalizedValue::from_percent(120), NormalizedValue::FULL);
        let half = NormalizedValue::from_percent(50);
        let quarter = NormalizedValue::FULL.scale(half);
        assert!(quarter.raw().abs_diff(half.raw()) <= 1);
        assert!(
            NormalizedValue::ZERO
                .blend(NormalizedValue::FULL, half)
                .raw()
                .abs_diff(half.raw())
                <= 1
        );
    }
}
