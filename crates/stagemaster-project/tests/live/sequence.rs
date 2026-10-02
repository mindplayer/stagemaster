use super::{
    sequence_support::*,
    support::{open, values},
};
use stagemaster_engine::live::{Error, Kind, LiveMixer};
use stagemaster_playback::Command;

#[test]
fn reclaiming_a_hidden_axis_fades_from_visible_manual_position() {
    let (doc, id) = fixture("inherited", false, &[(0, 100, None)]);
    let mut list = doc.compile_live_sequence(&id, 0).unwrap();
    let mut m = LiveMixer::new([1; 16], list.layout().clone(), 2).unwrap();
    let source = open(&mut m, 1, Kind::Playback, 0);
    let manual = open(&mut m, 2, Kind::Programmer, 0);
    list.apply(Command::Next, 0, &m, source).unwrap();
    list.apply(Command::Advance, 100, &m, source).unwrap();
    list.publish(&mut m, source, 1).unwrap();
    hand(&mut m, manual, 1, 8000);
    list.apply(Command::Execute(0), 100, &m, source).unwrap();
    list.publish(&mut m, source, 2).unwrap();
    assert_eq!(
        values(&m)[1],
        8000,
        "reclaim must not jump to the hidden value"
    );
    list.apply(Command::Advance, 150, &m, source).unwrap();
    list.publish(&mut m, source, 3).unwrap();
    assert_eq!(values(&m)[1], 29_000);
}

#[test]
fn inherited_values_do_not_reassert_but_goto_does_and_delayed_release_returns_to_other_source() {
    let (doc, id) = fixture(
        "inherited",
        false,
        &[(0, 0, None), (0, 0, None), (100, 0, None)],
    );
    let mut list = doc.compile_live_sequence(&id, 0).unwrap();
    let mut output = list.prepare_output().unwrap();
    let mut m = LiveMixer::new([1; 16], list.layout().clone(), 2).unwrap();
    let source = open(&mut m, 1, Kind::Playback, 0);
    let manual = open(&mut m, 2, Kind::Programmer, 0);
    list.apply(Command::Execute(0), 0, &m, source).unwrap();
    list.publish(&mut m, source, 1).unwrap();
    hand(&mut m, manual, 1, 8000);
    list.apply(Command::Next, 0, &m, source).unwrap();
    list.publish(&mut m, source, 2).unwrap();
    assert_eq!(values(&m), [40_000, 8000, 20_000, 0]);
    list.apply(Command::Execute(1), 0, &m, source).unwrap();
    assert_eq!(list.publish(&mut m, source, 2), Err(Error::Sequence));
    assert_eq!(values(&m)[1], 8000);
    list.publish(&mut m, source, 3).unwrap();
    assert_eq!(values(&m)[1], 50_000);
    hand(&mut m, manual, 2, 9000);
    list.apply(Command::Execute(1), 0, &m, source).unwrap();
    list.publish(&mut m, source, 4).unwrap();
    list.apply(Command::Next, 0, &m, source).unwrap();
    list.publish(&mut m, source, 5).unwrap();
    assert_eq!(values(&m)[1], 50_000);
    list.apply(Command::Advance, 99, &m, source).unwrap();
    list.publish(&mut m, source, 6).unwrap();
    assert_eq!(values(&m)[1], 50_000);
    list.apply(Command::Advance, 100, &m, source).unwrap();
    list.publish(&mut m, source, 7).unwrap();
    assert_eq!(values(&m), [10_000, 9000, 20_000, 0]);
    assert_eq!(winners(&m)[1], Some(manual));
    let mut slots = [0; 512];
    output.render(&m, &mut slots).unwrap();
    assert_eq!((slots[1], slots[5]), (35, 40));
    list.apply(Command::Stop, 100, &m, source).unwrap();
    list.publish(&mut m, source, 8).unwrap();
    assert_eq!(values(&m), [12000, 9000, 0, 0]);
}

#[test]
fn newly_owned_axis_fades_from_composition_at_delay_end_and_pause_keeps_ownership() {
    let (doc, id) = fixture("inherited", false, &[(100, 200, None)]);
    let mut list = doc.compile_live_sequence(&id, 0).unwrap();
    let mut m = LiveMixer::new([1; 16], list.layout().clone(), 2).unwrap();
    let manual = open(&mut m, 2, Kind::Programmer, 0);
    let source = open(&mut m, 1, Kind::Playback, 0);
    hand(&mut m, manual, 1, 8000);
    list.apply(Command::Execute(0), 0, &m, source).unwrap();
    list.publish(&mut m, source, 1).unwrap();
    assert_eq!(winners(&m)[1], Some(manual));
    hand(&mut m, manual, 2, 12_000);
    list.apply(Command::Advance, 100, &m, source).unwrap();
    list.publish(&mut m, source, 2).unwrap();
    assert_eq!(values(&m)[1], 12_000);
    list.apply(Command::Pause, 200, &m, source).unwrap();
    list.publish(&mut m, source, 3).unwrap();
    assert_eq!(values(&m)[1], 31_000);
    hand(&mut m, manual, 3, 9000);
    list.apply(Command::Resume, 500, &m, source).unwrap();
    list.apply(Command::Advance, 600, &m, source).unwrap();
    list.publish(&mut m, source, 4).unwrap();
    assert_eq!(values(&m)[1], 9000, "resuming cannot reassert the axis");
    m.close(manual, 4).unwrap();
    assert_eq!(
        values(&m)[1],
        50_000,
        "hidden source continued its own fade"
    );
}

#[test]
fn isolated_steps_release_omissions_without_fading_to_profile_defaults() {
    let (doc, id) = fixture("isolated", false, &[(0, 0, None), (50, 200, None)]);
    let mut list = doc.compile_live_sequence(&id, 0).unwrap();
    let mut m = LiveMixer::new([1; 16], list.layout().clone(), 2).unwrap();
    let manual = open(&mut m, 2, Kind::Programmer, 0);
    let source = open(&mut m, 1, Kind::Playback, 0);
    hand(&mut m, manual, 1, 8000);
    list.apply(Command::Next, 0, &m, source).unwrap();
    list.publish(&mut m, source, 1).unwrap();
    assert_eq!(values(&m)[1], 50_000);
    list.apply(Command::Next, 0, &m, source).unwrap();
    list.apply(Command::Advance, 49, &m, source).unwrap();
    list.publish(&mut m, source, 2).unwrap();
    assert_eq!(values(&m)[1], 50_000);
    list.apply(Command::Advance, 50, &m, source).unwrap();
    list.publish(&mut m, source, 3).unwrap();
    assert_eq!(values(&m), [12000, 8000, 0, 0]);
    assert_eq!(
        winners(&m),
        [Some(source), Some(manual), None, None, None, None, None]
    );
    list.apply(Command::Advance, 150, &m, source).unwrap();
    list.publish(&mut m, source, 4).unwrap();
    assert_eq!(values(&m)[0], 26_000);
}
