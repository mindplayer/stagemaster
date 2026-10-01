//! Read-only projections for interchangeable renderers. No engine, sockets or device output.
mod construction_mesh;
mod geometry;
mod light_rig;
pub use light_rig::{LightRig, editing_lights, playback_lights};
mod rigging;
mod seating;
use serde::Serialize;
use stagemaster_project::{Document, FixturePlacement, ProjectView};
use stagemaster_spatial::Installation;

pub type Triangle = [[f64; 3]; 3];
pub const MAX_TRIANGLES: usize = 100_000;
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mesh {
    pub id: String,
    pub name: String,
    pub color: [f32; 3],
    pub view_role: &'static str,
    pub triangles: Vec<Triangle>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    pub moving: bool,
    pub id: String,
    pub name: String,
    pub placement: FixturePlacement,
    pub origin_meters: [f64; 3],
    pub direction: [f64; 3],
    /// Generic, illustrative optics until a measured optical profile is available.
    pub full_beam_angle_degrees: f64,
    pub optics: &'static str,
    /// Additive renderer capability hint. Unmodeled fixtures still receive zero beam intensity.
    pub light_simulation: &'static str,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub project_id: String,
    pub project_name: String,
    pub spaces: Vec<stagemaster_project::StageSpace>,
    pub meshes: Vec<Mesh>,
    pub fixtures: Vec<Fixture>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Light {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pose: Option<JointPose>,
    pub fixture_id: String,
    pub intensity: f64,
    /// Normalized RGB emitter strengths, separate from dimming.
    pub color: [f64; 3],
}

#[derive(Clone, Debug, Serialize)]
pub struct JointPose {
    pub base: [[f64; 3]; 2],
    pub pan: [[f64; 3]; 2],
    pub head: [[f64; 3]; 2],
    pub direction: [f64; 3],
}
/// Build rendering geometry from a validated authoritative editing document.
/// # Errors
/// Refuses invalid projections and preview geometry budgets without altering the document.
pub fn scene(document: &Document) -> Result<Scene, String> {
    let view = document.view();
    let meshes = crate::construction_mesh::build(&view.stage)?;
    let fixtures = fixtures(&view)?;
    Ok(Scene {
        project_id: view.id,
        project_name: view.name,
        spaces: view.stage.spaces,
        meshes,
        fixtures,
    })
}

fn fixtures(view: &ProjectView) -> Result<Vec<Fixture>, String> {
    view.stage
        .placements
        .iter()
        .map(|placement| {
            let fixture = view
                .fixtures
                .iter()
                .find(|f| f.id == placement.fixture_id)
                .ok_or("灯位的灯具缺失")?;
            let mut attributes = fixture
                .attributes
                .iter()
                .filter(|a| a.function.is_none())
                .map(|a| a.key.as_str())
                .collect::<Vec<_>>();
            if fixture.positioning.is_some() {
                attributes.retain(|&k| k != "pan" && k != "tilt");
            }
            attributes.sort_unstable();
            if attributes != ["dimmer"]
                && attributes != ["blue", "dimmer", "green", "red"]
                && attributes != ["blue", "green", "red"]
            {
                return Err(format!("灯具“{}”的档案尚未支持三维预演", fixture.name));
            }
            let installation = Installation {
                position_meters: placement.position_meters.numbers(100_000.0)?,
                rotation_degrees_xyz: placement.rotation_degrees_xyz.numbers(3600.0)?,
            };
            let ray = installation.fixed_ray().map_err(|_| "灯具安装变换无效")?;
            Ok(Fixture {
                moving: fixture.positioning.is_some(),
                id: fixture.id.clone(),
                name: fixture.name.clone(),
                placement: placement.clone(),
                origin_meters: ray.origin_meters,
                direction: ray.direction,
                full_beam_angle_degrees: 25.0,
                optics: "generic-illustrative",
                light_simulation: if has_function_optics(fixture) {
                    "unmodeled-functions"
                } else {
                    "dimmer-rgb"
                },
            })
        })
        .collect::<Result<_, String>>()
}

fn number(value: &str) -> Result<f64, String> {
    value.parse::<f64>().map_err(|_| "场地数值无效".into())
}
fn points(values: &[[String; 2]]) -> Result<Vec<[f64; 2]>, String> {
    values
        .iter()
        .map(|p| Ok([number(&p[0])?, number(&p[1])?]))
        .collect()
}

fn has_function_optics(fixture: &stagemaster_project::FixtureView) -> bool {
    fixture.attributes.iter().any(|a| a.function.is_some())
}
