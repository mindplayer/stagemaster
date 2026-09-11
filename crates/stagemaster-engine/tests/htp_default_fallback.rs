use std::collections::BTreeMap;

use stagemaster_domain::{
    Attribute, AttributeAddress, AttributeDescriptor, FixtureId, MixMode, NormalizedValue,
    PlaybackId,
};
use stagemaster_engine::{Contribution, Mixer, Source};

const DEFAULT: NormalizedValue = NormalizedValue::from_raw(40_000);

fn address() -> AttributeAddress {
    AttributeAddress::new(FixtureId(1), Attribute::Intensity)
}

fn descriptors() -> BTreeMap<AttributeAddress, AttributeDescriptor> {
    BTreeMap::from([(
        address(),
        AttributeDescriptor {
            default: DEFAULT,
            mix_mode: MixMode::HighestTakesPrecedence,
        },
    )])
}

fn contribution(
    playback: u64,
    value: u16,
    weight: NormalizedValue,
    priority: i16,
    activation_order: u64,
) -> Contribution {
    Contribution {
        source: Source::Playback(PlaybackId(playback)),
        address: address(),
        value: NormalizedValue::from_raw(value),
        weight,
        priority,
        activation_order,
    }
}

fn resolve(
    contributions: impl IntoIterator<Item = Contribution>,
) -> stagemaster_engine::ResolvedAttribute {
    Mixer::resolve(&descriptors(), contributions)
        .remove(&address())
        .expect("known descriptor must produce an output")
}

#[test]
fn falls_back_to_default_with_empty_trace_when_there_are_no_contributions() {
    let resolved = resolve([]);

    assert_eq!(resolved.value, DEFAULT);
    assert!(resolved.trace.is_empty());
}

#[test]
fn contribution_below_default_is_not_clamped_by_default() {
    let resolved = resolve([contribution(1, 10_000, NormalizedValue::FULL, 0, 1)]);

    assert_eq!(resolved.value, NormalizedValue::from_raw(10_000));
    assert_eq!(resolved.trace.len(), 1);
}

#[test]
fn highest_scaled_value_wins_within_the_highest_priority_tier() {
    let resolved = resolve([
        contribution(1, 10_000, NormalizedValue::FULL, 5, 1),
        contribution(2, 20_000, NormalizedValue::FULL, 5, 2),
    ]);

    assert_eq!(resolved.value, NormalizedValue::from_raw(20_000));
    assert_eq!(resolved.trace.len(), 2);
}

#[test]
fn explicit_zero_is_an_effective_result_instead_of_default_fallback() {
    let resolved = resolve([contribution(1, 0, NormalizedValue::FULL, 0, 1)]);

    assert_eq!(resolved.value, NormalizedValue::ZERO);
    assert_eq!(resolved.trace.len(), 1);
}

#[test]
fn only_the_highest_active_priority_tier_contributes() {
    let resolved = resolve([
        contribution(1, 50_000, NormalizedValue::FULL, 1, 1),
        contribution(2, 10_000, NormalizedValue::FULL, 10, 2),
    ]);

    assert_eq!(resolved.value, NormalizedValue::from_raw(10_000));
    assert_eq!(resolved.trace.len(), 1);
    assert_eq!(resolved.trace[0].source, Source::Playback(PlaybackId(2)));
}

#[test]
fn zero_weight_high_priority_contribution_does_not_mask_lower_priority() {
    let resolved = resolve([
        contribution(1, 10_000, NormalizedValue::ZERO, 10, 2),
        contribution(2, 30_000, NormalizedValue::FULL, 1, 1),
    ]);

    assert_eq!(resolved.value, NormalizedValue::from_raw(30_000));
    assert_eq!(resolved.trace.len(), 1);
    assert_eq!(resolved.trace[0].source, Source::Playback(PlaybackId(2)));
}

#[test]
fn all_zero_weight_contributions_fall_back_to_default_with_empty_trace() {
    let resolved = resolve([
        contribution(1, 10_000, NormalizedValue::ZERO, 10, 2),
        contribution(2, 50_000, NormalizedValue::ZERO, 1, 1),
    ]);

    assert_eq!(resolved.value, DEFAULT);
    assert!(resolved.trace.is_empty());
}

#[test]
fn non_full_weights_use_existing_fixed_point_scaling_before_comparison() {
    let half = NormalizedValue::from_raw(32_768);
    let partially_scaled = NormalizedValue::from_raw(30_000).scale(half);
    let full_weight = NormalizedValue::from_raw(20_000);
    let resolved = resolve([
        contribution(1, 30_000, half, 3, 1),
        contribution(2, 20_000, NormalizedValue::FULL, 3, 2),
    ]);

    assert_eq!(resolved.value, partially_scaled.max(full_weight));
    assert_eq!(resolved.trace.len(), 2);
}
