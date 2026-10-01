#[path = "support/audio_split.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_project::Document;
use support::{audio, clips, fixture, sample, split};
#[test]
fn split_preserves_dynamic_samples_and_following_entry_snapshot_at_every_position() {
    for cut in [500, 1371, 3999] {
        let mut d = fixture();
        let before = d.clone();
        let id = clips(&d)[0].id.clone();
        split(&mut d, &id, cut).unwrap();
        let c = clips(&d);
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].id, id);
        assert_ne!(c[1].id, id);
        assert_eq!(
            (
                c[0].end_ms,
                c[1].start_ms,
                c[1].effect_offset_ms,
                c[1].fade_ms
            ),
            (cut, cut, cut, 0)
        );
        assert_eq!(c[2], clips(&before)[1]);
        for t in (0..10000)
            .step_by(37)
            .chain([cut - 1, cut, cut + 1, 3999, 4000, 4250, 5999, 0])
        {
            assert_eq!(sample(&d, t), sample(&before, t), "cut {cut}, at {t}");
        }
        assert_eq!(d, Document::decode(&d.encode().unwrap()).unwrap());
    }
}
#[test]
fn repeated_split_copy_move_disable_and_reset_preserve_explicit_source_time() {
    let mut d = fixture();
    let before = d.clone();
    let first = clips(&d)[0].id.clone();
    split(&mut d, &first, 1371).unwrap();
    let second = clips(&d)[1].id.clone();
    split(&mut d, &second, 2333).unwrap();
    let third = clips(&d)[2].clone();
    assert_eq!(third.effect_offset_ms, 2333);
    for t in [1371, 2332, 2333, 2768, 3999, 4125] {
        assert_eq!(sample(&d, t), sample(&before, t));
    }
    audio(
        &mut d,
        json!({"kind":"copyLightingClip","id":third.id,"startMs":7000}),
    )
    .unwrap();
    let copy = clips(&d).last().unwrap().clone();
    assert_eq!(copy.effect_offset_ms, 2333);
    assert_eq!(sample(&d, 7410), sample(&before, 2743));
    audio(&mut d,json!({"kind":"editLightingClips","ids":[copy.id],"action":{"kind":"move","destinationMs":6000}})).unwrap();
    assert_eq!(sample(&d, 6410), sample(&before, 2743));
    audio(&mut d,json!({"kind":"editLightingClips","ids":[copy.id],"action":{"kind":"enabled","enabled":false}})).unwrap();
    assert_eq!(sample(&d, 6410), vec![0]);
    audio(&mut d,json!({"kind":"editLightingClips","ids":[copy.id],"action":{"kind":"enabled","enabled":true}})).unwrap();
    assert_eq!(sample(&d, 6410), sample(&before, 2743));
    audio(
        &mut d,
        json!({"kind":"resetLightingClipEffectOffset","id":copy.id}),
    )
    .unwrap();
    assert_eq!(clips(&d).last().unwrap().effect_offset_ms, 0);
    assert_ne!(sample(&d, 6410), sample(&before, 2743));
}
#[test]
fn invalid_boundaries_capacity_and_locks_are_atomic() {
    let mut d = fixture();
    let first = clips(&d)[0].id.clone();
    let before = d.clone();
    for time in [0, 4000, 10000, u64::MAX] {
        assert!(split(&mut d, &first, time).is_err());
        assert_eq!(d, before);
    }
    assert!(split(&mut d, "ffffffff-0000-4000-8000-000000000000", 500).is_err());
    assert_eq!(d, before);
    audio(
        &mut d,
        json!({"kind":"setLightingClipLock","id":first,"locked":true}),
    )
    .unwrap();
    let locked = d.clone();
    assert!(split(&mut d, &first, 1000).is_err());
    assert!(
        audio(
            &mut d,
            json!({"kind":"resetLightingClipEffectOffset","id":first})
        )
        .is_err()
    );
    assert_eq!(d, locked);
    let mut root: Value = serde_json::from_slice(&before.encode().unwrap()).unwrap();
    let scene = clips(&before)[0].scene_id.clone();
    root["media"]["audioEditing"]["lightingClips"]=Value::Array((0..512).map(|i|json!({"id":format!("c0000000-0000-4000-8000-{i:012}"),"name":"短段","sceneId":scene,"startMs":i*10,"endMs":i*10+5,"fadeMs":0,"locked":false})).collect());
    let mut full = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let baseline = full.clone();
    let id = clips(&full)[0].id.clone();
    assert!(split(&mut full, &id, 2).unwrap_err().contains("512"));
    assert_eq!(full, baseline);
}

#[test]
fn effect_offset_format_is_compatible_strict_and_cannot_be_erased_by_ordinary_edits() {
    let mut d = fixture();
    assert!(
        !String::from_utf8(d.encode().unwrap())
            .unwrap()
            .contains("effectOffsetMs")
    );
    let id = clips(&d)[0].id.clone();
    split(&mut d, &id, 1333).unwrap();
    let c = clips(&d)[1].clone();
    let mut bad = serde_json::to_value(&c).unwrap();
    bad["effectOffsetMs"] = json!(0);
    let before = d.clone();
    assert!(audio(&mut d, json!({"kind":"putLightingClip","clip":bad})).is_err());
    assert_eq!(d, before);
    let root: Value = serde_json::from_slice(&d.encode().unwrap()).unwrap();
    for value in [
        json!(-1),
        json!(1.5),
        Value::Null,
        json!("1333"),
        json!(u64::MAX),
        json!(3_600_000),
    ] {
        let mut r = root.clone();
        r["media"]["audioEditing"]["lightingClips"][1]["effectOffsetMs"] = value;
        assert!(Document::decode(&serde_json::to_vec(&r).unwrap()).is_err());
    }
    let mut r = root.clone();
    r["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|v| v["key"] != "media.audio-clip-offset");
    assert!(Document::decode(&serde_json::to_vec(&r).unwrap()).is_err());
    let mut bare: Value =
        serde_json::from_slice(&Document::new("空").unwrap().encode().unwrap()).unwrap();
    bare["requires"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"media.audio-clip-offset","version":1}));
    assert!(Document::decode(&serde_json::to_vec(&bare).unwrap()).is_err());
    for cmd in [
        json!({"kind":"splitLightingClip","id":id,"timeMs":2000,"unknown":1}),
        json!({"kind":"resetLightingClipEffectOffset","id":id,"unknown":1}),
    ] {
        assert!(serde_json::from_value::<stagemaster_project::AudioEdit>(cmd).is_err());
    }
    audio(&mut d, json!({"kind":"clear"})).unwrap();
    assert!(
        !String::from_utf8(d.encode().unwrap())
            .unwrap()
            .contains("media.audio-clip-offset")
    );
}
