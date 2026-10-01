//! A section becomes one mesh; seats are illustrative solids with bounded complexity.
use crate::{Triangle, geometry};
use stagemaster_project::SeatingShape;
pub(super) fn mesh(shape: &SeatingShape) -> Result<Vec<Triangle>, String> {
    let layout = shape.layout()?;
    let mut mesh = Vec::with_capacity(layout.centers.len() * 72);
    let (sin, cos) = layout.yaw_radians.sin_cos();
    let w = layout.seat_width;
    let d = layout.seat_depth;
    for center in &layout.centers {
        let world = |x: f64, y: f64| [center[0] + x * cos - y * sin, center[1] + x * sin + y * cos];
        let mut solid = |x: f64, y: f64, width: f64, depth: f64, bottom: f64, top: f64| {
            geometry::wall(
                &mut mesh,
                world(x - width / 2.0, y),
                world(x + width / 2.0, y),
                depth,
                layout.floor + bottom,
                layout.floor + top,
            );
        };
        solid(0.0, 0.0, w, d, 0.40, 0.45);
        solid(0.0, -d / 2.0 + 0.025, w, 0.05, 0.45, 0.85);
        for x in [-w / 2.0 + 0.03, w / 2.0 - 0.03] {
            for y in [-d / 2.0 + 0.03, d / 2.0 - 0.03] {
                solid(x, y, 0.035, 0.035, 0.0, 0.40);
            }
        }
    }
    Ok(mesh)
}
