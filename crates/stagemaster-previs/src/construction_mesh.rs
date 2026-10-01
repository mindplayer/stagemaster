//! Construction geometry and mesh assembly, shared by renderer adapters.
use crate::{MAX_TRIANGLES, Mesh, Triangle, geometry, number, points, rigging, seating};
use stagemaster_project::{ConstructionShape, StageConstruction, StageView};
struct Geometry {
    triangles: Vec<Triangle>,
    floor: Option<Vec<Triangle>>,
    color: [f32; 3],
}
fn geometry(construction: &StageConstruction, stage: &StageView) -> Result<Geometry, String> {
    let mut triangles = Vec::new();
    let mut floor_triangles = None;
    let color = match &construction.shape {
        ConstructionShape::Seating(section) => {
            triangles = seating::mesh(section)?;
            [0.19, 0.25, 0.31]
        }
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
            let space = stage
                .spaces
                .iter()
                .find(|s| s.id == *space_id)
                .ok_or("围护空间缺失")?;
            let outline = points(&space.outline_meters)?;
            let floor = number(&space.floor_elevation_meters)?;
            let top = floor + number(space.clear_height_meters.as_deref().ok_or("围护净高缺失")?)?;
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
    Ok(Geometry {
        triangles,
        floor: floor_triangles,
        color,
    })
}
pub(super) fn build(stage: &StageView) -> Result<Vec<Mesh>, String> {
    let mut meshes = Vec::new();
    let mut triangle_count = 0;
    for construction in &stage.constructions {
        let Geometry {
            triangles,
            floor: floor_triangles,
            color,
        } = geometry(construction, stage)?;
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
    Ok(meshes)
}
