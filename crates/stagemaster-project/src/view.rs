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
}
#[derive(Serialize)]
pub struct NamedView {
    pub id: String,
    pub name: String,
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
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneValue {
    pub fixture_id: String,
    pub attribute: String,
    pub mode: String,
    pub value: Option<u64>,
    pub preset_name: Option<String>,
}
pub(super) fn project(root: &Value) -> ProjectView {
    let lighting = &root["lighting"];
    ProjectView {
        id: text(&root["project"], "id").into(),
        name: text(&root["project"], "name").into(),
        description: text(&root["project"], "description").into(),
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
    let value = if source["kind"] == "preset" {
        preset
            .and_then(|p| {
                array(p, "values")
                    .iter()
                    .find(|v| v["target"] == entry["target"])
            })
            .and_then(|v| v["value"]["value"].as_u64())
    } else {
        source["value"]["value"].as_u64()
    };
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
