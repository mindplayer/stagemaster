//! Deterministic playback state and explainable HTP/LTP output composition.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use stagemaster_domain::{AttributeAddress, AttributeDescriptor, NormalizedValue, PlaybackId};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Source {
    Playback(PlaybackId),
    Programmer(u64),
    ExternalInput(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Contribution {
    pub source: Source,
    pub address: AttributeAddress,
    pub value: NormalizedValue,
    pub weight: NormalizedValue,
    pub priority: i16,
    pub activation_order: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppliedContribution {
    pub source: Source,
    pub value: NormalizedValue,
    pub weight: NormalizedValue,
    pub priority: i16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAttribute {
    pub value: NormalizedValue,
    /// Every contribution that affected evaluation, ordered by application.
    pub trace: Vec<AppliedContribution>,
}

pub type OutputSnapshot = BTreeMap<AttributeAddress, ResolvedAttribute>;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Playback {
    id: PlaybackId,
    state: BTreeMap<AttributeAddress, NormalizedValue>,
    level: NormalizedValue,
    priority: i16,
    activation_order: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Runtime {
    playbacks: BTreeMap<PlaybackId, Playback>,
    next_activation_order: u64,
}

impl Runtime {
    pub fn activate(
        &mut self,
        id: PlaybackId,
        state: BTreeMap<AttributeAddress, NormalizedValue>,
        level: NormalizedValue,
        priority: i16,
    ) {
        self.next_activation_order = self.next_activation_order.saturating_add(1);
        self.playbacks.insert(
            id,
            Playback {
                id,
                state,
                level,
                priority,
                activation_order: self.next_activation_order,
            },
        );
    }

    pub fn set_level(&mut self, id: PlaybackId, level: NormalizedValue) -> bool {
        let Some(playback) = self.playbacks.get_mut(&id) else {
            return false;
        };
        playback.level = level;
        true
    }

    pub fn release(&mut self, id: PlaybackId) -> bool {
        self.playbacks.remove(&id).is_some()
    }

    #[must_use]
    pub fn render(
        &self,
        descriptors: &BTreeMap<AttributeAddress, AttributeDescriptor>,
    ) -> OutputSnapshot {
        let contributions = self.playbacks.values().flat_map(|playback| {
            playback.state.iter().map(|(address, value)| Contribution {
                source: Source::Playback(playback.id),
                address: *address,
                value: *value,
                weight: playback.level,
                priority: playback.priority,
                activation_order: playback.activation_order,
            })
        });
        Mixer::resolve(descriptors, contributions)
    }
}

pub struct Mixer;

impl Mixer {
    #[must_use]
    pub fn resolve(
        descriptors: &BTreeMap<AttributeAddress, AttributeDescriptor>,
        contributions: impl IntoIterator<Item = Contribution>,
    ) -> OutputSnapshot {
        let mut grouped: BTreeMap<AttributeAddress, Vec<Contribution>> = BTreeMap::new();
        for contribution in contributions {
            if contribution.weight != NormalizedValue::ZERO {
                grouped
                    .entry(contribution.address)
                    .or_default()
                    .push(contribution);
            }
        }

        descriptors
            .iter()
            .map(|(address, descriptor)| {
                let mut values = grouped.remove(address).unwrap_or_default();
                values.sort_by_key(|value| (value.priority, value.activation_order, value.source));
                let mut trace = Vec::with_capacity(values.len());
                let output = match descriptor.mix_mode {
                    stagemaster_domain::MixMode::HighestTakesPrecedence => {
                        let highest_priority = values.iter().map(|value| value.priority).max();
                        let mut output = descriptor.default;
                        for value in values
                            .into_iter()
                            .filter(|value| Some(value.priority) == highest_priority)
                        {
                            let effective = value.value.scale(value.weight);
                            if effective >= output {
                                output = effective;
                            }
                            trace.push(Self::trace(value));
                        }
                        output
                    }
                    stagemaster_domain::MixMode::LatestTakesPrecedence => {
                        let mut output = descriptor.default;
                        for value in values {
                            output = output.blend(value.value, value.weight);
                            trace.push(Self::trace(value));
                        }
                        output
                    }
                };
                (
                    *address,
                    ResolvedAttribute {
                        value: output,
                        trace,
                    },
                )
            })
            .collect()
    }

    const fn trace(value: Contribution) -> AppliedContribution {
        AppliedContribution {
            source: value.source,
            value: value.value,
            weight: value.weight,
            priority: value.priority,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stagemaster_domain::{Attribute, FixtureId, MixMode};

    fn descriptor(mix_mode: MixMode) -> (AttributeAddress, AttributeDescriptor) {
        (
            AttributeAddress::new(FixtureId(1), Attribute::Intensity),
            AttributeDescriptor {
                default: NormalizedValue::ZERO,
                mix_mode,
            },
        )
    }

    #[test]
    fn htp_takes_highest_scaled_value() {
        let (address, description) = descriptor(MixMode::HighestTakesPrecedence);
        let output = Mixer::resolve(
            &BTreeMap::from([(address, description)]),
            [
                Contribution {
                    source: Source::Playback(PlaybackId(1)),
                    address,
                    value: NormalizedValue::FULL,
                    weight: NormalizedValue::from_percent(40),
                    priority: 0,
                    activation_order: 1,
                },
                Contribution {
                    source: Source::Playback(PlaybackId(2)),
                    address,
                    value: NormalizedValue::from_percent(80),
                    weight: NormalizedValue::FULL,
                    priority: 0,
                    activation_order: 2,
                },
            ],
        );
        assert_eq!(output[&address].value, NormalizedValue::from_percent(80));
        assert_eq!(output[&address].trace.len(), 2);
    }

    #[test]
    fn ltp_uses_priority_then_latest_activation() {
        let (address, mut description) = descriptor(MixMode::LatestTakesPrecedence);
        description.default = NormalizedValue::from_percent(10);
        let mut runtime = Runtime::default();
        runtime.activate(
            PlaybackId(1),
            BTreeMap::from([(address, NormalizedValue::from_percent(30))]),
            NormalizedValue::FULL,
            0,
        );
        runtime.activate(
            PlaybackId(2),
            BTreeMap::from([(address, NormalizedValue::from_percent(70))]),
            NormalizedValue::FULL,
            0,
        );
        let output = runtime.render(&BTreeMap::from([(address, description)]));
        assert_eq!(output[&address].value, NormalizedValue::from_percent(70));
    }

    #[test]
    fn htp_uses_only_the_highest_active_priority_tier() {
        let (address, description) = descriptor(MixMode::HighestTakesPrecedence);
        let output = Mixer::resolve(
            &BTreeMap::from([(address, description)]),
            [
                Contribution {
                    source: Source::Playback(PlaybackId(1)),
                    address,
                    value: NormalizedValue::FULL,
                    weight: NormalizedValue::FULL,
                    priority: 0,
                    activation_order: 1,
                },
                Contribution {
                    source: Source::Playback(PlaybackId(2)),
                    address,
                    value: NormalizedValue::from_percent(30),
                    weight: NormalizedValue::FULL,
                    priority: 10,
                    activation_order: 2,
                },
            ],
        );
        assert_eq!(output[&address].value, NormalizedValue::from_percent(30));
        assert_eq!(output[&address].trace.len(), 1);
    }
}
