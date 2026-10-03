#[path = "support/audio.rs"]
mod audio_support;
#[path = "support/media_host.rs"]
mod media_support;
mod support;
use audio_support::{audio, consume};
use media_support::Rig;
use serde_json::json;
use stagemaster_live::{Command, PlaybackSelection, media::Status};
use stagemaster_live_host::Action;
use stagemaster_project::{Document, EditCommand};
use std::time::Duration;
use support::{fixtures::*, *};

fn rig() -> Rig {
    let mut raw = fixture(0);
    raw["lighting"]["scenes"][3]["assignments"] = json!([set("blue", 50_000)]);
    let mut doc = decode(&raw);
    for command in [
        json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":1000}}),
        json!({"kind":"convertLightingClips"}),
        json!({"kind":"addLightingClip","name":"第一段","sceneId":id(1),"startMs":0,"endMs":50,"fadeMs":0}),
        json!({"kind":"addLightingClip","name":"交叉","sceneId":id(3),"startMs":50,"endMs":150,"fadeMs":50,"fadeMode":"dynamic"}),
    ] {
        doc.edit(EditCommand::Audio {
            command: serde_json::from_value(command).unwrap(),
        })
        .unwrap();
    }
    let mut sources = specs();
    sources[0].playback = Some(PlaybackSelection::AudioTimeline);
    Rig::with_document(doc, &sources)
}
fn expected(doc: &Document, position: u64) -> Vec<u16> {
    let track = doc.audio_timeline().unwrap();
    let active = track.lighting_at(position);
    let compiled = doc.compile_audio_segment(active.map(|s| s.id)).unwrap();
    let mut player = compiled.playback.into_player().unwrap();
    player
        .advance(position - active.map_or(0, |s| s.start_ms))
        .unwrap();
    player.values().to_vec()
}
#[test]
fn decoded_audio_drives_authored_crossfade_in_original_background_host_without_operator() {
    let mut rig = rig();
    let (_dir, mut source, audio) = audio();
    let client = connect(&rig.host, 1, false, 60_000);
    let state = send(
        &client,
        1,
        client.acquired_state().revision,
        control(rig.autonomous, Command::Execute(0)),
    );
    let prepared = rig
        .prepare
        .prepare(
            rig.port.initial().key,
            &rig.doc,
            0,
            true,
            state.observed_ms + 2000,
        )
        .unwrap();
    consume(&mut source, 10);
    let snapshot = audio.snapshot().unwrap();
    let render = snapshot.render.unwrap();
    let (sample, map) = rig.sample(render.sequence, 10, render.applied.playing, render.at);
    rig.reached(sample, &map);
    let ticket = rig
        .port
        .stage(prepared.advance_to(10, true).unwrap(), sample, map)
        .unwrap();
    let state = send(&client, 2, state.revision, Action::ActivateMedia { ticket });
    drop(rig.port.reclaim(ticket, false).unwrap());
    let key = state.media[0].unwrap().group.key;
    client.release(TTL).unwrap().wait(WAIT).unwrap().unwrap();
    drop(client);
    for position in [35, 60, 85, 110] {
        std::thread::sleep(Duration::from_millis(25));
        consume(&mut source, 25);
        let snapshot = audio.snapshot().unwrap();
        let render = snapshot.render.unwrap();
        assert_eq!(
            snapshot.position.tick * 1000 / u64::from(render.sample_rate),
            position
        );
        let (sample, map) =
            rig.sample(render.sequence, position, render.applied.playing, render.at);
        rig.reached(sample, &map);
        let serial = rig.port.publish(key, sample, map).unwrap();
        let observed = until(&rig.host.observer(), |s| {
            s.state.media[0]
                .unwrap()
                .observation
                .is_some_and(|r| r.serial == serial)
        });
        observed.state.media[0]
            .unwrap()
            .observation
            .unwrap()
            .result
            .unwrap();
        let expected = expected(&rig.doc, position);
        rig.assert_light(
            &observed.frame.unwrap().slots,
            [expected[0], expected[1], expected[2]],
        );
        assert_eq!(observed.frame.unwrap().slots[3], 195);
        assert!(observed.state.owner.is_none());
    }
    audio.request_playback(false).unwrap();
    consume(&mut source, 1);
    let snapshot = audio.snapshot().unwrap();
    let render = snapshot.render.unwrap();
    let (sample, map) = rig.sample(render.sequence, 110, render.applied.playing, render.at);
    rig.reached(sample, &map);
    rig.port.publish(key, sample, map).unwrap();
    let paused = until(&rig.host.observer(), |s| {
        s.state.media[0].unwrap().group.status == Status::Paused
    });
    let lost = until(&rig.host.observer(), |s| {
        s.state.media[0].unwrap().group.status == Status::Lost
    });
    assert_eq!(paused.frame.unwrap().slots, lost.frame.unwrap().slots);
    assert_eq!(
        lost.state.sources[1].unwrap().status,
        Some(stagemaster_runtime::Status::Running)
    );
    rig.host.shutdown(WAIT).unwrap();
}
