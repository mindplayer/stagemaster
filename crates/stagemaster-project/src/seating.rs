//! Parametric audience sections. Derived seats never become document identities.
mod arc;
use crate::stage::{SpatialVector3, decimal};
use serde::{Deserialize, Serialize};

pub(crate) const CAPABILITY: &str = "stage.seating";
pub(crate) const ARC_CAPABILITY: &str = "stage.seating.arc";
pub const MAX_PROJECT_SEATS: usize = 1024;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeatingAisle {
    pub after_column: u16,
    pub width_meters: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeatingArc {
    pub radius_meters: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeatingShape {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc: Option<SeatingArc>,
    pub space_id: Option<String>,
    pub position_meters: SpatialVector3,
    pub yaw_degrees: String,
    pub rows: u16,
    pub columns: u16,
    pub seat_width_meters: String,
    pub seat_depth_meters: String,
    pub column_spacing_meters: String,
    pub row_spacing_meters: String,
    pub aisle: Option<SeatingAisle>,
}
/// Validated world geometry. Seat ordering is front-to-back, then local left-to-right.
pub struct SeatingLayout {
    pub centers: Vec<[f64; 2]>,
    pub seat_yaws_radians: Vec<f64>,
    pub focus: Option<[f64; 2]>,
    pub outline: [[f64; 2]; 4],
    pub floor: f64,
    pub yaw_radians: f64,
    pub seat_width: f64,
    pub seat_depth: f64,
}
impl SeatingShape {
    /// Count before generating geometry.
    /// # Errors
    /// Rejects invalid counts and the per-section 512-seat limit.
    pub fn seat_count(&self) -> Result<usize, String> {
        if !(1..=64).contains(&self.rows) || !(1..=64).contains(&self.columns) {
            return Err("座区排数和每排座位数必须在 1–64 之间".into());
        }
        let count = usize::from(self.rows) * usize::from(self.columns);
        if count > 512 {
            return Err("单个座区最多 512 个座位".into());
        }
        Ok(count)
    }
    /// Produce bounded geometry from immutable parameters, independently of any renderer.
    /// # Errors
    /// Rejects invalid dimensions, overlapping centers, aisle placement or world bounds.
    pub fn layout(&self) -> Result<SeatingLayout, String> {
        let count = self.seat_count()?;
        let p = self.position_meters.numbers(100_000.0)?;
        let yaw = decimal(&self.yaw_degrees, -3600.0, 3600.0)?.to_radians();
        let w = decimal(&self.seat_width_meters, 0.3, 1.2)?;
        let d = decimal(&self.seat_depth_meters, 0.3, 1.2)?;
        let dx = decimal(&self.column_spacing_meters, w, 5.0)?;
        let dy = decimal(&self.row_spacing_meters, d, 10.0)?;
        let extra = if let Some(aisle) = &self.aisle {
            if aisle.after_column == 0 || aisle.after_column >= self.columns {
                return Err("通道必须位于两列座位之间".into());
            }
            let width = decimal(&aisle.width_meters, 0.3, 10.0)?;
            if width + 1e-9 < dx - w {
                return Err("通道净宽不能小于原座椅列间净距".into());
            }
            (width - (dx - w)).max(0.0)
        } else {
            0.0
        };
        if let Some(arc) = &self.arc {
            let radius = decimal(&arc.radius_meters, 1.0, 10_000.0)?;
            return arc::layout(self, p, yaw, [w, d, dx, dy], radius);
        }
        let width = f64::from(self.columns - 1) * dx + w + extra;
        let depth = f64::from(self.rows - 1) * dy + d;
        let (sin, cos) = yaw.sin_cos();
        let world = |x: f64, y: f64| [p[0] + x * cos - y * sin, p[1] + x * sin + y * cos];
        let outline = [
            world(-width / 2.0, -depth / 2.0),
            world(width / 2.0, -depth / 2.0),
            world(width / 2.0, depth / 2.0),
            world(-width / 2.0, depth / 2.0),
        ];
        if outline.iter().flatten().any(|v| v.abs() > 100_000.0) || p[2] + 0.85 > 100_000.0 {
            return Err("座区边界超出场地范围".into());
        }
        let mut centers = Vec::with_capacity(count);
        for row in 0..self.rows {
            for column in 0..self.columns {
                let offset = if self
                    .aisle
                    .as_ref()
                    .is_some_and(|a| column >= a.after_column)
                {
                    extra
                } else {
                    0.0
                };
                centers.push(world(
                    -width / 2.0 + w / 2.0 + f64::from(column) * dx + offset,
                    depth / 2.0 - d / 2.0 - f64::from(row) * dy,
                ));
            }
        }
        Ok(SeatingLayout {
            centers,
            seat_yaws_radians: vec![yaw; count],
            focus: None,
            outline,
            floor: p[2],
            yaw_radians: yaw,
            seat_width: w,
            seat_depth: d,
        })
    }
}
