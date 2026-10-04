mod support;
use stagemaster_live::{BatchCommand, Change, Command, Session};
use stagemaster_playback::Status;
use support::*;

fn prepared() -> Session {
    let mut s = Session::prepare(&decode(&fixture(70)), [9; 16], &specs(), 0).unwrap();
    for n in [1, 2] {
        s.control(s.key([n; 16]).unwrap(), Command::Execute(0), 0)
            .unwrap();
    }
    s.patch(
        s.key([3; 16]).unwrap(),
        &[Change {
            attribute: 2,
            value: Some(0),
        }],
        0,
    )
    .unwrap();
    s
}

#[test]
fn pause_resume_share_time_and_stop_releases_only_the_selected_programs() {
    let mut s = prepared();
    let keys = [1, 2].map(|n| s.key([n; 16]).unwrap());
    s.control_output(stagemaster_live::OutputCommand::Level { percent: 37 }, 0)
        .unwrap();
    s.control_output(
        stagemaster_live::OutputCommand::Blackout { enabled: true },
        0,
    )
    .unwrap();
    s.control_batch(&keys, BatchCommand::Pause, 30).unwrap();
    let before: Vec<_> = s.sources().collect();
    assert!(before[..2].iter().all(|s| s.status == Some(Status::Paused)));
    assert_eq!(before[0].progress.unwrap().elapsed_ms, 30);
    assert_eq!(before[1].progress.unwrap().elapsed_ms, 30);
    s.tick(200).unwrap();
    assert_eq!(
        s.sources().take(2).map(|s| s.progress).collect::<Vec<_>>(),
        before[..2].iter().map(|s| s.progress).collect::<Vec<_>>()
    );
    s.control_batch(&keys, BatchCommand::Resume, 200).unwrap();
    s.tick(220).unwrap();
    assert!(
        s.sources()
            .take(2)
            .all(|s| s.status == Some(Status::Running) && s.progress.unwrap().elapsed_ms == 50)
    );
    s.control_batch(&keys[..1], BatchCommand::Stop, 220)
        .unwrap();
    let states: Vec<_> = s.sources().collect();
    assert_eq!(states[0].status, Some(Status::Idle));
    assert_eq!(states[1].status, Some(Status::Running));
    assert_eq!(
        s.manual_values(s.key([3; 16]).unwrap()).unwrap()[2],
        Some(0)
    );
    assert_eq!(s.output_master().percent(), 37);
    assert!(s.output_master().blackout());
    s.control_batch(&keys, BatchCommand::Stop, 230).unwrap();
    assert!(s.sources().take(2).all(|s| s.status == Some(Status::Idle)));
    assert_eq!(s.winner(2), Some([3; 16]));
}

#[test]
fn empty_duplicate_budget_foreign_manual_and_backwards_time_reject_before_mutation() {
    let mut s = prepared();
    let a = s.key([1; 16]).unwrap();
    let manual = s.key([3; 16]).unwrap();
    let other = Session::prepare(&decode(&fixture(70)), [8; 16], &specs(), 0).unwrap();
    let foreign = other.key([2; 16]).unwrap();
    let frame = *s.frame().unwrap();
    let before: Vec<_> = s.sources().collect();
    for keys in [
        vec![],
        vec![a, a],
        vec![a; 65],
        vec![a, foreign],
        vec![a, manual],
    ] {
        assert!(s.control_batch(&keys, BatchCommand::Stop, 50).is_err());
        assert_eq!(*s.frame().unwrap(), frame);
        assert_eq!(s.sources().collect::<Vec<_>>(), before);
        assert!(s.fault().is_none());
    }
    s.tick(10).unwrap();
    let frame = *s.frame().unwrap();
    assert!(s.control_batch(&[a], BatchCommand::Pause, 9).is_err());
    assert_eq!(*s.frame().unwrap(), frame);
}

