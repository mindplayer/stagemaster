#[path = "support/media.rs"]
mod media_support;
mod support;
use media_support::*;
use stagemaster_live::{Change, Command, media::Status};

#[test]
fn pause_seek_and_loss_do_not_retime_autonomous_effects_or_host_frames() {
    let (doc, mut session, map) = setup();
    start(&doc, &mut session, &map, 0, 1000);
    let first_run = session.media_key(GROUP).unwrap();
    session
        .observe_media(first_run, sample(2, 60, false, 1100), &map, 1101)
        .unwrap();
    session.tick(1500).unwrap();
    assert_frame(&doc, &session, &[25_000, 10_000, 0, 50_000]);
    assert_eq!(
        session.media_groups().next().unwrap().status,
        Status::Paused
    );
    // Paused heartbeat doesn't run the media list forward or renew host time on reads.
    session
        .observe_media(first_run, sample(3, 60, false, 1600), &map, 1601)
        .unwrap();
    session.tick(2000).unwrap();
    assert_frame(&doc, &session, &[25_000, 10_000, 0, 10_000]);
    // Resuming must consume the first delta rather than losing it in Player::Resume.
    session
        .observe_media(first_run, sample(4, 80, true, 2050), &map, 2051)
        .unwrap();
    session.tick(2500).unwrap();
    assert_frame(&doc, &session, &[25_000, 30_000, 0, 50_000]);
    // A backward seek replaces only the media group's prepared players.
    start(&doc, &mut session, &map, 0, 2600);
    let second_run = session.media_key(GROUP).unwrap();
    assert_ne!(first_run, second_run);
    session.tick(3000).unwrap();
    assert_frame(&doc, &session, &[0, 10_000, 0, 10_000]);
    session.tick(3601).unwrap();
    assert_eq!(session.media_groups().next().unwrap().status, Status::Lost);
    assert_eq!(
        session.sources().next().unwrap().status,
        Some(stagemaster_playback::Status::Paused)
    );
    let held = *session.frame().unwrap();
    assert!(
        session
            .observe_media(second_run, sample(2, 80, true, 3601), &map, 3602)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&held));
    session.tick(4000).unwrap();
    assert_frame(&doc, &session, &[0, 10_000, 0, 10_000]);
    start(&doc, &mut session, &map, 80, 4100);
    session.tick(4500).unwrap();
    assert_frame(&doc, &session, &[25_000, 30_000, 0, 50_000]);
    assert_eq!(session.observed_ms(), 4500);
}

#[test]
fn ordinary_samples_preserve_manual_takeover_but_seek_reasserts_and_stop_releases_only_group() {
    let (doc, mut session, map) = setup();
    start(&doc, &mut session, &map, 0, 1000);
    let key = session.media_key(GROUP).unwrap();
    let manual = session.key([3; 16]).unwrap();
    session
        .patch(
            manual,
            &[Change {
                attribute: 1,
                value: Some(40_000),
            }],
            1020,
        )
        .unwrap();
    session
        .observe_media(key, sample(2, 30, true, 1030), &map, 1031)
        .unwrap();
    assert_eq!(session.winner(1), Some([3; 16]));
    session
        .observe_media(key, sample(3, 60, false, 1060), &map, 1061)
        .unwrap();
    assert_eq!(session.winner(1), Some([3; 16])); // inherited red does not reassert
    start(&doc, &mut session, &map, 60, 1080);
    assert_eq!(session.winner(1), Some([1; 16])); // deliberate locate does
    assert_eq!(session.values().unwrap()[1], 10_000);
    let media_source = session.key([1; 16]).unwrap();
    let before = *session.frame().unwrap();
    assert!(
        session
            .control(media_source, Command::Execute(0), 1090)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&before));
    session
        .stop_media(session.media_key(GROUP).unwrap(), 1100)
        .unwrap();
    session.tick(1500).unwrap();
    assert_frame(&doc, &session, &[0, 40_000, 0, 50_000]);
    assert_eq!(
        session.media_groups().next().unwrap().status,
        Status::Stopped
    );
}

#[test]
fn repeated_stop_after_a_far_forward_locate_does_not_mix_media_and_host_origins() {
    let (doc, mut session, map) = setup();
    start(&doc, &mut session, &map, 1_000_000, 1000);
    session
        .stop_media(session.media_key(GROUP).unwrap(), 1010)
        .unwrap();
    session
        .stop_media(session.media_key(GROUP).unwrap(), 1020)
        .unwrap();
    assert!(session.fault().is_none());
    session.tick(1500).unwrap();
    assert_frame(&doc, &session, &[0, 0, 0, 50_000]);
}
