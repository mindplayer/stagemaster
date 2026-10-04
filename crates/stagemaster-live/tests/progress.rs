mod support;
use stagemaster_live::{Command, Phase, Session};
use support::*;
#[test]
fn source_progress_tracks_real_list_and_stays_independent_of_other_layers() {
    let mut session = Session::prepare(&decode(&fixture(50)), [9; 16], &specs(), 0).unwrap();
    let key = session.key([1; 16]).unwrap();
    assert_eq!(session.sources().nth(2).unwrap().progress, None);
    session.control(key, Command::Execute(0), 0).unwrap();
    session.tick(65).unwrap();
    let observed: Vec<_> = session.sources().collect();
    let info = observed[0].progress.unwrap();
    assert_eq!(observed[0].step, Some(1));
    assert_eq!(info.phase, Phase::Wait);
    assert_eq!(info.elapsed_ms, 5);
    assert_eq!(info.phase_duration_ms, Some(20));
    assert_eq!(info.next_step, Some(2));
    assert_eq!(observed[1].progress.unwrap().phase, Phase::Idle);
    let frame = *session.frame().unwrap();
    assert_eq!(observed, session.sources().collect::<Vec<_>>());
    assert_eq!(frame, *session.frame().unwrap());
    session.control(key, Command::Pause, 65).unwrap();
    session.tick(100_000).unwrap();
    assert_eq!(session.sources().next().unwrap().progress.unwrap(), info);
}
