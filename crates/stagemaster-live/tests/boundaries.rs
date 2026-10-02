mod support;
use stagemaster_live::{Change, Command, Session};
use stagemaster_project::PackageSelection;
use support::*;

#[test]
fn invalid_inputs_and_stale_source_keys_do_not_advance_or_refresh_a_frame() {
    let doc = decode(&fixture(70));
    let mut s = Session::prepare(&doc, [9; 16], &specs(), 0).unwrap();
    let a = s.key([1; 16]).unwrap();
    let m = s.key([3; 16]).unwrap();
    s.control(a, Command::Execute(0), 0).unwrap();
    s.tick(25).unwrap();
    let before = *s.frame().unwrap();
    let other = Session::prepare(&doc, [8; 16], &specs(), 0).unwrap();
    assert!(
        s.control(other.key([1; 16]).unwrap(), Command::Stop, 50)
            .is_err()
    );
    assert!(s.control(a, Command::Execute(3), 50).is_err());
    assert!(s.control(m, Command::Pause, 50).is_err());
    assert!(s.patch(a, &[], 50).is_err());
    assert!(
        s.patch(
            m,
            &[Change {
                attribute: 4,
                value: Some(1)
            }],
            50
        )
        .is_err()
    );
    assert!(
        s.patch(
            m,
            &[Change {
                attribute: 1,
                value: Some(1)
            }; 2],
            50
        )
        .is_err()
    );
    assert!(s.tick(24).is_err());
    assert!(s.set_level(a, 0, 24).is_err());
    assert_eq!(s.frame(), Some(&before));
    assert!(s.fault().is_none());
    assert_eq!(s.sources().next().unwrap().step, Some(0));
    s.tick(80).unwrap();
    assert_eq!(s.values().unwrap()[1], 30_000);
}
#[test]
fn preparation_rejects_invalid_sources_before_execution() {
    let doc = decode(&fixture(70));
    assert!(Session::prepare(&doc, [9; 16], &[], 0).is_err());
    assert!(Session::prepare(&doc, [0; 16], &specs(), 0).is_err());
    assert!(Session::prepare(&doc, [9; 16], &vec![specs()[0].clone(); 65], 0).is_err());
    assert!(Session::prepare(&doc, [9; 16], &vec![specs()[0].clone(); 2], 0).is_err());
    assert!(Session::prepare(&doc, [9; 16], &[specs()[2].clone()], 0).is_err());
    let missing = playback(1, PackageSelection::Scene { id: id(999) });
    assert!(Session::prepare(&doc, [9; 16], &[missing], 0).is_err());
    let mut zero = specs();
    zero[0].id = [0; 16];
    assert!(Session::prepare(&doc, [9; 16], &zero, 0).is_err());
}
#[test]
fn held_dynamic_scene_fader_and_manual_layer_share_one_encoder() {
    let mut doc = decode(&fixture(70));
    doc.edit(serde_json::from_value(serde_json::json!({"op":"effect","command":{"kind":"put","sceneId":id(5),"effect":{
        "id":id(90),"name":"呼吸","enabled":true,"fixtureIds":[doc.view().fixtures[0].id],
        "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,
        "channels":[{"attribute":"dimmer","low":10_000,"high":50_000}]
    }}})).unwrap()).unwrap();
    let mut specs = specs();
    specs.push(playback(4, PackageSelection::Scene { id: id(5) }));
    let mut s = Session::prepare(&doc, [9; 16], &specs, 0).unwrap();
    for n in [1, 2, 4] {
        s.control(s.key([n; 16]).unwrap(), Command::Execute(0), 0)
            .unwrap();
    }
    let held = s.key([4; 16]).unwrap();
    s.tick(500).unwrap();
    assert_eq!(s.values().unwrap()[0], 50_000);
    assert_eq!(s.winner(0), Some([4; 16]));
    s.set_level(held, 0, 500).unwrap();
    assert_eq!(s.values().unwrap()[0], 25_000); // Other list still contributes HTP intensity.
    assert_eq!(s.winner(1), Some([1; 16]));
    s.control(s.key([1; 16]).unwrap(), Command::Stop, 500)
        .unwrap();
    assert_eq!(s.values().unwrap()[0], 0);
    assert_eq!(s.winner(0), Some([4; 16])); // Zero fader retains ownership.
    s.set_level(held, u16::MAX, 500).unwrap();
    s.tick(1000).unwrap();
    assert_eq!(s.values().unwrap()[0], 10_000);
    let m = s.key([3; 16]).unwrap();
    s.patch(
        m,
        &[Change {
            attribute: 1,
            value: Some(40_000),
        }],
        1000,
    )
    .unwrap();
    let actual = s.frame().unwrap();
    let p = doc.compile_scene(&id(5)).unwrap();
    let mut expected = [0; 512];
    p.output
        .portable_output()
        .unwrap()
        .render(&[10_000, 40_000, 0, 0], &mut expected)
        .unwrap();
    assert_eq!(actual.slots, expected);
    s.control(m, Command::Stop, 1000).unwrap();
    assert_eq!(s.winner(1), Some([2; 16]));
}
