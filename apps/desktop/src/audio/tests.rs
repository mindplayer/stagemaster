use super::AudioPreview;
use serde_json::json;
use stagemaster_project::{AudioAsset, AudioMarker, AudioTimeline, Document};
fn document() -> (Document, AudioTimeline) {
    let mut doc = Document::new("音乐效果定位").unwrap();
    let view = doc.view();
    for index in 0..4 {
        doc.edit(serde_json::from_value(json!({"op":"addFixture","name":format!("灯 {index}"),"profileId":view.profiles[1].id,"domainId":view.domains[0].id,"universe":1,"address":1+index*4})).unwrap()).unwrap();
    }
    doc.edit(serde_json::from_value(json!({"op":"addScene","name":"追逐"})).unwrap())
        .unwrap();
    doc.edit(serde_json::from_value(json!({"op":"addScene","name":"暗场"})).unwrap())
        .unwrap();
    let view = doc.view();
    let scene = &view.scenes[0].id;
    doc.edit(serde_json::from_value(json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{"id":"29999999-0000-4000-8000-000000000001","name":"追逐","enabled":true,"fixtureIds":view.fixtures.iter().map(|f|&f.id).collect::<Vec<_>>(),"periodMs":1000,"spreadDegrees":360,"phaseDegrees":0,"reverse":false,"waveform":"pulse","dutyPercent":25,"channels":[{"attribute":"dimmer","low":0,"high":65535}]}}})).unwrap()).unwrap();
    let track = AudioTimeline {
        asset: AudioAsset {
            digest: "ab".repeat(32),
            file_name: "音乐.wav".into(),
            extension: "wav".into(),
            duration_ms: 10000,
        },
        in_ms: 500,
        out_ms: 9500,
        lighting_clips: None,
        markers: vec![
            AudioMarker {
                id: "a0000000-0000-4000-8000-000000000001".into(),
                name: "追逐开始".into(),
                fade_ms: 0,
                time_ms: 1000,
                scene_id: Some(scene.clone()),
            },
            AudioMarker {
                id: "a0000000-0000-4000-8000-000000000002".into(),
                name: "暗场开始".into(),
                fade_ms: 0,
                time_ms: 4000,
                scene_id: Some(view.scenes[1].id.clone()),
            },
        ],
    };
    doc.edit(stagemaster_project::EditCommand::Audio {
        command: stagemaster_project::AudioEdit::SetAsset {
            asset: track.asset.clone(),
        },
    })
    .unwrap();
    doc.edit(stagemaster_project::EditCommand::Audio {
        command: stagemaster_project::AudioEdit::Trim {
            in_ms: track.in_ms,
            out_ms: track.out_ms,
        },
    })
    .unwrap();
    for marker in &track.markers {
        doc.edit(stagemaster_project::EditCommand::Audio {
            command: stagemaster_project::AudioEdit::PutMarker {
                marker: marker.clone(),
            },
        })
        .unwrap();
    }
    (doc, track)
}
#[test]
fn backward_seek_rebuilds_effect_phase_and_marker_boundary_is_exact() {
    let (doc, track) = document();
    let mut preview = AudioPreview::default();
    preview
        .load("unused-for-paused-render.wav".into(), track)
        .unwrap();
    for (time, lit) in [
        (999, None),
        (1000, Some(0)),
        (1250, Some(4)),
        (4000, None),
        (1250, Some(4)),
    ] {
        preview.transport.seek(time).unwrap();
        let out = preview
            .render(&doc, 1, stagemaster_playback::OutputMaster::default())
            .unwrap()
            .output
            .unwrap();
        let active = out
            .slots
            .iter()
            .enumerate()
            .filter(|(_, v)| **v > 0)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        assert_eq!(
            active,
            lit.into_iter().collect::<Vec<_>>(),
            "position {time}"
        );
    }
    assert!(!preview.position().playing);
    assert!(preview.transport.seek(9001).is_err());
    preview.clear();
    assert!(!preview.active());
    assert_eq!(preview.position().duration_ms, 0);
}
#[test]
fn changing_asset_or_range_clears_old_transport_but_marker_edits_preserve_cursor() {
    let (mut doc, track) = document();

    let current = doc.audio_timeline().unwrap();
    let mut preview = AudioPreview::default();
    preview.load("unused.wav".into(), current).unwrap();
    preview.transport.seek(1200).unwrap();
    doc.edit(stagemaster_project::EditCommand::Audio {
        command: stagemaster_project::AudioEdit::PutMarker {
            marker: track.markers[0].clone(),
        },
    })
    .unwrap();
    preview.synchronize(&doc);
    assert_eq!(preview.position().position_ms, 1200);
    doc.edit(stagemaster_project::EditCommand::Audio {
        command: stagemaster_project::AudioEdit::Trim {
            in_ms: 100,
            out_ms: 9000,
        },
    })
    .unwrap();
    preview.synchronize(&doc);
    assert!(!preview.active());
}

