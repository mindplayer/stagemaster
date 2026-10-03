#[path = "support/media.rs"]
mod media_support;
mod support;
use media_support::*;
use stagemaster_live::{Change, Command, Session, media::Status};
use stagemaster_project::PackageSelection;
use support::*;

#[test]
fn group_members_start_pause_and_release_together_without_touching_another_source() {
    let doc = document();
    let mut sources = specs();
    sources.push(playback(4, PackageSelection::Scene { id: id(5) }));
    let mut spec = group();
    spec.sources.push([4; 16]);
    let mut session = Session::prepare_with_media(&doc, [9; 16], &sources, &[spec], 1000).unwrap();
    let map = mapping(session.host_clock(), 200);
    session
        .control(session.key([2; 16]).unwrap(), Command::Execute(0), 1000)
        .unwrap();
    start(&doc, &mut session, &map, 0, 1000);
    session
        .observe_media(
            session.media_key(GROUP).unwrap(),
            sample(2, 20, false, 1020),
            &map,
            1021,
        )
        .unwrap();
    session.tick(1500).unwrap();
    assert_frame(&doc, &session, &[50_000, 10_000, 0, 50_000]);
    assert_eq!(
        session.media_groups().next().unwrap().status,
        Status::Paused
    );
    assert!(
        session
            .sources()
            .filter(|s| s.id == [1; 16] || s.id == [4; 16])
            .all(|s| s.status == Some(stagemaster_playback::Status::Paused))
    );
    session
        .stop_media(session.media_key(GROUP).unwrap(), 1500)
        .unwrap();
    assert_frame(&doc, &session, &[0, 0, 0, 50_000]);
}

#[test]
fn authored_fade_at_a_media_position_is_independent_of_live_baseline_and_seek_history() {
    let mut raw = fixture(0);
    let mut second = step(11, 3, 0, None);
    second["fade"]["ticks"] = serde_json::json!("100");
    raw["lighting"]["sequences"][0] = list(20, &[step(10, 1, 0, Some(60)), second], false);
    let doc = decode(&raw);
    let mut session =
        Session::prepare_with_media(&doc, [9; 16], &specs(), &[group()], 1000).unwrap();
    let map = mapping(session.host_clock(), 200);
    start(&doc, &mut session, &map, 0, 1000);
    session
        .patch(
            session.key([3; 16]).unwrap(),
            &[Change {
                attribute: 1,
                value: Some(40_000),
            }],
            1010,
        )
        .unwrap();
    session
        .observe_media(
            session.media_key(GROUP).unwrap(),
            sample(2, 110, true, 1110),
            &map,
            1111,
        )
        .unwrap();
    assert_eq!(session.winner(1), Some([1; 16])); // newly activated step
    assert_eq!(session.values().unwrap()[1], 20_000); // half way 10,000 -> 30,000
    start(&doc, &mut session, &map, 100, 1120);
    assert_eq!(session.values().unwrap()[1], 18_000);
    start(&doc, &mut session, &map, 110, 1130);
    assert_eq!(session.values().unwrap()[1], 20_000);
}

#[test]
fn preparation_worker_needs_no_running_session_access_and_late_results_are_refused() {
    let (doc, mut session, map) = setup();
    let key = session.media_key(GROUP).unwrap();
    let preparer = session.media_preparer(key).unwrap();
    let (go, wait) = std::sync::mpsc::sync_channel(1);
    let job = std::thread::spawn(move || {
        wait.recv().unwrap();
        preparer.prepare(key, &doc, 60, true, 1600).unwrap()
    });
    session.tick(1500).unwrap();
    assert_eq!(session.values().unwrap()[3], 50_000);
    assert_eq!(session.media_groups().next().unwrap().status, Status::Ready);
    session.tick(2000).unwrap();
    assert_eq!(session.values().unwrap()[3], 10_000);
    go.send(()).unwrap();
    let mut late = job.join().unwrap();
    let before = *session.frame().unwrap();
    assert!(
        session
            .activate_media(&mut late, sample(1, 60, true, 2000), &map, 2001)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&before));
}
