use stagemaster_domain::{
    Attribute as Key, AttributeAddress, AttributeDescriptor, FixtureId, MixMode, NormalizedValue,
    PlaybackId,
};
use stagemaster_engine::{Contribution, Mixer, Source, live};
use std::collections::BTreeMap;

#[test]
fn full_weight_continuous_rules_match_existing_reference_mixer_across_priority_and_zero_cases() {
    let address = AttributeAddress::new(FixtureId(1), Key::Intensity);
    for mode in [
        MixMode::HighestTakesPrecedence,
        MixMode::LatestTakesPrecedence,
    ] {
        for count in 0..=6_u8 {
            for variant in 0..32_u16 {
                let layout = live::Layout::new(
                    [3; 32],
                    vec![live::Attribute {
                        default: 40_000,
                        mix: mode,
                        intensity: true,
                        discrete: false,
                    }],
                )
                .unwrap();
                let mut live = live::LiveMixer::new([1; 16], layout, 6).unwrap();
                let mut contributions = Vec::new();
                for index in 0..count {
                    let value = (variant * 1009).wrapping_add(u16::from(index) * 1703);
                    let priority = i16::from((index + u8::try_from(variant).unwrap()) % 3) - 1;
                    let handle = live
                        .open(
                            live::Source {
                                id: [index + 1; 16],
                                kind: live::Kind::Playback,
                            },
                            priority,
                            [3; 32],
                        )
                        .unwrap();
                    live.publish(
                        handle,
                        live::Frame {
                            layout: [3; 32],
                            serial: 1,
                            values: &[Some(value)],
                            assert: &[false],
                        },
                    )
                    .unwrap();
                    contributions.push(Contribution {
                        source: Source::Playback(PlaybackId(u64::from(index))),
                        address,
                        value: NormalizedValue::from_raw(value),
                        weight: NormalizedValue::FULL,
                        priority,
                        activation_order: u64::from(index) + 1,
                    });
                }
                let expected = Mixer::resolve(
                    &BTreeMap::from([(
                        address,
                        AttributeDescriptor {
                            default: NormalizedValue::from_raw(40_000),
                            mix_mode: mode,
                        },
                    )]),
                    contributions,
                );
                let mut values = [0];
                live.render(&mut values, &mut [None]).unwrap();
                assert_eq!(values[0], expected[&address].value.raw());
            }
        }
    }
}
