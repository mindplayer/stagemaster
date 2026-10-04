mod support;
use stagemaster_live::{Change, Command, Session};
use support::{decode, fixture, specs};

#[test]
fn manual_reading_borrows_original_contribution_without_advancing_or_mixing() {
    let doc = decode(&fixture(70));
    let mut session = Session::prepare(&doc, [9; 16], &specs(), 0).unwrap();
    let manual = session.key([3; 16]).unwrap();
    let playback = session.key([1; 16]).unwrap();
    let other = Session::prepare(&doc, [8; 16], &specs(), 0).unwrap();
    assert!(session.manual_values(playback).is_none());
    assert!(session.manual_values(other.key([3; 16]).unwrap()).is_none());
    session
        .patch(
            manual,
            &[Change {
                attribute: 0,
                value: Some(12345),
            }],
            1,
        )
        .unwrap();
    session.set_level(manual, 0, 2).unwrap();
    let frame = *session.frame().unwrap();
    for _ in 0..10 {
        assert_eq!(session.manual_values(manual).unwrap()[0], Some(12345));
        assert_eq!(session.observed_ms(), 2);
        assert_eq!(session.frame(), Some(&frame));
    }
    session.control(manual, Command::Stop, 3).unwrap();
    assert_eq!(session.manual_values(manual).unwrap()[0], None);
}