#[test]
fn mixed_idle_paused_and_running_use_existing_noop_rules() {
    let mut s = prepared();
    let keys = [1, 2].map(|n| s.key([n; 16]).unwrap());
    s.control(keys[0], Command::Stop, 0).unwrap();
    s.control(keys[1], Command::Pause, 0).unwrap();
    s.control_batch(&keys, BatchCommand::Resume, 20).unwrap();
    assert_eq!(s.sources().next().unwrap().status, Some(Status::Idle));
    assert_eq!(s.sources().nth(1).unwrap().status, Some(Status::Running));
    s.control_batch(&keys, BatchCommand::Pause, 30).unwrap();
    assert_eq!(s.sources().next().unwrap().status, Some(Status::Idle));
    s.control_batch(&keys, BatchCommand::Stop, 30).unwrap();
    s.control_batch(&keys, BatchCommand::Resume, 40).unwrap();
    assert!(s.sources().take(2).all(|s| s.status == Some(Status::Idle)));
}

#[test]
fn finished_program_is_not_restarted_by_pause_or_resume_and_stop_releases_it() {
    let mut raw = fixture(70);
    raw["lighting"]["sequences"][0] = list(20, &[step(10, 1, 0, Some(10))], false);
    let mut s = Session::prepare(&decode(&raw), [9; 16], &specs(), 0).unwrap();
    let key = s.key([1; 16]).unwrap();
    s.control(key, Command::Execute(0), 0).unwrap();
    s.tick(20).unwrap();
    assert_eq!(s.sources().next().unwrap().status, Some(Status::Finished));
    let held = s.values().unwrap().to_vec();
    for command in [BatchCommand::Pause, BatchCommand::Resume] {
        s.control_batch(&[key], command, 20).unwrap();
        assert_eq!(s.sources().next().unwrap().status, Some(Status::Finished));
        assert_eq!(s.values().unwrap(), held);
    }
    s.control_batch(&[key], BatchCommand::Stop, 20).unwrap();
    assert_eq!(s.sources().next().unwrap().status, Some(Status::Idle));
    assert_eq!(s.winner(1), None);
}

#[path = "support/media.rs"]
mod media_support;
#[test]
fn an_ordinary_program_following_media_rejects_the_entire_batch() {
    let (doc, mut s, map) = media_support::setup();
    media_support::start(&doc, &mut s, &map, 0, 1000);
    let keys = [2, 1].map(|n| s.key([n; 16]).unwrap());
    let frame = *s.frame().unwrap();
    let before: Vec<_> = s.sources().collect();
    assert!(s.control_batch(&keys, BatchCommand::Stop, 1010).is_err());
    assert_eq!(s.sources().collect::<Vec<_>>(), before);
    assert_eq!(*s.frame().unwrap(), frame);
}

#[test]
fn all_64_unique_ordinary_targets_are_bounded_and_do_not_start_idle_programs() {
    let doc = decode(&fixture(70));
    let sources: Vec<_> = (1..=64)
        .map(|n| {
            playback(
                n,
                stagemaster_project::PackageSelection::Scene { id: id(1) },
            )
        })
        .collect();
    let mut s = Session::prepare(&doc, [9; 16], &sources, 0).unwrap();
    let keys: Vec<_> = (1..=64).map(|n| s.key([n; 16]).unwrap()).collect();
    s.control(keys[0], Command::Execute(0), 0).unwrap();
    s.control_batch(&keys, BatchCommand::Pause, 20).unwrap();
    assert_eq!(s.sources().next().unwrap().status, Some(Status::Paused));
    assert!(s.sources().skip(1).all(|s| s.status == Some(Status::Idle)));
    s.control_batch(&keys, BatchCommand::Resume, 30).unwrap();
    assert_eq!(s.sources().next().unwrap().status, Some(Status::Running));
    assert!(s.sources().skip(1).all(|s| s.status == Some(Status::Idle)));
    s.control_batch(&keys, BatchCommand::Stop, 40).unwrap();
    assert!(s.sources().all(|s| s.status == Some(Status::Idle)));
}
