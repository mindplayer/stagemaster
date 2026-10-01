#[path = "support/audio_split.rs"]
mod support;
use serde_json::json;
use stagemaster_project::{AudioLightingClip, Document};
use support::{audio, clips, fixture, sample};
fn trim(d: &mut Document, c: &AudioLightingClip, start: u64, end: u64) -> Result<(), String> {
    let mut c = c.clone();
    c.start_ms = start;
    c.end_ms = end;
    audio(d, json!({"kind":"trimLightingClip","clip":c}))
}
#[test]
fn trimming_preserves_source_samples_but_reanchors_the_entry_fade() {
    for start in [1, 371, 2000] {
        let mut d = fixture();
        let before = d.clone();
        let c = clips(&d)[0].clone();
        trim(&mut d, &c, start, c.end_ms).unwrap();
        let after = clips(&d);
        assert_eq!(after[0].effect_offset_ms, start);
        assert_eq!(after[0].fade_ms, 500);
        assert_eq!(after[1], clips(&before)[1]);
        assert_eq!(sample(&d, start), vec![0]); // Entry belongs to the new edge.
        for time in (start + 500..4500).step_by(13) {
            assert_eq!(
                sample(&d, time),
                sample(&before, time),
                "start {start}, at {time}"
            );
        }
        assert_eq!(d, Document::decode(&d.encode().unwrap()).unwrap());
    }
}
#[test]
fn zero_fade_trim_is_reversible_and_repeated_trim_does_not_restart_effects() {
    let mut d = fixture();
    let mut c = clips(&d)[0].clone();
    c.fade_ms = 0;
    audio(&mut d, json!({"kind":"putLightingClip","clip":c})).unwrap();
    let before = d.clone();
    for start in [1333, 2333, 731, 0] {
        let c = clips(&d)[0].clone();
        trim(&mut d, &c, start, 4000).unwrap();
        assert_eq!(clips(&d)[0].effect_offset_ms, start);
        for time in (start..4500).step_by(23) {
            assert_eq!(sample(&d, time), sample(&before, time));
        }
    }
    // Capability persists, values and identity round-trip exactly.
    assert_eq!(clips(&d), clips(&before));
    let c = clips(&d)[0].clone();
    trim(&mut d, &c, 1333, 3500).unwrap();
    let trimmed = clips(&d)[0].clone();
    let mut moved = trimmed.clone();
    moved.start_ms = 7000;
    moved.end_ms = 7000 + trimmed.end_ms - trimmed.start_ms;
    audio(&mut d, json!({"kind":"putLightingClip","clip":moved})).unwrap();
    assert_eq!(sample(&d, 7437), sample(&before, 1770));
    let c = clips(&d).last().unwrap().clone();
    trim(&mut d, &c, 6500, 9667).unwrap();
    assert_eq!(clips(&d).last().unwrap().effect_offset_ms, 833);
    assert_eq!(sample(&d, 7437), sample(&before, 1770));
}
#[test]
fn invalid_trim_is_atomic_and_noop_preserves_document() {
    let mut d = fixture();
    let c = clips(&d)[0].clone();
    let before = d.clone();
    trim(&mut d, &c, c.start_ms, c.end_ms).unwrap();
    assert_eq!(d, before);
    for (start, end) in [
        (0, 0),
        (3900, 4000),
        (0, 4001),
        (u64::MAX, u64::MAX),
        (0, 10001),
    ] {
        assert!(trim(&mut d, &c, start, end).is_err());
        assert_eq!(d, before);
    }
    let mut forged = c.clone();
    forged.effect_offset_ms = 17;
    assert!(audio(&mut d, json!({"kind":"trimLightingClip","clip":forged})).is_err());
    assert_eq!(d, before);
    let mut moved = c.clone();
    moved.start_ms = 1000;
    moved.end_ms = 4000;
    audio(&mut d, json!({"kind":"putLightingClip","clip":moved})).unwrap();
    let moved = clips(&d)[0].clone();
    let baseline = d.clone();
    assert!(
        trim(&mut d, &moved, 999, 4000)
            .unwrap_err()
            .contains("零点")
    );
    assert_eq!(d, baseline);
    audio(
        &mut d,
        json!({"kind":"setLightingClipLock","id":c.id,"locked":true}),
    )
    .unwrap();
    let baseline = d.clone();
    assert!(trim(&mut d, &clips(&baseline)[0], 1100, 3900).is_err());
    assert_eq!(d, baseline);
    let value = json!({"kind":"trimLightingClip","clip":c,"offset":12});
    assert!(serde_json::from_value::<stagemaster_project::AudioEdit>(value).is_err());
}
#[test]
fn trim_respects_source_budget_and_preserves_disabled_state() {
    let d = fixture();
    let mut root: serde_json::Value = serde_json::from_slice(&d.encode().unwrap()).unwrap();
    root["requires"].as_array_mut().unwrap().extend([
        json!({"key":"media.audio-clip-offset","version":1}),
        json!({"key":"media.audio-clip-state","version":1}),
    ]);
    root["media"]["audioEditing"]["lightingClips"][0]["effectOffsetMs"] = json!(3_596_000);
    root["media"]["audioEditing"]["lightingClips"][0]["enabled"] = json!(false);
    let mut d = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let c = clips(&d)[0].clone();
    trim(&mut d, &c, 1000, 4000).unwrap();
    assert_eq!(clips(&d)[0].effect_offset_ms, 3_597_000);
    assert!(!clips(&d)[0].enabled);
    // Move away from the neighbor so the failure is the source bound.
    let mut c = clips(&d)[0].clone();
    c.start_ms = 7000;
    c.end_ms = 10000;
    audio(&mut d, json!({"kind":"putLightingClip","clip":c})).unwrap();
    let c = clips(&d).last().unwrap().clone();
    let baseline = d.clone();
    let mut c = c;
    c.start_ms = 6000;
    c.end_ms = 9000;
    audio(&mut d, json!({"kind":"putLightingClip","clip":c})).unwrap();
    let c = clips(&d).last().unwrap().clone();
    let before = d.clone();
    assert!(trim(&mut d, &c, 6500, 9001).unwrap_err().contains("3600"));
    assert_eq!(d, before);
    assert_ne!(d, baseline);
}
#[test]
fn a_split_excerpt_can_be_trimmed_and_copied_without_losing_its_original_progress() {
    let mut d = fixture();
    let original = d.clone();
    let first = clips(&d)[0].id.clone();
    support::split(&mut d, &first, 1000).unwrap();
    let right = clips(&d)[1].clone();
    trim(&mut d, &right, 1500, 3000).unwrap();
    let right = clips(&d)[1].clone();
    assert_eq!(right.effect_offset_ms, 1500);
    audio(
        &mut d,
        json!({"kind":"copyLightingClip","id":right.id,"startMs":7000}),
    )
    .unwrap();
    assert_eq!(sample(&d, 7307), sample(&original, 1807));
    for time in (1500..3000).step_by(17) {
        assert_eq!(sample(&d, time), sample(&original, time));
    }
    let mut missing = right;
    missing.id = "ffffffff-0000-4000-8000-000000000000".into();
    let before = d.clone();
    assert!(trim(&mut d, &missing, 1600, 3000).is_err());
    assert_eq!(d, before);
}
