use crate::{JointPose, Light};
use stagemaster_project::{Document, FixturePlacement, PreviewOutput};
use stagemaster_spatial::Installation;

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
    placements: Vec<FixturePlacement>,
    profiles: Vec<stagemaster_project::ProfileView>,
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
            placements: view.stage.placements,
            profiles: view.profiles,
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
        Ok(lights(self, |fixture, key, default| {
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
        lights(self, |fixture, key, default| {
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
fn lights(rig: &LightRig, value: impl Fn(&str, &str, u64) -> u64) -> Vec<Light> {
    rig.fixtures
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
            let pose = f.positioning.as_ref().map(|model| {
                let placement = rig
                    .placements
                    .iter()
                    .find(|p| p.fixture_id == f.id)
                    .expect("placed fixture");
                let profile = rig
                    .profiles
                    .iter()
                    .find(|p| p.id == f.profile_id)
                    .expect("validated profile");
                let axis = |key: &str, m: &stagemaster_project::PositionAxis| {
                    let a = f
                        .attributes
                        .iter()
                        .find(|a| a.key == key)
                        .expect("validated axis");
                    let fine = profile
                        .channels
                        .iter()
                        .find(|c| c.attribute == key)
                        .expect("mapped axis")
                        .fine
                        .is_some();
                    m.decode(
                        u16::try_from(value(&f.id, key, a.default_value))
                            .expect("normalized value"),
                        fine,
                    )
                    .expect("validated range")
                };
                let install = Installation {
                    position_meters: placement
                        .position_meters
                        .numbers(100_000.0)
                        .expect("validated placement"),
                    rotation_degrees_xyz: placement
                        .rotation_degrees_xyz
                        .numbers(3600.0)
                        .expect("validated installation"),
                };
                let angles = stagemaster_spatial::positioning::JointAngles {
                    pan_degrees: axis("pan", &model.pan),
                    tilt_degrees: axis("tilt", &model.tilt),
                };
                let pose = model
                    .head(f.zero_correction.as_ref())
                    .expect("validated model")
                    .pose(install, angles)
                    .expect("decoded angles within travel");
                JointPose {
                    base: pose.base,
                    pan: pose.pan,
                    head: pose.head,
                    direction: pose.direction,
                }
            });
            Light {
                pose,
                fixture_id: f.id.clone(),
                intensity: if super::has_unmodeled_optics(f) {
                    0.0
                } else {
                    attribute("dimmer", 1.0)
                },
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
