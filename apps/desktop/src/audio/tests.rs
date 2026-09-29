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
        markers: vec![
            AudioMarker {
                id: "a0000000-0000-4000-8000-000000000001".into(),
                name: "追逐开始".into(),
                time_ms: 1000,
                scene_id: Some(scene.clone()),
            },
            AudioMarker {
                id: "a0000000-0000-4000-8000-000000000002".into(),
                name: "暗场开始".into(),
                time_ms: 4000,
                scene_id: Some(view.scenes[1].id.clone()),
            },
        ],
    };
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
        let out = preview.render(&doc, 1).unwrap().output.unwrap();
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
    doc.edit(stagemaster_project::EditCommand::Audio {
        command: stagemaster_project::AudioEdit::SetAsset {
            asset: track.asset.clone(),
        },
    })
    .unwrap();
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
