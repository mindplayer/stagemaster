#[path = "support/audio_split.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_project::{ClipFadeMode, Document};
use support::{audio, clips, split};

fn fixture() -> Document {
    let d = support::fixture();
    let mut root: Value = serde_json::from_slice(&d.encode().unwrap()).unwrap();
    let mut effect = root["lighting"]["scenes"][0]["effects"][0].clone();
    effect["phaseDegrees"] = json!(0);
    effect["periodMs"] = json!(1000);
    effect["channels"][0]["low"] = json!(0);
    effect["channels"][0]["high"] = json!(65535);
    root["lighting"]["scenes"][0]["effects"][0] = effect.clone();
    effect["id"] = json!("b0000000-0000-4000-8000-000000000002");
    effect["periodMs"] = json!(2000);
    root["lighting"]["scenes"][1]["effects"] = json!([effect]);
    let mut d = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let mut next = clips(&d)[1].clone();
    next.fade_mode = ClipFadeMode::Dynamic;
    next.fade_ms = 1000;
    audio(&mut d, json!({"kind":"putLightingClip", "clip":next})).unwrap();
    d
}
fn sample(d: &Document, time: u64) -> (Vec<u16>, Vec<u8>) {
    let track = d.audio_timeline().unwrap();
    let active = track.lighting_at(time);
    let compiled = d.compile_audio_segment(active.map(|c| c.id)).unwrap();
    let mut player = compiled.playback.into_player().unwrap();
    player
        .advance(time - active.map_or(0, |c| c.start_ms))
        .unwrap();
    (
        player.values().to_vec(),
        compiled.output.render(player.values()).unwrap().slots,
    )
}
fn compare(before: &Document, after: &Document, times: impl Iterator<Item = u64>) {
    for time in times {
        assert_eq!(sample(before, time), sample(after, time), "at {time}");
    }
}
#[test]
fn source_keeps_moving_and_old_single_plan_cannot_silently_freeze_it() {
    let d = fixture();
    for (time, value) in [
        (4000, 0),
        (4250, 28672),
        (4500, 49152),
        (5000, 65535),
        (5500, 32768),
    ] {
        assert_eq!(sample(&d, time).0, vec![value]);
    }
    let id = clips(&d)[1].id.clone();
    assert!(
        d.compile_audio_lighting(Some(&id))
            .err()
            .unwrap()
            .contains("宿主复合")
    );
    // The old static lane still compiles, with no change to its fade behavior.
    assert_eq!(sample(&d, 250).0, support::sample(&d, 250));
}
#[test]
fn split_inside_crossfade_repeatedly_preserves_both_sources_and_roundtrip() {
    let before = fixture();
    let mut edited = before.clone();
    for time in [4001, 4107, 4499, 4999, 5500] {
        let track = edited.audio_timeline().unwrap();
        let id = track.lighting_at(time).unwrap().id.to_owned();
        split(&mut edited, &id, time).unwrap();
    }
    compare(
        &before,
        &edited,
        (0..6500)
            .step_by(17)
            .chain([4000, 4001, 4106, 4107, 4499, 4999, 5000, 5500]),
    );
    let reopened = Document::decode(&edited.encode().unwrap()).unwrap();
    assert_eq!(edited, reopened);
    compare(&edited, &reopened, (3999..6001).rev().step_by(31));
}
#[test]
fn internal_slice_restore_move_copy_and_source_edits_keep_semantic_history() {
    let before = fixture();
    let mut edited = before.clone();
    let mut slice = clips(&edited)[1].clone();
    slice.start_ms = 4200;
    slice.end_ms = 4555;
    audio(
        &mut edited,
        json!({"kind":"sliceLightingClip","clip":slice}),
    )
    .unwrap();
    compare(&before, &edited, (4200..4555).step_by(7));
    let mut restore = clips(&edited)[1].clone();
    restore.start_ms = 4000;
    restore.end_ms = 6000;
    audio(
        &mut edited,
        json!({"kind":"trimLightingClip","clip":restore}),
    )
    .unwrap();
    compare(&before, &edited, (4000..6000).step_by(37));
    let id = clips(&edited)[1].id.clone();
    audio(
        &mut edited,
        json!({"kind":"copyLightingClip","id":id,"startMs":7000}),
    )
    .unwrap();
    for t in [0, 100, 500, 1000, 1999] {
        assert_eq!(sample(&before, 4000 + t), sample(&edited, 7000 + t));
    }
    let copy = clips(&edited)[2].id.clone();
    audio(&mut edited,json!({"kind":"editLightingClips","ids":[copy],"action":{"kind":"move","destinationMs":6000}})).unwrap();
    assert_eq!(sample(&before, 4250), sample(&edited, 6250));
    let mut root: Value = serde_json::from_slice(&edited.encode().unwrap()).unwrap();
    root["lighting"]["scenes"][0]["effects"][0]["channels"][0]["high"] = json!(1000);
    edited = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    assert_ne!(sample(&before, 4250), sample(&edited, 6250));
}
#[test]
fn ordinary_crossfade_follows_adjacency_but_preserved_crossfade_keeps_its_source() {
    let mut d = fixture();
    let id = clips(&d)[1].id.clone();
    audio(&mut d,json!({"kind":"editLightingClips","ids":[id],"action":{"kind":"move","destinationMs":5000}})).unwrap();
    assert_eq!(sample(&d, 5250).0, vec![4096]); // default 0 -> target 16384, 25%.
    let before = d.clone();
    split(&mut d, &id, 5250).unwrap();
    let right = clips(&d)[2].id.clone();
    audio(&mut d,json!({"kind":"editLightingClips","ids":[right],"action":{"kind":"move","destinationMs":7000}})).unwrap();
    assert_eq!(sample(&before, 5250), sample(&d, 7000));
    audio(
        &mut d,
        json!({"kind":"resetLightingClipEntryFade","id":right}),
    )
    .unwrap();
    assert_eq!(sample(&d, 7000).0, vec![0]);
}
#[test]
fn third_dynamic_source_is_rejected_atomically_but_snapshot_can_take_actual_boundary() {
    let mut d = fixture();
    let mut short = clips(&d)[1].clone();
    short.end_ms = 4300;
    audio(&mut d, json!({"kind":"sliceLightingClip","clip":short})).unwrap();
    let before = d.clone();
    let scene = clips(&d)[0].scene_id.clone();
    audio(&mut d,json!({"kind":"addLightingClip","name":"第三段","sceneId":scene,"startMs":4300,"endMs":5500,"fadeMs":500})).unwrap();
    let compiled = before
        .compile_audio_segment(Some(&clips(&before)[1].id))
        .unwrap();
    let mut source = compiled.playback.into_player().unwrap();
    source.advance(300).unwrap();
    assert_eq!(sample(&d, 4300).0, source.values());
    let baseline = d.clone();
    let mut third = clips(&d)[2].clone();
    third.fade_mode = ClipFadeMode::Dynamic;
    assert!(
        audio(&mut d, json!({"kind":"putLightingClip","clip":third}))
            .unwrap_err()
            .contains("尚未结束")
    );
    assert_eq!(d, baseline);
}

