#[path = "support/audio_split.rs"]
#[allow(dead_code)]
mod support;
use serde_json::{Value, json};
use stagemaster_playback::Command;
use stagemaster_project::{Document, LiveSourceBudget};
use support::{audio, clips, split};

fn reference(doc: &Document, position: u64) -> Vec<u16> {
    let track = doc.audio_timeline().unwrap();
    let segment = track.lighting_at(position);
    let compiled = doc.compile_audio_segment(segment.map(|s| s.id)).unwrap();
    let mut player = compiled.playback.into_player().unwrap();
    player
        .advance(position - segment.map_or(0, |s| s.start_ms))
        .unwrap();
    player.values().to_vec()
}
fn compare(doc: &Document) {
    let mut prepared = doc.compile_live_audio().unwrap();
    prepared.apply_timeline(Command::Execute(0), 0).unwrap();
    let mut values = vec![None; prepared.layout().attributes().len()];
    let mut claims = values.iter().map(|_| None).collect::<Vec<Option<u64>>>();
    let times = [
        0, 1, 250, 499, 500, 999, 1000, 3999, 4000, 4107, 4250, 4499, 4500, 4999, 5000, 5500, 5999,
        6000, 9999, 10000,
    ];
    for time in times {
        prepared.apply_timeline(Command::Advance, time).unwrap();
        prepared
            .copy_contribution(&mut values, &mut claims)
            .unwrap();
        assert_eq!(
            values,
            reference(doc, time)
                .into_iter()
                .map(Some)
                .collect::<Vec<_>>(),
            "at {time}"
        );
        prepared.acknowledge_contribution();
    }
    let saved = values.clone();
    for bad in [9999, 10001, u64::MAX] {
        assert!(prepared.apply_timeline(Command::Advance, bad).is_err());
        prepared
            .copy_contribution(&mut values, &mut claims)
            .unwrap();
        assert_eq!(values, saved);
    }
}
#[test]
fn prepared_snapshot_fades_and_sliced_history_match_original_segment_compiler() {
    let mut doc = support::fixture();
    compare(&doc);
    let first = clips(&doc)[0].id.clone();
    split(&mut doc, &first, 250).unwrap();
    let second = doc
        .audio_timeline()
        .unwrap()
        .lighting_at(4500)
        .unwrap()
        .id
        .to_owned();
    split(&mut doc, &second, 4499).unwrap();
    compare(&doc);
    let disabled = clips(&doc)[0].id.clone();
    audio(&mut doc, json!({"kind":"editLightingClips","ids":[disabled],"action":{"kind":"enabled","enabled":false}})).unwrap();
    compare(&doc);
}
#[test]
fn prepared_dynamic_crossfade_and_repeated_slices_keep_both_source_clocks() {
    let original = support::fixture();
    let mut root: Value = serde_json::from_slice(&original.encode().unwrap()).unwrap();
    let mut effect = root["lighting"]["scenes"][0]["effects"][0].clone();
    effect["id"] = json!("b0000000-0000-4000-8000-000000000002");
    effect["periodMs"] = json!(1733);
    root["lighting"]["scenes"][1]["effects"] = json!([effect]);
    let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let mut next = clips(&doc)[1].clone();
    next.fade_mode = stagemaster_project::ClipFadeMode::Dynamic;
    next.fade_ms = 1000;
    audio(&mut doc, json!({"kind":"putLightingClip","clip":next})).unwrap();
    compare(&doc);
    for time in [4107, 4499, 4999] {
        let id = doc
            .audio_timeline()
            .unwrap()
            .lighting_at(time)
            .unwrap()
            .id
            .to_owned();
        split(&mut doc, &id, time).unwrap();
    }
    compare(&doc);
    assert_eq!(doc.compile_live_audio().unwrap().budget().targets, 10);
}
#[test]
fn legacy_markers_non_lighting_beats_and_end_hold_match_the_existing_track() {
    let mut doc = support::fixture();
    audio(&mut doc, json!({"kind":"clear"})).unwrap();
    audio(&mut doc, json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":10000}})).unwrap();
    for (n, time, scene, fade) in [
        (1, 0, Some(doc.view().scenes[0].id.clone()), 500),
        (2, 3000, None, 0),
        (3, 4000, Some(doc.view().scenes[1].id.clone()), 750),
    ] {
        audio(&mut doc, json!({"kind":"putMarker","marker":{"id":format!("a0000000-0000-4000-8000-{n:012}"),"name":"卡点","timeMs":time,"sceneId":scene,"fadeMs":fade}})).unwrap();
    }
    compare(&doc);
}
#[test]
fn residency_budget_counts_both_transition_plans_and_checks_overflow_atomically() {
    let doc = support::fixture();
    assert_eq!(doc.compile_live_audio().unwrap().budget().targets, 3);
    let mut budget = LiveSourceBudget::default();
    budget
        .add(LiveSourceBudget {
            targets: 1,
            effect_channels: 2,
            keyframes: 3,
        })
        .unwrap();
    for value in [
        LiveSourceBudget {
            targets: usize::MAX,
            ..LiveSourceBudget::default()
        },
        LiveSourceBudget {
            effect_channels: 16_384,
            ..LiveSourceBudget::default()
        },
        LiveSourceBudget {
            keyframes: 131_072,
            ..LiveSourceBudget::default()
        },
    ] {
        assert!(budget.add(value).is_err());
        assert_eq!(
            (budget.targets, budget.effect_channels, budget.keyframes),
            (1, 2, 3)
        );
    }
}
