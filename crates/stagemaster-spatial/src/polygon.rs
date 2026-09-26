//! Bounded planar geometry, using OGC validity checks and Earcut from geo.
use geo::{Area, Coord, LineString, Polygon, TriangulateEarcut, Validation};

#[derive(Clone, Debug)]
pub struct FloorPlan {
    pub area_square_meters: f64,
    /// Counterclockwise triangles in the project's right-handed XY plane.
    pub triangles: Vec<[[f64; 2]; 3]>,
}

/// Validate and triangulate one simple (possibly concave) ring, with no holes.
/// The caller omits the repeated closing point. No repair silently changes a design.
/// # Errors
/// Rejects non-finite, oversized, repeated, tiny, degenerate or self-crossing rings.
pub fn floor_plan(points: &[[f64; 2]]) -> Result<FloorPlan, String> {
    if !(3..=128).contains(&points.len()) {
        return Err("轮廓需要 3–128 个顶点".into());
    }
    if points
        .iter()
        .flatten()
        .any(|v| !v.is_finite() || v.abs() > 100_000.0)
    {
        return Err("轮廓坐标必须在 ±100000 米内".into());
    }
    for (index, a) in points.iter().enumerate() {
        let b = points[(index + 1) % points.len()];
        if (a[0] - b[0]).hypot(a[1] - b[1]) < 0.001 {
            return Err("相邻轮廓顶点至少相隔 1 毫米，末点不重复首点".into());
        }
    }
    let polygon = Polygon::new(
        LineString::new(points.iter().map(|p| Coord { x: p[0], y: p[1] }).collect()),
        vec![],
    );
    if !polygon.is_valid() {
        return Err("轮廓不能自交、重复交叠或退化".into());
    }
    let area = polygon.unsigned_area();
    if !(0.01..=1_000_000.0).contains(&area) {
        return Err("轮廓面积必须在 0.01–1000000 平方米内".into());
    }
    let triangles = polygon
        .earcut_triangles()
        .into_iter()
        .map(|triangle| {
            let [a, b, c] = triangle.to_array();
            let mut points = [[a.x, a.y], [b.x, b.y], [c.x, c.y]];
            if (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) < 0.0 {
                points.swap(1, 2);
            }
            points
        })
        .collect();
    Ok(FloorPlan {
        area_square_meters: area,
        triangles,
    })
}
