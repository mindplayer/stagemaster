#[path = "support/media.rs"]
#[allow(dead_code)]
mod media_support;
mod support;
use media_support::*;
use serde_json::json;
use stagemaster_live::{PlaybackSelection, Session, SourceSpec};
use stagemaster_project::{Document, PackageSelection};
use support::*;

fn document_with_clips(count: u64) -> Document {
    let mut doc = document();
    for command in [
        json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":1000}}),
        json!({"kind":"convertLightingClips"}),
    ] {
        doc.edit(serde_json::from_value(json!({"op":"audio","command":command})).unwrap())
            .unwrap();
    }
    let mut root: serde_json::Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let original = root["lighting"]["fixtures"][0].clone();
    let patch = root["lighting"]["patches"][0].clone();
    let mut ids = vec![original["id"].clone()];
    for n in 1..64 {
        let mut fixture = original.clone();
        fixture["id"] = json!(id(1000 + n));
        let mut patched = patch.clone();
        patched["fixtureId"] = fixture["id"].clone();
        patched["address"] = json!(1 + n * 4);
        ids.push(fixture["id"].clone());
        root["lighting"]["fixtures"]
            .as_array_mut()
            .unwrap()
            .push(fixture);
        root["lighting"]["patches"]
            .as_array_mut()
            .unwrap()
            .push(patched);
    }
    let mut effect = root["lighting"]["scenes"][3]["effects"][0].clone();
    effect["id"] = json!(id(91));
    effect["fixtureIds"] = json!(ids);
    effect["channels"][0]["attribute"] = json!("red");
    root["lighting"]["scenes"][0]["effects"] = json!([effect]);
    root["media"]["audioEditing"]["lightingClips"] = json!(
        (0..count)
            .map(|n| json!({
                "id":id(2000+n),"name":"连续片段","sceneId":id(1),
                "startMs":n*2,"endMs":n*2+2,"fadeMs":0,"locked":false
            }))
            .collect::<Vec<_>>()
    );
    decode(&root)
}

#[test]
fn authored_plans_and_mixed_sources_share_the_actual_resident_effect_budget() {
    let maximum = document_with_clips(256);
    assert_eq!(
        maximum
            .compile_live_audio()
            .unwrap()
            .budget()
            .effect_channels,
        16_384
    );
    let error = document_with_clips(257).compile_live_audio().err().unwrap();
    assert!(error.contains("累计"), "{error}");

    let doc = document_with_clips(128);
    let mut sources = vec![
        SourceSpec {
            id: [1; 16],
            priority: 0,
            playback: Some(PlaybackSelection::AudioTimeline),
        },
        SourceSpec {
            id: [2; 16],
            priority: 0,
            playback: Some(PlaybackSelection::AudioTimeline),
        },
    ];
    let mut media = group();
    media.sources = vec![[1; 16], [2; 16]];
    assert!(Session::prepare_with_media(&doc, [9; 16], &sources, &[media.clone()], 0).is_ok());
    sources.push(playback(3, PackageSelection::Scene { id: id(1) }));
    let error = Session::prepare_with_media(&doc, [9; 16], &sources, &[media], 0)
        .err()
        .unwrap();
    assert!(error.contains("累计"), "{error}");
}
