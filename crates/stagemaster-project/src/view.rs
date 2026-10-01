use crate::{array, text};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectView {
    pub audio: Option<crate::AudioTimeline>,
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
    pub positioning: Option<crate::PositionModel>,
    pub revision: String,
    pub manufacturer: String,
    pub model: String,
    pub mode: String,
    pub channels: Vec<crate::ProfileChannel>,
    pub authorable: bool,
    pub id: String,
    pub name: String,
    pub footprint: u64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureView {
    pub positioning: Option<crate::PositionModel>,
    pub zero_correction: Option<crate::FixtureZero>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_reference: Option<crate::PositionReferenceView>,
    pub profile_id: String,
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
    pub function: Option<FunctionAttributeView>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionAttributeView {
    pub functions: Vec<crate::FunctionDefinition>,
    pub default: crate::FunctionSelection,
    pub fine: bool,
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
    pub function_value: Option<crate::FunctionSelection>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<crate::StepScript>,
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
        audio: crate::audio::read(root),
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
                        script: crate::sequence_script::read(step),
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
            .map(crate::fixture_view::profile)
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
        position_reference: crate::position::reference::view::project(root, fixture, profile),
        positioning: crate::position::model(profile).expect("validated model"),
        zero_correction: fixture
            .get("zeroCorrection")
            .map(|v| serde_json::from_value(v.clone()).expect("validated zero")),
        profile_id: text(fixture, "profileId").into(),
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
                default_value: u64::from(
                    crate::fixture_value::encode(profile, text(a, "key"), &a["default"])
                        .expect("validated default"),
                ),
                function: crate::fixture_view::function_attribute(profile, a),
            })
            .collect(),
    }
}
fn scene_value(lighting: &Value, entry: &Value) -> SceneValue {
    let source = &entry["source"];
    let preset = array(lighting, "presets")
        .iter()
        .find(|p| p["id"] == source["presetId"]);
    let resolved = crate::library::resolved_value(lighting, entry);
    let profile = crate::fixture_value::profile(lighting, text(&entry["target"], "fixtureId"))
        .expect("validated fixture");
    let value = resolved.as_ref().map(|v| {
        u64::from(
            crate::fixture_value::encode(profile, text(&entry["target"], "attribute"), v)
                .expect("validated value"),
        )
    });
    let function_value = resolved
        .as_ref()
        .filter(|v| v["kind"] == "function")
        .map(|v| crate::fixture_value::selection(v).expect("validated selection"));
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
        function_value,
    }
}
pub(super) fn attribute_label(key: &str) -> &str {
    match key {
        "dimmer" => "亮度",
        "pan" => "水平轴",
        "tilt" => "垂直轴",
        "red" => "红色",
        "green" => "绿色",
        "blue" => "蓝色",
        "color-wheel" => "色盘",
        "gobo-wheel" => "图案盘",
        "shutter" => "快门与频闪",
        "prism" => "棱镜",
        _ => crate::fixture_optics::label(key).unwrap_or(key),
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
                value: Some(u64::from(
                    crate::fixture_value::encode(
                        crate::fixture_value::profile(lighting, text(&v["target"], "fixtureId"))
                            .expect("validated fixture"),
                        text(&v["target"], "attribute"),
                        &v["value"],
                    )
                    .expect("validated value"),
                )),
                function_value: (v["value"]["kind"] == "function").then(|| {
                    crate::fixture_value::selection(&v["value"]).expect("validated selection")
                }),
                preset_id: None,
                preset_name: None,
            })
            .collect(),
        used_by_scenes,
        used_by_sequences,
    }
}