#[test]
fn native_cursor_seek_uses_transition_snapshot_and_retains_pause_state() {
    let (mut doc, mut track) = document();
    track.markers[1].fade_ms = 1000;
    doc.edit(stagemaster_project::EditCommand::Audio {
        command: stagemaster_project::AudioEdit::PutMarker {
            marker: track.markers[1].clone(),
        },
    })
    .unwrap();
    let mut preview = AudioPreview::default();
    preview
        .load("unused-for-paused-render.wav".into(), track)
        .unwrap();
    for (time, level) in [
        (4000, 255),
        (4500, 128),
        (5000, 0),
        (4500, 128),
        (4000, 255),
    ] {
        preview.transport.seek(time).unwrap();
        let result = preview
            .render(&doc, 1, stagemaster_playback::OutputMaster::default())
            .unwrap();
        assert_eq!(result.status, "paused");
        let output = result.output.unwrap();
        assert_eq!(output.slots[0], level, "at {time}");
        assert!(output.slots[1..].iter().all(|slot| *slot == 0));
        assert!(!preview.position().playing);
    }
}

#[test]
fn explicit_clip_gap_copy_and_backward_seek_use_native_cursor_without_reloading_audio() {
    let (mut doc, _) = document();
    doc.edit(
        serde_json::from_value(json!({"op":"audio","command":{"kind":"convertLightingClips"}}))
            .unwrap(),
    )
    .unwrap();
    let clips = doc.audio_timeline().unwrap().lighting_clips.unwrap();
    let mut first = clips[0].clone();
    first.end_ms = 2000;
    doc.edit(
        serde_json::from_value(
            json!({"op":"audio","command":{"kind":"putLightingClip","clip":first}}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut preview = AudioPreview::default();
    preview
        .load("unused.wav".into(), doc.audio_timeline().unwrap())
        .unwrap();
    for (time, lit) in [(1250, Some(4)), (2000, None), (3000, None), (1250, Some(4))] {
        preview.transport.seek(time).unwrap();
        let result = preview
            .render(&doc, 2, stagemaster_playback::OutputMaster::default())
            .unwrap();
        let out = result.output.unwrap();
        assert_eq!(
            out.slots
                .iter()
                .enumerate()
                .filter(|(_, v)| **v > 0)
                .map(|(i, _)| i)
                .collect::<Vec<_>>(),
            lit.into_iter().collect::<Vec<_>>()
        );
    }
    let saved = preview.position().position_ms;
    doc.edit(serde_json::from_value(json!({"op":"audio","command":{"kind":"copyLightingClip","id":first.id,"startMs":2500}})).unwrap()).unwrap();
    preview.synchronize(&doc);
    assert_eq!(preview.position().position_ms, saved);
    preview.transport.seek(2750).unwrap();
    let out = preview
        .render(&doc, 3, stagemaster_playback::OutputMaster::default())
        .unwrap()
        .output
        .unwrap();
    assert_eq!(out.slots[4], 255);
    assert!(!preview.position().playing);
}

#[test]
fn music_master_restores_current_effect_without_seeking_or_rebuilding_transport() {
    let (doc, track) = document();
    let mut preview = AudioPreview::default();
    preview.load("unused-paused.wav".into(), track).unwrap();
    let mut master = stagemaster_playback::OutputMaster::default();
    master.set_blackout(true);
    preview.transport.seek(1250).unwrap();
    let dark = preview.render(&doc, 1, master).unwrap().output.unwrap();
    assert_eq!(dark.slots[4], 0);
    preview.transport.seek(1500).unwrap();
    master.set_blackout(false);
    master.set_percent(50).unwrap();
    let restored = preview.render(&doc, 1, master).unwrap().output.unwrap();
    assert_eq!(restored.slots[8], 128);
    assert_eq!(restored.slots[4], 0);
    assert_eq!(preview.position().position_ms, 1500);
}

#[test]
fn disabled_clip_invalidates_cached_light_without_moving_native_cursor() {
    let (mut doc, _) = document();
    doc.edit(
        serde_json::from_value(json!({"op":"audio","command":{"kind":"convertLightingClips"}}))
            .unwrap(),
    )
    .unwrap();
    let track = doc.audio_timeline().unwrap();
    let id = track.lighting_clips.as_ref().unwrap()[0].id.clone();
    let mut preview = AudioPreview::default();
    preview
        .load("unused-for-paused-render.wav".into(), track)
        .unwrap();
    preview.transport.seek(1250).unwrap();
    let master = stagemaster_playback::OutputMaster::default();
    let before = preview
        .render(&doc, 1, master)
        .unwrap()
        .output
        .unwrap()
        .slots;
    assert!(before.iter().any(|v| *v > 0));
    for (enabled, version) in [(false, 2), (true, 3)] {
        doc.edit(serde_json::from_value(json!({"op":"audio","command":{"kind":"editLightingClips","ids":[id],"action":{"kind":"enabled","enabled":enabled}}})).unwrap()).unwrap();
        preview.synchronize(&doc);
        assert_eq!(preview.position().position_ms, 1250);
        let after = preview
            .render(&doc, version, master)
            .unwrap()
            .output
            .unwrap()
            .slots;
        if enabled {
            assert_eq!(after, before);
        } else {
            assert!(after.iter().all(|v| *v == 0));
        }
        assert!(!preview.position().playing);
    }
}
