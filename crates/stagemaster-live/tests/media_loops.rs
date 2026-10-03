#[path = "support/audio_timeline.rs"]
mod audio_timeline;
#[path = "support/media.rs"]
#[allow(dead_code)]
mod media_support;
mod support;
use audio_timeline::*;
use media_support::*;
use serde_json::json;
use stagemaster_live::{
    Change, Command, Session,
    media::{MediaProgress, Sample},
};
use stagemaster_playback::LoopPlayback;
use stagemaster_project::{AudioLoopPlays, AudioLoopRegion, Document};
use support::id;

fn looping_doc() -> Document {
    let mut doc = document_with_audio();
    let regions = vec![AudioLoopRegion {
        id: id(99),
        name: "交叉循环".into(),
        start_ms: 100,
        end_ms: 300,
        plays: AudioLoopPlays::Count { count: 4 },
        enabled: true,
        locked: false,
    }];
    let mut raw: serde_json::Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    raw["requires"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"media.audio-loop-regions","version":1}));
    raw["media"]["audioEditing"]["loopRegions"] = json!(regions);
    doc = Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    doc
}
fn observation(
    seq: u64,
    consumed: u64,
    cursor: stagemaster_playback::LoopPosition,
    at: u64,
    playing: bool,
) -> Sample {
    let mut s = sample(seq, cursor.tick, playing, at);
    s.progress = Some(MediaProgress {
        instance: 1,
        sample_rate: 1000,
        position_ticks: cursor.tick,
        consumed_ticks: consumed,
        repeated_ticks: cursor.repeated_ticks,
    });
    s
}
fn activate(doc: &Document, session: &mut Session, first: Sample) -> stagemaster_time::Mapping {
    let map = mapping(session.host_clock(), 200);
    let key = session.media_key(GROUP).unwrap();
    let mut plan = session
        .media_preparer(key)
        .unwrap()
        .prepare(key, doc, first.position_ms, first.playing, 1020)
        .unwrap();
    session
        .activate_media(&mut plan, first, &map, 1001)
        .unwrap();
    map
}

#[test]
fn sparse_real_loop_path_samples_precompiled_crossfades_and_preserves_other_sources() {
    let doc = looping_doc();
    let mut s = prepared(&doc);
    let mut cursor = LoopPlayback::new(
        doc.audio_timeline()
            .unwrap()
            .compile_loops(1000)
            .unwrap()
            .schedule,
        0,
    )
    .unwrap();
    s.control(s.key([2; 16]).unwrap(), Command::Execute(0), 1000)
        .unwrap();
    let map = activate(
        &doc,
        &mut s,
        observation(1, 0, cursor.position(), 1000, true),
    );
    let key = s.media_key(GROUP).unwrap();
    let mut consumed = 0;
    // Third sample skips a whole repeat and ends later in the same clip;
    // the fourth leaves the finite region while retaining cumulative repeat evidence.
    for (seq, delta) in (2..).zip([250, 70, 410, 230, 100]) {
        consumed += delta;
        let position = cursor.advance(delta).unwrap();
        s.observe_media(
            key,
            observation(seq, consumed, position, 1000 + consumed, true),
            &map,
            1001 + consumed,
        )
        .unwrap();
        let expected = reference(&doc, position.tick);
        assert_eq!(&s.values().unwrap()[..3], &expected[..3]);
        assert_eq!(s.winner(3), Some([2; 16]));
        assert_eq!(s.media_key(GROUP), Some(key)); // no new plan/generation on a loop
        let mut values = expected;
        values[3] = s.values().unwrap()[3];
        assert_frame(&doc, &s, &values);
    }
}

#[test]
fn pause_and_ordinary_samples_keep_manual_control_until_the_next_admitted_repeat() {
    let doc = looping_doc();
    let mut s = prepared(&doc);
    let mut cursor = LoopPlayback::new(
        doc.audio_timeline()
            .unwrap()
            .compile_loops(1000)
            .unwrap()
            .schedule,
        150,
    )
    .unwrap();
    let map = activate(
        &doc,
        &mut s,
        observation(1, 0, cursor.position(), 1000, true),
    );
    let key = s.media_key(GROUP).unwrap();
    let manual = s.key([3; 16]).unwrap();
    s.patch(
        manual,
        &[Change {
            attribute: 1,
            value: Some(55_000),
        }],
        1002,
    )
    .unwrap();
    s.observe_media(
        key,
        observation(2, 10, cursor.advance(10).unwrap(), 1010, false),
        &map,
        1011,
    )
    .unwrap();
    s.observe_media(
        key,
        observation(3, 10, cursor.position(), 1100, false),
        &map,
        1101,
    )
    .unwrap();
    assert_eq!(s.winner(1), Some([3; 16]));
    s.observe_media(
        key,
        observation(4, 20, cursor.advance(10).unwrap(), 1110, true),
        &map,
        1111,
    )
    .unwrap();
    assert_eq!(s.winner(1), Some([3; 16]));
    s.observe_media(
        key,
        observation(5, 210, cursor.advance(190).unwrap(), 1300, true),
        &map,
        1301,
    )
    .unwrap();
    assert_eq!(s.winner(1), Some([1; 16]));
    assert_eq!(
        s.values().unwrap()[1],
        reference(&doc, cursor.position().tick)[1]
    );
    s.stop_media(key, 1302).unwrap();
    assert_eq!(s.winner(1), Some([3; 16]));
    assert!(
        s.observe_media(
            key,
            observation(6, 220, cursor.advance(10).unwrap(), 1310, true),
            &map,
            1311
        )
        .is_err()
    );
}

