#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;

use stagemaster_dmx::{
    ChannelMapping, DmxAddress, FixtureProfile, Patch, PatchedFixture, Universe,
};
use stagemaster_domain::{
    Attribute, AttributeAddress, CueId, FixtureId, GroupId, MixMode, NormalizedValue, PlaybackId,
    PresetId, SequenceId,
};
use stagemaster_engine::Runtime;
use stagemaster_show::{CueNumber, FixtureGroup, Preset, Programmer, Sequence};

fn main() -> Result<(), Box<dyn Error>> {
    let profile = rgb_profile();
    let patch = Patch::new(vec![
        fixture(FixtureId(1), "左侧面光", 1, profile.clone())?,
        fixture(FixtureId(2), "右侧面光", 10, profile)?,
    ])?;
    let front = FixtureGroup::new(GroupId(1), "面光", [FixtureId(1), FixtureId(2)]);

    let ocean = Preset::selective(
        PresetId(1),
        "海蓝",
        front.fixtures.iter().copied().map(|fixture| {
            (
                AttributeAddress::new(fixture, Attribute::Blue),
                NormalizedValue::from_percent(80),
            )
        }),
    );
    let mut presets = BTreeMap::from([(ocean.id, ocean)]);

    let mut programmer = Programmer::default();
    programmer.select(front.fixtures.iter().copied());
    programmer.set_literal(Attribute::Intensity, NormalizedValue::FULL);
    programmer.apply_preset(&presets[&PresetId(1)])?;

    let mut main_sequence = Sequence::new(SequenceId(1), "主序列");
    main_sequence.upsert_cue(programmer.record_cue(
        CueId(1),
        CueNumber::whole(1),
        "面光全亮／海蓝",
    ))?;

    // A preset update changes the referenced cue without rewriting that cue.
    presets
        .get_mut(&PresetId(1))
        .expect("demo preset exists")
        .values
        .values_mut()
        .for_each(|value| *value = NormalizedValue::from_percent(65));

    let tracked = main_sequence.tracked_state(CueId(1), &presets)?;
    let mut runtime = Runtime::default();
    runtime.activate(PlaybackId(1), tracked, NormalizedValue::FULL, 0);
    let output = runtime.render(&patch.descriptors());
    let frames = patch.encode(&output);
    let universe = Universe::new(1)?;
    let frame = &frames[&universe];

    println!("StageMaster core loop");
    println!("  fixtures: {}", patch.fixtures().len());
    println!("  cue: {}", main_sequence.cues()[0].name);
    println!(
        "  U1.001 intensity: {}",
        frame.channel(1).expect("channel exists")
    );
    println!(
        "  U1.004 blue: {}",
        frame.channel(4).expect("channel exists")
    );
    println!(
        "  U1.010 intensity: {}",
        frame.channel(10).expect("channel exists")
    );
    println!(
        "  U1.013 blue: {}",
        frame.channel(13).expect("channel exists")
    );
    let blue = AttributeAddress::new(FixtureId(1), Attribute::Blue);
    println!("  blue source trace: {:?}", output[&blue].trace);
    Ok(())
}

fn fixture(
    id: FixtureId,
    name: &str,
    address: u16,
    profile: FixtureProfile,
) -> Result<PatchedFixture, Box<dyn Error>> {
    Ok(PatchedFixture {
        id,
        name: name.into(),
        universe: Universe::new(1)?,
        address: DmxAddress::new(address)?,
        profile,
    })
}

fn rgb_profile() -> FixtureProfile {
    FixtureProfile {
        manufacturer: "StageMaster".into(),
        model: "RGB Demo".into(),
        mode: "4ch".into(),
        footprint: 4,
        channels: vec![
            channel(Attribute::Intensity, 0, MixMode::HighestTakesPrecedence),
            channel(Attribute::Red, 1, MixMode::LatestTakesPrecedence),
            channel(Attribute::Green, 2, MixMode::LatestTakesPrecedence),
            channel(Attribute::Blue, 3, MixMode::LatestTakesPrecedence),
        ],
    }
}

const fn channel(attribute: Attribute, coarse_offset: u16, mix_mode: MixMode) -> ChannelMapping {
    ChannelMapping {
        attribute,
        coarse_offset,
        fine_offset: None,
        default: NormalizedValue::ZERO,
        mix_mode,
    }
}