#[test]
fn preserved_static_source_keeps_its_own_entry_fade_while_crossing() {
    let before = fixture();
    let mut d = before.clone();
    let mut first = clips(&d)[0].clone();
    first.end_ms = 100;
    audio(&mut d, json!({"kind":"sliceLightingClip","clip":first})).unwrap();
    let mut next = clips(&d)[1].clone();
    next.start_ms = 100;
    next.end_ms = 2100;
    audio(&mut d, json!({"kind":"putLightingClip","clip":next})).unwrap();
    assert_eq!(sample(&d, 100).0, sample(&before, 100).0);
    // Q16 phase floor(350*65536/1000)=22937; triangle=45874, value=45873.
    // Its 70% entry gives 32111; crossing 25% into target 16384 gives 28179.
    assert_eq!(sample(&before, 350).0, vec![32111]);
    assert_eq!(sample(&d, 350).0, vec![28179]);
    let original = d.clone();
    let id = clips(&d)[1].id.clone();
    split(&mut d, &id, 333).unwrap();
    assert!(
        clips(&d)[2]
            .entry_crossfade
            .as_ref()
            .unwrap()
            .source
            .entry_fade
            .is_some()
    );
    compare(&original, &d, (100..2100).step_by(23));
}
#[test]
fn source_only_scene_references_block_removal_even_when_clip_disabled() {
    let mut d = fixture();
    let first = clips(&d)[0].clone();
    let id = clips(&d)[1].id.clone();
    split(&mut d, &id, 4250).unwrap();
    audio(&mut d, json!({"kind":"removeLightingClip","id":first.id})).unwrap();
    let ids: Vec<_> = clips(&d).iter().map(|c| c.id.clone()).collect();
    audio(
        &mut d,
        json!({"kind":"editLightingClips","ids":ids,"action":{"kind":"enabled","enabled":false}}),
    )
    .unwrap();
    let baseline = d.clone();
    let error = d
        .edit(serde_json::from_value(json!({"op":"removeScene","id":first.scene_id})).unwrap())
        .unwrap_err();
    assert!(error.contains("来源场景"));
    assert_eq!(d, baseline);
}
#[test]
fn mode_group_edit_lock_and_history_forgery_have_atomic_rejection() {
    let mut d = fixture();
    let id = clips(&d)[1].id.clone();
    split(&mut d, &id, 4222).unwrap();
    let right = clips(&d)[2].clone();
    let baseline = d.clone();
    let mut forged = right.clone();
    forged.entry_crossfade.as_mut().unwrap().source.elapsed_ms += 1;
    for kind in ["putLightingClip", "trimLightingClip", "sliceLightingClip"] {
        assert!(audio(&mut d, json!({"kind":kind,"clip":forged})).is_err());
        assert_eq!(d, baseline);
    }
    let mut early = right.clone();
    early.start_ms = 3000;
    assert!(audio(&mut d, json!({"kind":"trimLightingClip","clip":early})).is_err());
    assert_eq!(d, baseline);
    audio(
        &mut d,
        json!({"kind":"setLightingClipLock","id":right.id,"locked":true}),
    )
    .unwrap();
    let baseline = d.clone();
    assert!(audio(&mut d,json!({"kind":"editLightingClips","ids":[id,right.id],"action":{"kind":"fade","fadeMs":100,"fadeMode":"snapshot"}})).is_err());
    assert_eq!(d, baseline);
    audio(
        &mut d,
        json!({"kind":"setLightingClipLock","id":right.id,"locked":false}),
    )
    .unwrap();
    audio(&mut d,json!({"kind":"editLightingClips","ids":[id,right.id],"action":{"kind":"fade","fadeMs":100,"fadeMode":"snapshot"}})).unwrap();
    for clip in &clips(&d)[1..] {
        assert!(clip.entry_crossfade.is_none());
        assert_eq!(clip.fade_mode, ClipFadeMode::Snapshot);
    }
}
#[test]
fn malformed_origins_missing_capability_and_invalid_ranges_fail_loading() {
    let mut d = fixture();
    let id = clips(&d)[1].id.clone();
    split(&mut d, &id, 4222).unwrap();
    let root: Value = serde_json::from_slice(&d.encode().unwrap()).unwrap();
    let path = "/media/audioEditing/lightingClips/2";
    for (suffix, value) in [
        (
            "/entryCrossfade/source/sceneId",
            json!("c0000000-0000-4000-8000-000000000001"),
        ),
        ("/entryCrossfade/source/elapsedMs", json!(3_600_000)),
        ("/entryCrossfade/offsetMs", json!(-1)),
        ("/entryCrossfade/durationMs", json!(0)),
        ("/fadeMs", json!(3)),
        ("/fadeMode", json!("snapshot")),
    ] {
        let mut invalid = root.clone();
        *invalid.pointer_mut(&format!("{path}{suffix}")).unwrap() = value;
        assert!(
            Document::decode(&serde_json::to_vec(&invalid).unwrap()).is_err(),
            "{suffix}"
        );
    }
    let mut invalid = root.clone();
    invalid["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "media.audio-clip-crossfade");
    assert!(Document::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
    let mut invalid = root;
    invalid["media"]["audioEditing"]["lightingClips"][2]["entryCrossfade"]["source"]["entryCrossfade"] =
        json!({});
    assert!(Document::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
}
