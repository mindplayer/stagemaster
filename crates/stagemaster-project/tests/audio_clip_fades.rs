#[path = "support/fixture_function.rs"]
mod functions;
#[path = "support/audio_split.rs"]
mod support;
use serde_json::json;
use stagemaster_playback::Player;
use stagemaster_project::Document;
use support::{audio, clips, fixture, sample, split};

fn slots(document: &Document, time: u64) -> Vec<u8> {
    let track = document.audio_timeline().unwrap();
    let active = track.lighting_at(time);
    let compiled = document
        .compile_audio_lighting(active.map(|clip| clip.id))
        .unwrap();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    player
        .advance(time - active.map_or(0, |clip| clip.start_ms))
        .unwrap();
    compiled.output.render(player.values()).unwrap().slots
}
fn equal_at(before: &Document, after: &Document, times: impl Iterator<Item = u64>) {
    for time in times {
        assert_eq!(
            sample(before, time),
            sample(after, time),
            "logical at {time}"
        );
        assert_eq!(slots(before, time), slots(after, time), "DMX at {time}");
    }
}
#[test]
fn split_inside_first_or_following_entry_preserves_whole_timeline_and_roundtrip() {
    for cut in [1, 233, 499, 4001, 4321, 4749] {
        let before = fixture();
        let mut edited = before.clone();
        let id = clips(&before)[usize::from(cut > 4000)].id.clone();
        split(&mut edited, &id, cut).unwrap();
        let index = usize::from(cut > 4000);
        let left = &clips(&edited)[index];
        let right = &clips(&edited)[index + 1];
        assert_eq!(left.id, id);
        assert_eq!(
            left.entry_fade.as_ref().unwrap().from,
            right.entry_fade.as_ref().unwrap().from
        );
        assert_eq!(
            right.entry_fade.as_ref().unwrap().offset_ms,
            cut - left.start_ms
        );
        equal_at(
            &before,
            &edited,
            (0..6500)
                .step_by(31)
                .chain([cut - 1, cut, cut + 1, 499, 500, 4000, 4749, 4750]),
        );
        let reopened = Document::decode(&edited.encode().unwrap()).unwrap();
        assert_eq!(reopened, edited);
    }
}
#[test]
fn repeated_slices_copy_move_and_frozen_predecessor_are_independent() {
    let before = fixture();
    let mut edited = before.clone();
    let second = clips(&edited)[1].id.clone();
    split(&mut edited, &second, 4100).unwrap();
    let next = clips(&edited)[2].id.clone();
    split(&mut edited, &next, 4200).unwrap();
    let last = clips(&edited)[3].clone();
    assert_eq!(last.entry_fade.as_ref().unwrap().offset_ms, 200);
    equal_at(&before, &edited, (3990..6010).step_by(13));
    audio(
        &mut edited,
        json!({"kind":"copyLightingClip","id":last.id,"startMs":7000}),
    )
    .unwrap();
    let copy = clips(&edited).last().unwrap().clone();
    for elapsed in [0, 1, 100, 549, 550, 999] {
        assert_eq!(
            sample(&edited, 7000 + elapsed),
            sample(&before, 4200 + elapsed)
        );
    }
    // Original previous dynamic source changes, but the captured starting state remains.
    let mut root: serde_json::Value = serde_json::from_slice(&edited.encode().unwrap()).unwrap();
    root["lighting"]["scenes"][0]["effects"][0]["channels"][0]["high"] = json!(1001);
    edited = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    assert_eq!(sample(&edited, 7150), sample(&before, 4350));
    audio(&mut edited, json!({"kind":"editLightingClips","ids":[copy.id],"action":{"kind":"move","destinationMs":6200}})).unwrap();
    assert_eq!(sample(&edited, 6350), sample(&before, 4350));
    audio(&mut edited, json!({"kind":"editLightingClips","ids":[copy.id],"action":{"kind":"enabled","enabled":false}})).unwrap();
    assert_eq!(sample(&edited, 6350), vec![0]);
    audio(&mut edited, json!({"kind":"editLightingClips","ids":[copy.id],"action":{"kind":"enabled","enabled":true}})).unwrap();
    assert_eq!(sample(&edited, 6350), sample(&before, 4350));
}
#[test]
fn complete_trim_and_restore_retain_fade_history_and_adjacent_short_slice_boundary() {
    let before = fixture();
    let mut edited = before.clone();
    let mut clip = clips(&edited)[0].clone();
    clip.start_ms = 113;
    clip.end_ms = 311;
    audio(&mut edited, json!({"kind":"sliceLightingClip","clip":clip})).unwrap();
    let sliced = clips(&edited)[0].clone();
    assert_eq!((sliced.fade_ms, sliced.effect_offset_ms), (198, 113));
    equal_at(&before, &edited, 113..311);
    let mut restored = sliced;
    restored.start_ms = 0;
    restored.end_ms = 4000;
    audio(
        &mut edited,
        json!({"kind":"trimLightingClip","clip":restored}),
    )
    .unwrap();
    equal_at(&before, &edited, (0..6000).step_by(17));
    // A newly adjacent ordinary entry takes the actual short slice's end value.
    let mut short = clips(&edited)[0].clone();
    short.end_ms = 113;
    audio(&mut edited, json!({"kind":"trimLightingClip","clip":short})).unwrap();
    let mut next = clips(&edited)[1].clone();
    next.start_ms = 113;
    audio(&mut edited, json!({"kind":"putLightingClip","clip":next})).unwrap();
    let at_boundary = u64::from(sample(&before, 113)[0]);
    assert_eq!(sample(&edited, 113)[0], u16::try_from(at_boundary).unwrap());
    assert_eq!(
        u64::from(sample(&edited, 363)[0]),
        (at_boundary * 500 + 375) / 750
    );
    // Capturing this new transition remains independent of a predecessor chain.
    let baseline = edited.clone();
    let next_id = clips(&edited)[1].id.clone();
    split(&mut edited, &next_id, 313).unwrap();
    equal_at(&baseline, &edited, (113..900).step_by(7));
}
#[test]
fn ordinary_edits_cannot_forge_snapshots_and_explicit_fade_reset_is_atomic() {
    let mut edited = fixture();
    let id = clips(&edited)[0].id.clone();
    split(&mut edited, &id, 201).unwrap();
    let current = clips(&edited)[1].clone();
    let baseline = edited.clone();
    let mut forged = current.clone();
    forged.entry_fade.as_mut().unwrap().from[0].value = 555;
    for kind in ["putLightingClip", "trimLightingClip", "sliceLightingClip"] {
        assert!(audio(&mut edited, json!({"kind":kind,"clip":forged})).is_err());
        assert_eq!(edited, baseline);
    }
    let mut renamed = current.clone();
    renamed.name = "新名称".into();
    audio(
        &mut edited,
        json!({"kind":"putLightingClip","clip":renamed}),
    )
    .unwrap();
    assert_eq!(clips(&edited)[1].entry_fade, current.entry_fade);
    audio(
        &mut edited,
        json!({"kind":"resetLightingClipEntryFade","id":current.id}),
    )
    .unwrap();
    assert!(clips(&edited)[1].entry_fade.is_none());
    assert_eq!(sample(&edited, 201), sample(&baseline, 201));
    assert_ne!(sample(&edited, 299), sample(&baseline, 299));
    let reset = edited.clone();
    audio(
        &mut edited,
        json!({"kind":"resetLightingClipEntryFade","id":current.id}),
    )
    .unwrap();
    assert_eq!(edited, reset);
    audio(
        &mut edited,
        json!({"kind":"editLightingClips","ids":[id],"action":{"kind":"fade","fadeMs":201}}),
    )
    .unwrap();
    assert!(clips(&edited)[0].entry_fade.is_none());
}

