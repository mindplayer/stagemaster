use super::*;
use crate::output_control;
fn set_master(shared: &SharedSession, percent: u8, blackout: bool) {
    let mut session = shared.lock().unwrap();
    let current = session
        .output_request(output_control::Request::Snapshot)
        .unwrap();
    session
        .output_request(output_control::Request::Set {
            epoch: current.epoch,
            serial: current.serial + 1,
            percent,
            blackout,
        })
        .unwrap();
}
#[tokio::test]
async fn all_previs_sources_scale_once_and_keep_spatial_color_state() {
    let shared = shared();
    let scene_id;
    {
        let mut s = shared.lock().unwrap();
        let v = s.previs_document().unwrap().view();
        scene_id = v.scenes[0].id.clone();
        let command=serde_json::from_value(json!({"op":"setSceneValue","sceneId":scene_id,"fixtureId":v.fixtures[0].id,"attribute":"dimmer","mode":"literal","value":65535})).unwrap();
        let generation = s.previs_revision().generation;
        s.edit(generation, command).unwrap();
        let generation = s.previs_revision().generation;
        s.set_previs_source(
            generation,
            Source::Scene {
                scene_id: scene_id.clone(),
            },
        )
        .unwrap();
    }
    let bridge = start(shared.clone()).await;
    let client = client();
    json_get(&client, &bridge, "/v1/scene").await;
    let full = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(full["lights"][0]["intensity"], 1.0);
    set_master(&shared, 50, false);
    let half = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(half["lights"][0]["intensity"], 0.5);
    assert_eq!(half["lights"][0]["color"], full["lights"][0]["color"]);
    {
        let mut s = shared.lock().unwrap();
        let generation = s.previs_revision().generation;
        let loaded = s
            .preview(preview::Request::LoadScene {
                generation,
                scene_id: scene_id.clone(),
            })
            .unwrap();
        let value = serde_json::to_value(loaded).unwrap();
        let epoch = u32::try_from(value["epoch"].as_u64().unwrap()).unwrap();
        s.preview(preview::Request::Control {
            epoch,
            serial: 1,
            command: preview::Command::Execute { step_id: scene_id },
        })
        .unwrap();
        s.preview(preview::Request::Control {
            epoch,
            serial: 2,
            command: preview::Command::Pause,
        })
        .unwrap();
        s.set_previs_source(generation, Source::Playback).unwrap();
    }
    let dynamic = json_get(&client, &bridge, "/v1/frame").await;
    assert!((dynamic["lights"][0]["intensity"].as_f64().unwrap() - 0.5).abs() < 0.00002);
    set_master(&shared, 50, true);
    let dark = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(dark["lights"][0]["intensity"], 0.0);
    assert_eq!(dark["lights"][0]["color"], dynamic["lights"][0]["color"]);
    set_master(&shared, 100, false);
    let restored = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(restored["lights"][0]["intensity"], 1.0);
}
