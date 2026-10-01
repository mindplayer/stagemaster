#[path = "support/audio_split.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_project::Document;
use support::{audio, clips, fixture, sample, split};
fn root() -> Value {
    let mut document = fixture();
    let id = clips(&document)[0].id.clone();
    split(&mut document, &id, 133).unwrap();
    serde_json::from_slice(&document.encode().unwrap()).unwrap()
}
fn decode(root: &Value) -> Result<Document, String> {
    Document::decode(&serde_json::to_vec(root).unwrap())
}
#[test]
fn strict_snapshots_require_capability_valid_references_and_consistent_visible_time() {
    let baseline = root();
    for (field, values) in [
        (
            "offsetMs",
            vec![
                json!(-1),
                json!(0.5),
                json!("1"),
                Value::Null,
                json!(3_600_000),
            ],
        ),
        ("durationMs", vec![json!(0), json!(3_600_001), json!(132)]),
        ("unknown", vec![json!(1)]),
    ] {
        for value in values {
            let mut root = baseline.clone();
            root["media"]["audioEditing"]["lightingClips"][0]["entryFade"][field] = value;
            assert!(decode(&root).is_err(), "field {field}");
        }
    }
    for (field, value) in [
        ("fixtureId", json!(baseline["project"]["id"])),
        ("attribute", json!("missing")),
        ("value", json!(65536)),
        ("value", json!(-1)),
        ("address", json!(1)),
    ] {
        let mut root = baseline.clone();
        root["media"]["audioEditing"]["lightingClips"][0]["entryFade"]["from"][0][field] = value;
        assert!(decode(&root).is_err());
    }
    let mut duplicate = baseline.clone();
    let values = duplicate["media"]["audioEditing"]["lightingClips"][0]["entryFade"]["from"]
        .as_array_mut()
        .unwrap();
    values.push(values[0].clone());
    assert!(decode(&duplicate).unwrap_err().contains("重复"));
    let mut missing = baseline.clone();
    missing["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "media.audio-clip-fade");
    assert!(decode(&missing).unwrap_err().contains("能力"));
    let mut bare: Value =
        serde_json::from_slice(&Document::new("空").unwrap().encode().unwrap()).unwrap();
    bare["requires"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"media.audio-clip-fade","version":1}));
    assert!(decode(&bare).unwrap_err().contains("轨道"));
    let mut document = decode(&baseline).unwrap();
    audio(&mut document, json!({"kind":"clear"})).unwrap();
    assert!(
        !String::from_utf8(document.encode().unwrap())
            .unwrap()
            .contains("media.audio-clip-fade")
    );
}
#[test]
fn source_identity_survives_reordering_and_repatching_and_missing_identity_is_not_dropped() {
    let mut document = fixture();
    let view = document.view();
    document.edit(serde_json::from_value(json!({"op":"addFixture","name":"第二灯","profileId":view.profiles[0].id,"domainId":view.domains[0].id,"universe":1,"address":10})).unwrap()).unwrap();
    let id = clips(&document)[0].id.clone();
    split(&mut document, &id, 99).unwrap();
    let before = sample(&document, 199);
    let mut root: Value = serde_json::from_slice(&document.encode().unwrap()).unwrap();
    root["lighting"]["fixtures"]
        .as_array_mut()
        .unwrap()
        .reverse();
    root["lighting"]["patches"][0]["address"] = json!(20);
    let reordered = decode(&root).unwrap();
    assert_eq!(
        sample(&reordered, 199),
        before.into_iter().rev().collect::<Vec<_>>()
    );
    // Removing the first identity and its effect must still fail on the preserved fade reference.
    let missing = root["lighting"]["fixtures"]
        .as_array_mut()
        .unwrap()
        .pop()
        .unwrap();
    root["lighting"]["patches"]
        .as_array_mut()
        .unwrap()
        .retain(|p| p["fixtureId"] != missing["id"]);
    root["lighting"]["scenes"][0]["effects"] = json!([]);
    assert!(decode(&root).unwrap_err().contains("起始值引用的灯具"));
}
#[test]
fn snapshot_budget_is_bounded_and_excessive_split_is_atomic() {
    let mut root = root();
    let original = root["lighting"]["fixtures"][0].clone();
    let fixtures: Vec<Value> = (0..512)
        .map(|index| {
            let mut f = original.clone();
            f["id"] = json!(format!("e0000000-0000-4000-8000-{index:012}"));
            f
        })
        .collect();
    let from: Vec<Value> = fixtures
        .iter()
        .map(|f| json!({"fixtureId":f["id"],"attribute":"dimmer","value":0}))
        .collect();
    let original_id = original["id"].clone();
    root["lighting"]["fixtures"] = json!(fixtures);
    root["lighting"]["patches"][0]["fixtureId"] = json!(fixtures[0]["id"]);
    root["lighting"]["scenes"][0]["effects"][0]["fixtureIds"] = json!([fixtures[0]["id"]]);
    for scene in root["lighting"]["scenes"].as_array_mut().unwrap() {
        for a in scene["assignments"].as_array_mut().unwrap() {
            if a["target"]["fixtureId"] == original_id {
                a["target"]["fixtureId"] = fixtures[0]["id"].clone();
            }
        }
    }
    let source = root["media"]["audioEditing"]["lightingClips"][0].clone();
    root["media"]["audioEditing"]["lightingClips"] = Value::Array(
        (0..64)
            .map(|index| {
                let mut clip = source.clone();
                clip["id"] = json!(format!("c0000000-0000-4000-8000-{index:012}"));
                clip["startMs"] = json!(index * 10);
                clip["endMs"] = json!(index * 10 + 10);
                clip["fadeMs"] = json!(10);
                clip["entryFade"]["from"] = json!(from);
                clip
            })
            .collect(),
    );
    let mut document = decode(&root).unwrap();
    let baseline = document.clone();
    let id = clips(&document)[0].id.clone();
    assert!(split(&mut document, &id, 5).unwrap_err().contains("32768"));
    assert_eq!(document, baseline);
}
