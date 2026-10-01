//! Read-only projections for interchangeable renderers. No engine, sockets or device output.
mod geometry;
mod light_rig;
pub use light_rig::{LightRig, editing_lights, playback_lights};
mod rigging;
use serde::Serialize;
use stagemaster_project::{ConstructionShape, Document, FixturePlacement, ProjectView};
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
    let mut meshes = Vec::new();
    let mut triangle_count = 0;
    for construction in &view.stage.constructions {
        let mut triangles = Vec::new();
        let mut floor_triangles = None;
        let color = match &construction.shape {
            ConstructionShape::Rig(rig) => {
                triangles = rigging::mesh(rig)?;
                [0.58, 0.62, 0.68]
            }
            ConstructionShape::Platform {
                outline_meters,
                base_elevation_meters,
                height_meters,
                ..
            } => {
                let z = number(base_elevation_meters)?;
                geometry::prism(
                    &mut triangles,
                    &points(outline_meters)?,
                    z,
                    z + number(height_meters)?,
                )?;
                [0.30, 0.32, 0.35]
            }
            ConstructionShape::Enclosure {
                space_id,
                wall_thickness_meters,
                floor_thickness_meters,
                ceiling_thickness_meters,
            } => {
                let space = view
                    .stage
                    .spaces
                    .iter()
                    .find(|s| s.id == *space_id)
                    .ok_or("围护空间缺失")?;
                let outline = points(&space.outline_meters)?;
                let floor = number(&space.floor_elevation_meters)?;
                let top =
                    floor + number(space.clear_height_meters.as_deref().ok_or("围护净高缺失")?)?;
                geometry::prism(
                    &mut triangles,
                    &outline,
                    floor - number(floor_thickness_meters)?,
                    floor,
                )?;
                floor_triangles = Some(std::mem::take(&mut triangles));
                if let Some(thickness) = ceiling_thickness_meters {
                    geometry::prism(&mut triangles, &outline, top, top + number(thickness)?)?;
                }
                let thickness = number(wall_thickness_meters)?;
                for (index, a) in outline.iter().enumerate() {
                    let b = outline[(index + 1) % outline.len()];
                    geometry::wall(&mut triangles, *a, b, thickness, floor, top);
                }
                [0.18, 0.21, 0.25]
            }
        };
        triangle_count += triangles.len() + floor_triangles.as_ref().map_or(0, Vec::len);
        if triangle_count > MAX_TRIANGLES {
            return Err("场地超出当前预演的 100000 个三角面限制".into());
        }
        let is_enclosure = floor_triangles.is_some();
        if let Some(floor) = floor_triangles {
            meshes.push(Mesh {
                id: format!("{}:floor", construction.id),
                name: construction.name.clone(),
                triangles: floor,
                color,
                view_role: "solid",
            });
        }
        meshes.push(Mesh {
            id: if is_enclosure {
                format!("{}:shell", construction.id)
            } else {
                construction.id.clone()
            },
            view_role: if is_enclosure {
                "enclosureShell"
            } else {
                "solid"
            },
            name: construction.name.clone(),
            triangles,
            color,
        });
    }
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
