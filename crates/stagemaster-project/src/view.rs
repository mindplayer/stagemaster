use crate::{array, text};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub profiles: Vec<ProfileView>,
    pub domains: Vec<NamedView>,
    pub fixtures: Vec<FixtureView>,
    pub scenes: Vec<SceneView>,
    pub groups: Vec<GroupView>,
    pub presets: Vec<PresetView>,
    pub sequences: Vec<SequenceView>,
    pub stage: crate::StageView,
}
#[derive(Serialize)]
pub struct NamedView {
    pub id: String,
    pub name: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupView {
    pub id: String,
    pub name: String,
    pub fixture_ids: Vec<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetView {
    pub id: String,
    pub name: String,
    pub values: Vec<SceneValue>,
    pub used_by_scenes: Vec<NamedView>,
    pub used_by_sequences: Vec<NamedView>,
}
#[derive(Serialize)]
pub struct ProfileView {
    pub id: String,
    pub name: String,
    pub footprint: u64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureView {
    pub id: String,
    pub name: String,
    pub profile_name: String,
    pub domain_name: String,
    pub domain_id: String,
    pub footprint: u64,
    pub universe: Option<u64>,
    pub address: Option<u64>,
    pub attributes: Vec<AttributeView>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeView {
    pub key: String,
    pub label: String,
    pub default_value: u64,
}
#[derive(Serialize)]
pub struct SceneView {
    pub id: String,
    pub name: String,
    pub values: Vec<SceneValue>,
    pub effects: Vec<crate::SceneEffect>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneValue {
    pub fixture_id: String,
    pub attribute: String,
    pub mode: String,
    pub value: Option<u64>,
    pub preset_name: Option<String>,
    pub preset_id: Option<String>,
}
#[derive(Serialize)]
pub struct SequenceView {
    pub id: String,
    pub name: String,
    pub tracking: String,
    pub repeat: String,
    pub steps: Vec<StepView>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepView {
    pub id: String,
    pub name: String,
    pub number: String,
    pub scene_id: String,
    pub delay_ms: u64,
    pub fade_ms: u64,
    pub wait_ms: Option<u64>,
}
pub(super) fn project(root: &Value) -> ProjectView {
    let lighting = &root["lighting"];
    ProjectView {
        stage: crate::stage::view(root),
        id: text(&root["project"], "id").into(),
        name: text(&root["project"], "name").into(),
        description: text(&root["project"], "description").into(),
        groups: array(lighting, "groups")
            .iter()
            .map(|g| GroupView {
                id: text(g, "id").into(),
                name: text(g, "name").into(),
                fixture_ids: array(g, "fixtureIds")
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect(),
            })
            .collect(),
        presets: array(lighting, "presets")
            .iter()
            .map(|p| preset(lighting, p))
            .collect(),
        sequences: array(lighting, "sequences")
            .iter()
            .map(|seq| SequenceView {
                id: text(seq, "id").into(),
                name: text(seq, "name").into(),
                tracking: text(seq, "tracking").into(),
                repeat: text(seq, "repeat").into(),
                steps: array(seq, "steps")
                    .iter()
                    .map(|step| StepView {
                        id: text(step, "id").into(),
                        name: text(step, "name").into(),
                        number: text(step, "number").into(),
                        scene_id: text(step, "sceneId").into(),
                        delay_ms: crate::sequence::duration_ms(&step["delay"])
                            .expect("validated time"),
                        fade_ms: crate::sequence::duration_ms(&step["fade"])
                            .expect("validated time"),
                        wait_ms: (step["advance"]["kind"] == "after").then(|| {
                            crate::sequence::duration_ms(&step["advance"]["wait"])
                                .expect("validated time")
                        }),
                    })
                    .collect(),
            })
            .collect(),
        profiles: array(lighting, "profiles")
            .iter()
            .map(|p| ProfileView {
                id: text(p, "id").into(),
                name: text(p, "name").into(),
                footprint: p["footprint"].as_u64().unwrap_or_default(),
            })
            .collect(),
        domains: array(root, "domains")
            .iter()
            .map(|d| NamedView {
                id: text(d, "id").into(),
                name: text(d, "name").into(),
            })
            .collect(),
        fixtures: array(lighting, "fixtures")
            .iter()
            .map(|f| fixture(root, f))
            .collect(),
        scenes: array(lighting, "scenes")
            .iter()
            .map(|scene| SceneView {
                effects: crate::effects::read(scene),
                id: text(scene, "id").into(),
                name: text(scene, "name").into(),
                values: array(scene, "assignments")
                    .iter()
                    .map(|entry| scene_value(lighting, entry))
                    .collect(),
            })
            .collect(),
    }
}
fn fixture(root: &Value, fixture: &Value) -> FixtureView {
    let lighting = &root["lighting"];
    let profile = array(lighting, "profiles")
        .iter()
        .find(|p| p["id"] == fixture["profileId"])
        .expect("validated profile");
    let domain = array(root, "domains")
        .iter()
        .find(|d| d["id"] == fixture["domainId"])
        .expect("validated domain");
    let patch = array(lighting, "patches")
        .iter()
        .find(|p| p["fixtureId"] == fixture["id"]);
    FixtureView {
        id: text(fixture, "id").into(),
        name: text(fixture, "name").into(),
        profile_name: text(profile, "name").into(),
        domain_name: text(domain, "name").into(),
        domain_id: text(fixture, "domainId").into(),
        footprint: profile["footprint"].as_u64().unwrap_or_default(),
        universe: patch.and_then(|p| p["universe"].as_u64()),
        address: patch.and_then(|p| p["address"].as_u64()),
        attributes: array(profile, "attributes")
            .iter()
            .map(|a| AttributeView {
                key: text(a, "key").into(),
                label: attribute_label(text(a, "key")).into(),
                default_value: a["default"]["value"].as_u64().unwrap_or_default(),
            })
            .collect(),
    }
}
fn scene_value(lighting: &Value, entry: &Value) -> SceneValue {
    let source = &entry["source"];
    let preset = array(lighting, "presets")
        .iter()
        .find(|p| p["id"] == source["presetId"]);
    let value = crate::library::resolved_value(lighting, entry).and_then(|v| v["value"].as_u64());
    SceneValue {
        fixture_id: text(&entry["target"], "fixtureId").into(),
        attribute: text(&entry["target"], "attribute").into(),
        mode: if entry["operation"] == "release" {
            "release".into()
        } else {
            text(source, "kind").into()
        },
        value,
        preset_name: preset.map(|p| text(p, "name").into()),
        preset_id: preset.map(|p| text(p, "id").into()),
    }
}
fn attribute_label(key: &str) -> &str {
    match key {
        "dimmer" => "亮度",
        "red" => "红色",
        "green" => "绿色",
        "blue" => "蓝色",
        "pan" => "水平",
        "tilt" => "垂直",
        _ => key,
    }
}

fn preset(lighting: &Value, preset: &Value) -> PresetView {
    let used_by_scenes: Vec<NamedView> = array(lighting, "scenes")
        .iter()
        .filter(|s| {
            array(s, "assignments")
                .iter()
                .any(|a| a["source"]["presetId"] == preset["id"])
        })
        .map(|s| NamedView {
            id: text(s, "id").into(),
            name: text(s, "name").into(),
        })
        .collect();
    let used_by_sequences = array(lighting, "sequences")
        .iter()
        .filter(|s| {
            array(s, "steps").iter().any(|step| {
                used_by_scenes
                    .iter()
                    .any(|scene| step["sceneId"] == scene.id)
            })
        })
        .map(|s| NamedView {
            id: text(s, "id").into(),
            name: text(s, "name").into(),
        })
        .collect();
    PresetView {
        id: text(preset, "id").into(),
        name: text(preset, "name").into(),
        values: array(preset, "values")
            .iter()
            .map(|v| SceneValue {
                fixture_id: text(&v["target"], "fixtureId").into(),
                attribute: text(&v["target"], "attribute").into(),
                mode: "literal".into(),
                value: v["value"]["value"].as_u64(),
                preset_id: None,
                preset_name: None,
            })
            .collect(),
        used_by_scenes,
        used_by_sequences,
    }
}
