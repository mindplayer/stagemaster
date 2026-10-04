mod support;
use stagemaster_live::{Change, Command, OutputCommand, Session};
use support::*;

#[test]
fn invalid_master_is_atomic_and_blackout_does_not_stop_timed_steps_or_release_manual_zero() {
    let doc = decode(&fixture(0));
    let before = doc.encode().unwrap();
    let mut sources = specs();
    sources[2].priority = 10; // Match the desktop's explicit higher-priority manual layer.
    let mut session = Session::prepare(&doc, [9; 16], &sources, 0).unwrap();
    let one = session.key([1; 16]).unwrap();
    let two = session.key([2; 16]).unwrap();
    let hand = session.key([3; 16]).unwrap();
    session.control(one, Command::Execute(0), 0).unwrap();
    session.control(two, Command::Execute(0), 0).unwrap();
    let frame = *session.frame().unwrap();
    assert!(
        session
            .control_output(OutputCommand::Level { percent: 101 }, 100)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&frame));
    assert_eq!(session.observed_ms(), 0);
    assert_eq!(session.output_master().percent(), 100);
    session
        .control_output(OutputCommand::Blackout { enabled: true }, 10)
        .unwrap();
    session.tick(100).unwrap();
    assert_eq!(session.sources().next().unwrap().step, Some(2));
    assert_eq!(session.values().unwrap()[0], 0);
    session
        .control_output(OutputCommand::Blackout { enabled: false }, 100)
        .unwrap();
    assert_eq!(session.values().unwrap()[0], 25_000);
    session
        .patch(
            hand,
            &[Change {
                attribute: 0,
                value: Some(0),
            }],
            101,
        )
        .unwrap();
    let winner = session.winner(0);
    session
        .control_output(OutputCommand::Level { percent: 0 }, 102)
        .unwrap();
    assert_eq!(session.winner(0), winner);
    assert_eq!(session.manual_values(hand).unwrap()[0], Some(0));
    assert!(
        session
            .control_output(OutputCommand::Level { percent: 50 }, 101)
            .is_err()
    );
    assert_eq!(session.output_master().percent(), 0);
    session
        .control_output(OutputCommand::Level { percent: 100 }, 103)
        .unwrap();
    assert_eq!(session.values().unwrap()[0], 0);
    session
        .patch(
            hand,
            &[Change {
                attribute: 0,
                value: None,
            }],
            104,
        )
        .unwrap();
    assert_eq!(session.values().unwrap()[0], 25_000);
    assert_eq!(doc.encode().unwrap(), before);
}