#[test]
fn invalid_loop_evidence_and_unprepared_opt_in_leave_the_whole_frame_unchanged() {
    let doc = looping_doc();
    let mut s = prepared(&doc);
    let mut cursor = LoopPlayback::new(
        doc.audio_timeline()
            .unwrap()
            .compile_loops(1000)
            .unwrap()
            .schedule,
        250,
    )
    .unwrap();
    let map = activate(
        &doc,
        &mut s,
        observation(1, 0, cursor.position(), 1000, true),
    );
    let key = s.media_key(GROUP).unwrap();
    let good = observation(2, 100, cursor.advance(100).unwrap(), 1100, true);
    let before = *s.frame().unwrap();
    for case in 0..8 {
        let mut bad = good;
        let p = bad.progress.as_mut().unwrap();
        match case {
            0 => p.repeated_ticks = 0,
            1 => p.consumed_ticks = 0,
            2 => p.sample_rate = 2000,
            3 => p.sample_rate = 0,
            4 => p.position_ticks += 1,
            5 => {
                p.consumed_ticks += 2000;
                p.repeated_ticks += 2000;
            }
            6 => p.instance = 2,
            _ => bad.progress = None,
        }
        assert!(
            s.observe_media(key, bad, &map, 1101).is_err(),
            "case {case}"
        );
        assert_eq!(s.frame(), Some(&before));
        assert!(s.fault().is_none());
    }
    s.observe_media(key, good, &map, 1101).unwrap();
    let before = *s.frame().unwrap();
    assert!(s.observe_media(key, good, &map, 1102).is_err());
    assert_eq!(s.frame(), Some(&before));
    let linear = document_with_audio();
    let mut s =
        Session::prepare_with_media(&linear, [9; 16], &support::specs(), &[group()], 1000).unwrap();
    let key = s.media_key(GROUP).unwrap();
    let mut plan = s
        .media_preparer(key)
        .unwrap()
        .prepare(key, &linear, 250, true, 1020)
        .unwrap();
    let map = mapping(s.host_clock(), 200);
    assert!(
        s.activate_media(
            &mut plan,
            observation(
                1,
                0,
                LoopPlayback::new(
                    linear
                        .audio_timeline()
                        .unwrap()
                        .compile_loops(1000)
                        .unwrap()
                        .schedule,
                    250
                )
                .unwrap()
                .position(),
                1000,
                true
            ),
            &map,
            1001
        )
        .is_err()
    );
}

#[test]
fn linear_pcm_cannot_claim_a_repeat_and_seek_verifies_exact_sample_quantization() {
    let doc = document_with_audio();
    let mut s = prepared(&doc);
    let cursor = LoopPlayback::new(
        doc.audio_timeline()
            .unwrap()
            .compile_loops(1000)
            .unwrap()
            .schedule,
        250,
    )
    .unwrap();
    let first = observation(1, 0, cursor.position(), 1000, true);
    let map = activate(&doc, &mut s, first);
    let key = s.media_key(GROUP).unwrap();
    let mut bad = first;
    bad.at = sample(2, 150, true, 1100).at;
    bad.sequence = 2;
    bad.position_ms = 150;
    let p = bad.progress.as_mut().unwrap();
    p.position_ticks = 150;
    p.consumed_ticks = 100;
    p.repeated_ticks = 200;
    let before = *s.frame().unwrap();
    assert!(s.observe_media(key, bad, &map, 1101).is_err());
    assert_eq!(s.frame(), Some(&before));
    let mut actual = first;
    actual.position_ms = 2500;
    actual.progress = Some(MediaProgress {
        instance: 1,
        sample_rate: 44100,
        position_ticks: 110_294,
        consumed_ticks: 0,
        repeated_ticks: 0,
    });
    assert!(actual.matches_seek(2501));
    assert!(!actual.matches_seek(2500));
    actual.progress.as_mut().unwrap().position_ticks += 1;
    assert!(!actual.matches_seek(2501));
}

#[test]
fn integer_millisecond_sources_keep_their_existing_quantization_allowance() {
    let doc = document();
    let mut spec = group();
    spec.limits.position_tolerance_ms = 0;
    let mut session =
        Session::prepare_with_media(&doc, [9; 16], &support::specs(), &[spec], 1000).unwrap();
    let map = mapping(session.host_clock(), 0);
    start(&doc, &mut session, &map, 0, 1000);
    let key = session.media_key(GROUP).unwrap();
    let mut next = sample(2, 1, true, 1001);
    next.at = provider().at(10_000_500_000);
    session.observe_media(key, next, &map, 1002).unwrap();
    let before = *session.frame().unwrap();
    next.sequence = 3;
    next.position_ms = 3;
    next.at = provider().at(10_000_750_000);
    assert!(session.observe_media(key, next, &map, 1003).is_err());
    assert_eq!(session.frame(), Some(&before));
}
