#[path = "support/media.rs"]
#[allow(dead_code)]
mod media_support;
mod support;
use media_support::*;
use serde_json::json;
use stagemaster_live::{Change, Command, PlaybackSelection, Session};
use stagemaster_project::{Document, EditCommand};
use support::*;

fn document_with_audio() -> Document {
    let mut doc = document();
    for command in [
        json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":1000}}),
        json!({"kind":"convertLightingClips"}),
        json!({"kind":"addLightingClip","name":"第一段","sceneId":id(1),"startMs":0,"endMs":200,"fadeMs":0}),
        json!({"kind":"addLightingClip","name":"交叉段","sceneId":id(3),"startMs":200,"endMs":400,"fadeMs":100,"fadeMode":"dynamic"}),
        json!({"kind":"addLightingClip","name":"第三段","sceneId":id(2),"startMs":500,"endMs":900,"fadeMs":100}),
    ] {
        doc.edit(EditCommand::Audio {
            command: serde_json::from_value(command).unwrap(),
        })
        .unwrap();
    }
    doc
}
fn prepared(doc: &Document) -> Session {
    let mut sources = specs();
    sources[0].playback = Some(PlaybackSelection::AudioTimeline);
    Session::prepare_with_media(doc, [9; 16], &sources, &[group()], 1000).unwrap()
}
fn reference(doc: &Document, position: u64) -> Vec<u16> {
    let track = doc.audio_timeline().unwrap();
    let clip = track.lighting_at(position);
    let compiled = doc.compile_audio_segment(clip.map(|c| c.id)).unwrap();
    let mut player = compiled.playback.into_player().unwrap();
    player
        .advance(position - clip.map_or(0, |c| c.start_ms))
        .unwrap();
    player.values().to_vec()
}
#[test]
fn actual_authored_clips_share_composition_with_autonomous_effect_and_manual_ownership() {
    let doc = document_with_audio();
    let mut session = prepared(&doc);
    let map = mapping(session.host_clock(), 200);
    let autonomous = session.key([2; 16]).unwrap();
    let manual = session.key([3; 16]).unwrap();
    session
        .control(autonomous, Command::Execute(0), 1000)
        .unwrap();
    start(&doc, &mut session, &map, 0, 1000);
    let key = session.media_key(GROUP).unwrap();
    let timeline = session.key([1; 16]).unwrap();
    assert!(session.control(timeline, Command::Stop, 1001).is_err());
    for (seq, position) in (2..).zip([
        10, 100, 199, 200, 250, 299, 300, 399, 400, 500, 550, 600, 900, 1000,
    ]) {
        session
            .observe_media(
                key,
                sample(seq, position, true, position + 1000),
                &map,
                position + 1001,
            )
            .unwrap();
        let expected = reference(&doc, position);
        let actual = session.values().unwrap();
        assert_eq!(&actual[..3], &expected[..3], "position {position}");
        assert!(actual[3] >= 10_000); // Unused blue belongs to the autonomous source.
        assert_eq!(session.winner(3), Some([2; 16]));
        assert_frame(
            &doc,
            &session,
            &[expected[0], expected[1], expected[2], actual[3]],
        );
    }
    // A fresh prepared generation is the only path back to an earlier material position.
    start(&doc, &mut session, &map, 10, 2020);
    let key = session.media_key(GROUP).unwrap();
    session
        .patch(
            manual,
            &[Change {
                attribute: 1,
                value: Some(50_000),
            }],
            2022,
        )
        .unwrap();
    session
        .observe_media(key, sample(2, 20, false, 2030), &map, 2031)
        .unwrap();
    assert_eq!(session.values().unwrap()[1], 50_000);
    session
        .observe_media(key, sample(3, 20, false, 2040), &map, 2041)
        .unwrap();
    assert_eq!(session.winner(1), Some([3; 16]));
    session
        .observe_media(key, sample(4, 30, true, 2050), &map, 2051)
        .unwrap();
    assert_eq!(session.winner(1), Some([3; 16]));
    session
        .observe_media(key, sample(5, 200, true, 2220), &map, 2221)
        .unwrap();
    assert_eq!(session.winner(1), Some([1; 16]));
    assert_eq!(session.values().unwrap()[1], reference(&doc, 200)[1]);
    session.stop_media(key, 2222).unwrap();
    assert_eq!(session.winner(1), Some([3; 16]));
    assert_eq!(session.winner(3), Some([2; 16]));
}
#[test]
fn missing_media_binding_changed_snapshot_and_out_of_range_samples_are_rejected_atomically() {
    let doc = document_with_audio();
    let mut sources = specs();
    sources[0].playback = Some(PlaybackSelection::AudioTimeline);
    assert!(Session::prepare(&doc, [9; 16], &sources, 1000).is_err());
    assert!(Session::prepare_with_media(&doc, [9; 16], &sources, &[], 1000).is_err());
    let mut session = prepared(&doc);
    let map = mapping(session.host_clock(), 200);
    start(&doc, &mut session, &map, 900, 1000);
    let key = session.media_key(GROUP).unwrap();
    let before = *session.frame().unwrap();
    assert!(
        session
            .observe_media(key, sample(2, 1001, true, 1101), &map, 1102)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&before));
    assert!(session.fault().is_none());
    let mut changed = doc.clone();
    changed.edit(serde_json::from_value(json!({"op":"audio","command":{"kind":"editLightingClips","ids":[doc.audio_timeline().unwrap().lighting_clips.unwrap()[0].id],"action":{"kind":"enabled","enabled":false}}})).unwrap()).unwrap();
    let prepare = session.media_preparer(key).unwrap();
    assert!(prepare.prepare(key, &changed, 0, true, 1200).is_err());
    assert!(prepare.prepare(key, &doc, 1001, true, 1200).is_err());
    assert_eq!(session.frame(), Some(&before));
}

#[test]
fn jumping_between_gaps_still_applies_crossed_enabled_clip_end() {
    for enabled in [true, false] {
        let mut doc = document_with_audio();
        let clip = doc.audio_timeline().unwrap().lighting_clips.unwrap()[2]
            .id
            .clone();
        doc.edit(
            serde_json::from_value(json!({"op":"audio","command":{
                "kind":"editLightingClips","ids":[clip],
                "action":{"kind":"enabled","enabled":enabled}
            }}))
            .unwrap(),
        )
        .unwrap();
        let mut session = prepared(&doc);
        let map = mapping(session.host_clock(), 200);
        start(&doc, &mut session, &map, 450, 1450);
        let key = session.media_key(GROUP).unwrap();
        session
            .patch(
                session.key([3; 16]).unwrap(),
                &[Change {
                    attribute: 1,
                    value: Some(50_000),
                }],
                1452,
            )
            .unwrap();
        session
            .observe_media(key, sample(2, 950, true, 1950), &map, 1951)
            .unwrap();
        assert_eq!(
            session.values().unwrap()[1],
            if enabled { 0 } else { 50_000 }
        );
        assert_eq!(
            session.winner(1),
            Some(if enabled { [1; 16] } else { [3; 16] })
        );
        // After this one transition, ordinary empty-range samples must not steal again.
        session
            .patch(
                session.key([3; 16]).unwrap(),
                &[Change {
                    attribute: 1,
                    value: Some(45_000),
                }],
                1952,
            )
            .unwrap();
        session
            .observe_media(key, sample(3, 960, true, 1960), &map, 1961)
            .unwrap();
        assert_eq!(session.values().unwrap()[1], 45_000);
    }
}