#[test]
fn discrete_wheels_and_shutter_snap_without_entering_the_historical_snapshot() {
    for fine in [false, true] {
        let (mut document, fixture_id, scenes) = functions::setup(fine);
        for (attribute, function) in [
            ("color-wheel", "blue"),
            ("shutter", "strobe"),
            ("gobo-wheel", "dots"),
        ] {
            functions::choose(
                &mut document,
                &scenes[1],
                &fixture_id,
                attribute,
                function,
                0,
            )
            .unwrap();
        }
        for command in [
            json!({"kind":"setAsset","asset":{"digest":"cd".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":2000}}),
            json!({"kind":"convertLightingClips"}),
            json!({"kind":"addLightingClip","name":"前段","sceneId":scenes[0],"startMs":0,"endMs":1000,"fadeMs":0}),
            json!({"kind":"addLightingClip","name":"后段","sceneId":scenes[1],"startMs":1000,"endMs":2000,"fadeMs":700}),
        ] {
            audio(&mut document, command).unwrap();
        }
        let before = document.clone();
        let id = clips(&document)[1].id.clone();
        split(&mut document, &id, 1233).unwrap();
        assert!(
            clips(&document)[1]
                .entry_fade
                .as_ref()
                .unwrap()
                .from
                .iter()
                .all(|v| v.attribute == "dimmer")
        );
        equal_at(
            &before,
            &document,
            (999..2000)
                .step_by(11)
                .chain([1000, 1232, 1233, 1234, 1699, 1700]),
        );
        let mut forged = functions::raw(&document);
        forged["media"]["audioEditing"]["lightingClips"][1]["entryFade"]["from"][0]["attribute"] =
            json!("shutter");
        assert!(functions::decode(&forged).unwrap_err().contains("连续属性"));
    }
}
