//! Patch validation and pure DMX512 frame encoding. Physical transports implement `DmxSink`.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use stagemaster_domain::{
    Attribute, AttributeAddress, AttributeDescriptor, FixtureId, MixMode, NormalizedValue,
};
use stagemaster_engine::OutputSnapshot;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Universe(u16);

impl Universe {
    /// Creates a non-zero logical universe number.
    ///
    /// # Errors
    ///
    /// Returns [`PatchError::InvalidUniverse`] when `number` is zero.
    pub fn new(number: u16) -> Result<Self, PatchError> {
        if number == 0 {
            Err(PatchError::InvalidUniverse(number))
        } else {
            Ok(Self(number))
        }
    }

    #[must_use]
    pub const fn number(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DmxAddress(u16);

impl DmxAddress {
    /// Creates a one-based DMX512 address.
    ///
    /// # Errors
    ///
    /// Returns [`PatchError::InvalidAddress`] unless `address` is between 1 and 512.
    pub fn new(address: u16) -> Result<Self, PatchError> {
        if (1..=512).contains(&address) {
            Ok(Self(address))
        } else {
            Err(PatchError::InvalidAddress(address))
        }
    }

    #[must_use]
    pub const fn number(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChannelMapping {
    pub attribute: Attribute,
    /// Zero-based offset from the fixture's start address.
    pub coarse_offset: u16,
    pub fine_offset: Option<u16>,
    pub default: NormalizedValue,
    pub mix_mode: MixMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureProfile {
    pub manufacturer: String,
    pub model: String,
    pub mode: String,
    pub footprint: u16,
    pub channels: Vec<ChannelMapping>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatchedFixture {
    pub id: FixtureId,
    pub name: String,
    pub universe: Universe,
    pub address: DmxAddress,
    pub profile: FixtureProfile,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Patch {
    fixtures: Vec<PatchedFixture>,
}

impl Patch {
    /// Validates fixture profiles, fixture IDs, universe boundaries, and occupied addresses.
    ///
    /// # Errors
    ///
    /// Returns a [`PatchError`] describing the first invalid profile or patch conflict.
    pub fn new(fixtures: Vec<PatchedFixture>) -> Result<Self, PatchError> {
        let mut fixture_ids = BTreeSet::new();
        let mut occupied = BTreeMap::new();
        for fixture in &fixtures {
            if !fixture_ids.insert(fixture.id) {
                return Err(PatchError::DuplicateFixture(fixture.id));
            }
            Self::validate_profile(&fixture.profile)?;
            let end = fixture
                .address
                .number()
                .checked_add(fixture.profile.footprint.saturating_sub(1));
            let Some(end) = end else {
                return Err(PatchError::FixtureCrossesUniverse {
                    fixture: fixture.id,
                    start: fixture.address,
                    footprint: fixture.profile.footprint,
                });
            };
            if fixture.profile.footprint == 0 || end > 512 {
                return Err(PatchError::FixtureCrossesUniverse {
                    fixture: fixture.id,
                    start: fixture.address,
                    footprint: fixture.profile.footprint,
                });
            }
            for channel in fixture.address.number()..=end {
                if let Some(first) = occupied.insert((fixture.universe, channel), fixture.id) {
                    return Err(PatchError::AddressCollision {
                        universe: fixture.universe,
                        channel,
                        first,
                        second: fixture.id,
                    });
                }
            }
        }
        Ok(Self { fixtures })
    }

    fn validate_profile(profile: &FixtureProfile) -> Result<(), PatchError> {
        let mut offsets = BTreeSet::new();
        let mut attributes = BTreeSet::new();
        for channel in &profile.channels {
            if channel.coarse_offset >= profile.footprint
                || channel
                    .fine_offset
                    .is_some_and(|offset| offset >= profile.footprint)
            {
                return Err(PatchError::ProfileOffsetOutsideFootprint {
                    attribute: channel.attribute,
                    footprint: profile.footprint,
                });
            }
            if !offsets.insert(channel.coarse_offset)
                || channel
                    .fine_offset
                    .is_some_and(|offset| !offsets.insert(offset))
            {
                return Err(PatchError::DuplicateProfileOffset);
            }
            if !attributes.insert(channel.attribute) {
                return Err(PatchError::DuplicateAttribute(channel.attribute));
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn fixtures(&self) -> &[PatchedFixture] {
        &self.fixtures
    }

    #[must_use]
    pub fn descriptors(&self) -> BTreeMap<AttributeAddress, AttributeDescriptor> {
        self.fixtures
            .iter()
            .flat_map(|fixture| {
                fixture.profile.channels.iter().map(|channel| {
                    (
                        AttributeAddress::new(fixture.id, channel.attribute),
                        AttributeDescriptor {
                            default: channel.default,
                            mix_mode: channel.mix_mode,
                        },
                    )
                })
            })
            .collect()
    }

    #[must_use]
    pub fn encode(&self, output: &OutputSnapshot) -> BTreeMap<Universe, DmxFrame> {
        let mut frames = BTreeMap::new();
        for fixture in &self.fixtures {
            let frame = frames
                .entry(fixture.universe)
                .or_insert_with(DmxFrame::default);
            for channel in &fixture.profile.channels {
                let address = AttributeAddress::new(fixture.id, channel.attribute);
                let value = output
                    .get(&address)
                    .map_or(channel.default, |resolved| resolved.value);
                let [coarse, fine] = value.raw().to_be_bytes();
                frame.set(fixture.address.number() + channel.coarse_offset, coarse);
                if let Some(offset) = channel.fine_offset {
                    frame.set(fixture.address.number() + offset, fine);
                }
            }
        }
        frames
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct DmxFrame([u8; 512]);

impl Default for DmxFrame {
    fn default() -> Self {
        Self([0; 512])
    }
}

impl fmt::Debug for DmxFrame {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("DmxFrame")
            .field(&"512 slots")
            .finish()
    }
}

impl DmxFrame {
    fn set(&mut self, one_based_channel: u16, value: u8) {
        self.0[usize::from(one_based_channel - 1)] = value;
    }

    #[must_use]
    pub fn channel(&self, one_based_channel: u16) -> Option<u8> {
        self.0
            .get(usize::from(one_based_channel.checked_sub(1)?))
            .copied()
    }

    #[must_use]
    pub const fn slots(&self) -> &[u8; 512] {
        &self.0
    }
}

pub trait DmxSink {
    type Error;

    /// A real adapter owns physical timing; callers provide complete logical frames.
    ///
    /// # Errors
    ///
    /// Returns the adapter-specific error if it cannot accept or transmit the frame.
    fn send(&mut self, universe: Universe, frame: &DmxFrame) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatchError {
    InvalidUniverse(u16),
    InvalidAddress(u16),
    DuplicateFixture(FixtureId),
    FixtureCrossesUniverse {
        fixture: FixtureId,
        start: DmxAddress,
        footprint: u16,
    },
    AddressCollision {
        universe: Universe,
        channel: u16,
        first: FixtureId,
        second: FixtureId,
    },
    ProfileOffsetOutsideFootprint {
        attribute: Attribute,
        footprint: u16,
    },
    DuplicateProfileOffset,
    DuplicateAttribute(Attribute),
}

impl fmt::Display for PatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for PatchError {}

#[cfg(test)]
mod tests {
    use super::*;
    use stagemaster_engine::ResolvedAttribute;

    fn dimmer_profile() -> FixtureProfile {
        FixtureProfile {
            manufacturer: "StageMaster".into(),
            model: "Dimmer".into(),
            mode: "16 bit".into(),
            footprint: 2,
            channels: vec![ChannelMapping {
                attribute: Attribute::Intensity,
                coarse_offset: 0,
                fine_offset: Some(1),
                default: NormalizedValue::ZERO,
                mix_mode: MixMode::HighestTakesPrecedence,
            }],
        }
    }

    #[test]
    fn rejects_address_collisions() {
        let universe = Universe::new(1).expect("valid universe");
        let first = PatchedFixture {
            id: FixtureId(1),
            name: "A".into(),
            universe,
            address: DmxAddress::new(1).expect("valid address"),
            profile: dimmer_profile(),
        };
        let mut second = first.clone();
        second.id = FixtureId(2);
        second.address = DmxAddress::new(2).expect("valid address");
        assert!(matches!(
            Patch::new(vec![first, second]),
            Err(PatchError::AddressCollision { channel: 2, .. })
        ));
    }

    #[test]
    fn encodes_sixteen_bit_attributes_coarse_then_fine() {
        let fixture = PatchedFixture {
            id: FixtureId(1),
            name: "A".into(),
            universe: Universe::new(1).expect("valid universe"),
            address: DmxAddress::new(10).expect("valid address"),
            profile: dimmer_profile(),
        };
        let patch = Patch::new(vec![fixture]).expect("valid patch");
        let address = AttributeAddress::new(FixtureId(1), Attribute::Intensity);
        let output = BTreeMap::from([(
            address,
            ResolvedAttribute {
                value: NormalizedValue::from_raw(0xABCD),
                trace: vec![],
            },
        )]);
        let frames = patch.encode(&output);
        let frame = &frames[&Universe::new(1).expect("valid universe")];
        assert_eq!(frame.channel(10), Some(0xAB));
        assert_eq!(frame.channel(11), Some(0xCD));
    }
}
