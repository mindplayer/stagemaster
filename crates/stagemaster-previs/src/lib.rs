//! Read-only projections for interchangeable renderers. No engine, sockets or device output.
mod geometry;
use serde::Serialize;
use stagemaster_project::{
    ConstructionShape, Document, FixturePlacement, PreviewOutput, ProjectView,
};
use stagemaster_spatial::Installation;

pub type Triangle = [[f64; 3]; 3];
pub const MAX_TRIANGLES: usize = 100_000;
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mesh {
    pub id: String,
    pub name: String,
    pub color: [f32; 3],
    pub triangles: Vec<Triangle>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    pub id: String,
    pub name: String,
    pub placement: FixturePlacement,
    pub origin_meters: [f64; 3],
    pub direction: [f64; 3],
    /// Generic, illustrative optics until a measured optical profile is available.
    pub full_beam_angle_degrees: f64,
    pub optics: &'static str,
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
    pub fixture_id: String,
    pub intensity: f64,
    /// Normalized RGB emitter strengths, separate from dimming.
    pub color: [f64; 3],
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
        let color = match &construction.shape {
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
        triangle_count += triangles.len();
        if triangle_count > MAX_TRIANGLES {
            return Err("场地超出当前预演的 100000 个三角面限制".into());
        }
        meshes.push(Mesh {
            id: construction.id.clone(),
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
                .map(|a| a.key.as_str())
                .collect::<Vec<_>>();
            attributes.sort_unstable();
            if attributes != ["dimmer"] && attributes != ["blue", "dimmer", "green", "red"] {
                return Err(format!("灯具“{}”的档案尚未支持三维预演", fixture.name));
            }
            let installation = Installation {
                position_meters: placement.position_meters.numbers(100_000.0)?,
                rotation_degrees_xyz: placement.rotation_degrees_xyz.numbers(3600.0)?,
            };
            let ray = installation.fixed_ray().map_err(|_| "灯具安装变换无效")?;
            Ok(Fixture {
                id: fixture.id.clone(),
                name: fixture.name.clone(),
                placement: placement.clone(),
                origin_meters: ray.origin_meters,
                direction: ray.direction,
                full_beam_angle_degrees: 25.0,
                optics: "generic-illustrative",
            })
        })
        .collect::<Result<_, String>>()
}

/// Project a committed scene (or profile defaults), resolving preset references in Rust.
/// It deliberately does not carry sequence tracking into an isolated editing view.
/// # Errors
/// Rejects a deleted scene. Unsupported attributes remain outside this generic-light projection.
pub fn editing_lights(document: &Document, scene_id: Option<&str>) -> Result<Vec<Light>, String> {
    LightRig::new(document).editing(scene_id)
}

/// Cached fixture/scene projection owned by one document content version.
/// A host must rebuild it on content changes; polling does not reconstruct the project view.
pub struct LightRig {
    fixtures: Vec<stagemaster_project::FixtureView>,
    scenes: Vec<stagemaster_project::SceneView>,
}
impl LightRig {
    #[must_use]
    pub fn new(document: &Document) -> Self {
        let mut view = document.view();
        let placed = view
            .stage
            .placements
            .iter()
            .map(|p| p.fixture_id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        view.fixtures.retain(|f| placed.contains(f.id.as_str()));
        Self {
            fixtures: view.fixtures,
            scenes: view.scenes,
        }
    }
    /// Resolve a committed scene without sequence tracking.
    /// # Errors
    /// Rejects a missing scene in this version of the document.
    pub fn editing(&self, scene_id: Option<&str>) -> Result<Vec<Light>, String> {
        let scene = scene_id
            .map(|id| {
                self.scenes
                    .iter()
                    .find(|s| s.id == id)
                    .ok_or("预演场景不存在")
            })
            .transpose()?;
        Ok(lights(&self.fixtures, |fixture, key, default| {
            scene
                .and_then(|scene| {
                    scene
                        .values
                        .iter()
                        .find(|v| v.fixture_id == fixture && v.attribute == key)
                })
                .filter(|v| v.mode != "release")
                .and_then(|v| v.value)
                .unwrap_or(default)
        }))
    }
    /// The host must verify that the player output belongs to this rig's document version.
    #[must_use]
    pub fn playback(&self, output: &PreviewOutput) -> Vec<Light> {
        lights(&self.fixtures, |fixture, key, default| {
            output
                .fixtures
                .iter()
                .find(|f| f.id == fixture)
                .and_then(|f| f.attributes.iter().find(|a| a.key == key))
                .map_or(default, |a| u64::from(a.value))
        })
    }
}
/// Project the same encoded preview source used by the independent player monitor.
/// The caller must match document/scene versions before combining these results.
#[must_use]
pub fn playback_lights(document: &Document, output: &PreviewOutput) -> Vec<Light> {
    LightRig::new(document).playback(output)
}
fn lights(
    fixtures: &[stagemaster_project::FixtureView],
    value: impl Fn(&str, &str, u64) -> u64,
) -> Vec<Light> {
    fixtures
        .iter()
        .map(|f| {
            let attribute = |key: &str, fallback: f64| {
                f.attributes
                    .iter()
                    .find(|a| a.key == key)
                    .map_or(fallback, |a| {
                        f64::from(
                            u16::try_from(value(&f.id, key, a.default_value)).unwrap_or(u16::MAX),
                        ) / 65535.0
                    })
            };
            let rgb = ["red", "green", "blue"]
                .iter()
                .all(|key| f.attributes.iter().any(|a| a.key == *key));
            Light {
                fixture_id: f.id.clone(),
                intensity: attribute("dimmer", 1.0),
                color: if rgb {
                    [
                        attribute("red", 0.0),
                        attribute("green", 0.0),
                        attribute("blue", 0.0),
                    ]
                } else {
                    [1.0; 3]
                },
            }
        })
        .collect()
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
