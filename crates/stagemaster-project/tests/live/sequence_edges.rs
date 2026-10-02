use super::{
    sequence_support::*,
    support::{edit, open, values},
};
use serde_json::json;
use stagemaster_engine::live::{Frame, Kind, LiveMixer};
use stagemaster_playback::{Command, Status};

#[test]
fn effects_are_step_local_and_release_their_axis_when_no_static_value_was_tracked() {
    let (mut doc, id) = fixture("inherited", false, &[(0, 0, None), (0, 0, None)]);
    let v = doc.view();
    let scene = v.scenes[0].id.clone();
    let fixture = v.fixtures[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,"attribute":"pan","mode":"remove","value":0}),
    );
    edit(
        &mut doc,
        json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{
        "id":"99999999-0000-4000-8000-000000000030","name":"列表运动","enabled":true,"fixtureIds":[fixture],
        "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"position","dutyPercent":50,
        "channels":[{"attribute":"pan","amplitudeDegrees":"30","offsetDegrees":"0","phaseDegrees":0}]}}}),
    );
    let mut list = doc.compile_live_sequence(&id, 0).unwrap();
    let mut m = LiveMixer::new([1; 16], list.layout().clone(), 2).unwrap();
    let manual = open(&mut m, 2, Kind::Programmer, 0);
    let source = open(&mut m, 1, Kind::Playback, 0);
    hand(&mut m, manual, 1, 8000);
    list.apply(Command::Next, 0, &m, source).unwrap();
    list.apply(Command::Advance, 250, &m, source).unwrap();
    list.publish(&mut m, source, 1).unwrap();
    assert_eq!(winners(&m)[1], Some(source));
    assert_ne!(values(&m)[1], 8000);
    list.apply(Command::Next, 250, &m, source).unwrap();
    list.publish(&mut m, source, 2).unwrap();
    assert_eq!(winners(&m)[1], Some(manual));
    assert_eq!(values(&m)[1], 8000);
}

#[test]
fn full_cycle_catchup_reclaims_tracked_color_even_when_sampled_step_is_unchanged() {
    for (delay, sampled, index, elapsed, red) in [
        (0, 300, 2, 0, 20_000),
        (0, 600, 2, 0, 20_000),
        (0, 300_000_000_000, 2, 0, 20_000),
        // No complete cycle: the new first step is still delayed and cannot reassert red.
        (50, 125, 0, 25, 9000),
        // Omitted complete cycles changed red even though the destination has not activated.
        (50, 425, 0, 25, 20_000),
        (50, 300_000_000_125, 0, 25, 20_000),
    ] {
        let (doc, id) = fixture(
            "inherited",
            true,
            &[
                (delay, 0, Some(100 - delay)),
                (0, 0, Some(100)),
                (0, 0, Some(100)),
            ],
        );
        let mut list = doc.compile_live_sequence(&id, 0).unwrap();
        let mut m = LiveMixer::new([1; 16], list.layout().clone(), 2).unwrap();
        let source = open(&mut m, 1, Kind::Playback, 0);
        let manual = open(&mut m, 2, Kind::Programmer, 0);
        list.apply(Command::Execute(2), 0, &m, source).unwrap();
        list.publish(&mut m, source, 1).unwrap();
        m.publish(
            manual,
            Frame {
                layout: m.layout().id(),
                serial: 1,
                values: &[None, Some(8000), Some(9000), None, None, None, None],
                assert: &[false; 7],
            },
        )
        .unwrap();
        assert_eq!(values(&m), [10_000, 8000, 9000, 0]);
        list.apply(Command::Advance, sampled, &m, source).unwrap();
        list.publish(&mut m, source, 2).unwrap();
        assert_eq!(list.index(), Some(index));
        assert_eq!(list.elapsed_ms(), elapsed);
        assert_eq!(values(&m), [10_000, 8000, red, 0]);
        assert_eq!(
            winners(&m)[2],
            Some(if red == 9000 { manual } else { source })
        );
    }
}

#[test]
fn finished_list_holds_until_stop_and_wrong_snapshot_or_invalid_time_cannot_change_it() {
    let (doc, id) = fixture("inherited", false, &[(0, 100, Some(50))]);
    let mut list = doc.compile_live_sequence(&id, 0).unwrap();
    let mut m = LiveMixer::new([1; 16], list.layout().clone(), 1).unwrap();
    let source = open(&mut m, 1, Kind::Playback, 0);
    list.apply(Command::Execute(0), 0, &m, source).unwrap();
    list.apply(Command::Advance, 150, &m, source).unwrap();
    list.publish(&mut m, source, 1).unwrap();
    assert_eq!(list.status(), Status::Finished);
    assert_eq!(values(&m), [12000, 50_000, 20_000, 0]);
    assert!(list.apply(Command::Execute(9), 1000, &m, source).is_err());
    assert!(list.apply(Command::Stop, 149, &m, source).is_err());
    let mut changed = raw(&doc);
    changed["project"]["name"] = json!("另一个快照");
    let other = decode(&changed).compile_live_sequence(&id, 0).unwrap();
    let other_mixer = LiveMixer::new([2; 16], other.layout().clone(), 1).unwrap();
    assert!(
        list.apply(Command::Stop, 150, &other_mixer, source)
            .is_err()
    );
    assert_eq!(list.status(), Status::Finished);
    list.apply(Command::Stop, 150, &m, source).unwrap();
    list.publish(&mut m, source, 2).unwrap();
    assert_eq!(values(&m), [12000, 32768, 0, 0]);
    assert_eq!(winners(&m), [None; 7]);
    m.close(source, 3).unwrap();
    assert!(list.apply(Command::Execute(0), 150, &m, source).is_err());
    assert_eq!(list.status(), Status::Idle);
}

#[test]
fn new_intensity_origin_is_not_attenuated_twice_and_zero_level_stays_dark() {
    let (doc, id) = fixture("isolated", false, &[(0, 0, None), (0, 200, None)]);
    for (level, initial, halfway, final_value) in [(32768, 12000, 16000, 20000), (0, 0, 0, 0)] {
        let mut list = doc.compile_live_sequence(&id, 0).unwrap();
        let mut m = LiveMixer::new([1; 16], list.layout().clone(), 1).unwrap();
        let source = open(&mut m, 1, Kind::Playback, 0);
        m.set_level(source, 1, level).unwrap();
        list.apply(Command::Execute(1), 0, &m, source).unwrap();
        list.publish(&mut m, source, 2).unwrap();
        assert_eq!(values(&m)[0], initial);
        list.apply(Command::Advance, 100, &m, source).unwrap();
        list.publish(&mut m, source, 3).unwrap();
        assert_eq!(values(&m)[0], halfway);
        list.apply(Command::Advance, 200, &m, source).unwrap();
        list.publish(&mut m, source, 4).unwrap();
        assert_eq!(values(&m)[0], final_value);
    }
}
