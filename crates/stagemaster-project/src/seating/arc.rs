//! Same-count concentric rows, with exact chair footprint and aisle checks.
use super::{SeatingLayout, SeatingShape};
use std::f64::consts::PI;

pub(super) fn layout(
    shape: &SeatingShape,
    position: [f64; 3],
    yaw: f64,
    dimensions: [f64; 4],
    radius: f64,
) -> Result<SeatingLayout, String> {
    let [w, d, dx, dy] = dimensions;
    let step = dx / radius;
    let gap = if let Some(aisle) = &shape.aisle {
        let clear = crate::stage::decimal(&aisle.width_meters, 0.3, 10.0)?;
        let radial = 2.0 * radius - d;
        let diagonal = radial.hypot(w);
        if clear >= diagonal {
            return Err("半径不足以容纳通道，请增大前排半径".into());
        }
        (2.0 * (w.atan2(radial) + (clear / diagonal).asin()) - step).max(0.0)
    } else {
        0.0
    };
    let spread = f64::from(shape.columns - 1) * step + gap;
    if spread + 2.0 * (w / (2.0 * radius - d)).atan() > PI + 1e-9 {
        return Err("弧形座区超过半圆，请增加半径或减少每排座位".into());
    }
    let count = shape.seat_count()?;
    let mut centers = Vec::with_capacity(count);
    let mut angles = Vec::with_capacity(count);
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for row in 0..shape.rows {
        let r = radius + f64::from(row) * dy;
        for column in 0..shape.columns {
            let after_aisle = shape
                .aisle
                .as_ref()
                .is_some_and(|a| column >= a.after_column);
            let angle =
                -spread / 2.0 + f64::from(column) * step + if after_aisle { gap } else { 0.0 };
            let (sin, cos) = angle.sin_cos();
            let center = [r * sin, radius - r * cos];
            for x in [-w / 2.0, w / 2.0] {
                for y in [-d / 2.0, d / 2.0] {
                    let corner = [center[0] + x * cos - y * sin, center[1] + x * sin + y * cos];
                    for axis in 0..2 {
                        min[axis] = min[axis].min(corner[axis]);
                        max[axis] = max[axis].max(corner[axis]);
                    }
                }
            }
            centers.push(center);
            angles.push(angle);
        }
    }
    validate_footprints(&centers, &angles, w, d, usize::from(shape.columns))?;
    let middle = [min[0].midpoint(max[0]), min[1].midpoint(max[1])];
    let (sin, cos) = yaw.sin_cos();
    let world = |p: [f64; 2]| {
        let x = p[0] - middle[0];
        let y = p[1] - middle[1];
        [
            position[0] + x * cos - y * sin,
            position[1] + x * sin + y * cos,
        ]
    };
    let outline = [
        world(min),
        world([max[0], min[1]]),
        world(max),
        world([min[0], max[1]]),
    ];
    if outline.iter().flatten().any(|v| v.abs() > 100_000.0) || position[2] + 0.85 > 100_000.0 {
        return Err("座区边界超出场地范围".into());
    }
    Ok(SeatingLayout {
        centers: centers.into_iter().map(world).collect(),
        seat_yaws_radians: angles.into_iter().map(|a| a + yaw).collect(),
        focus: Some(world([0.0, radius])),
        outline,
        floor: position[2],
        yaw_radians: yaw,
        seat_width: w,
        seat_depth: d,
    })
}

fn validate_footprints(
    centers: &[[f64; 2]],
    angles: &[f64],
    w: f64,
    d: f64,
    columns: usize,
) -> Result<(), String> {
    let diagonal_squared = w * w + d * d;
    for i in 0..centers.len() {
        for j in 0..i {
            let delta = [centers[i][0] - centers[j][0], centers[i][1] - centers[j][1]];
            if delta[0] * delta[0] + delta[1] * delta[1] >= diagonal_squared {
                continue;
            }
            let separate = [angles[i], angles[j]].iter().any(|angle| {
                [*angle, *angle + PI / 2.0].iter().any(|axis| {
                    let (sin, cos) = axis.sin_cos();
                    let distance = (delta[0] * cos + delta[1] * sin).abs();
                    let extent = |a: f64| {
                        let (s, c) = (a - axis).sin_cos();
                        (w * c.abs()).midpoint(d * s.abs())
                    };
                    distance >= extent(angles[i]) + extent(angles[j]) - 1e-9
                })
            });
            if !separate {
                return Err(format!(
                    "第 {} 排第 {} 座与第 {} 排第 {} 座重叠，请增大间距或半径",
                    j / columns + 1,
                    j % columns + 1,
                    i / columns + 1,
                    i % columns + 1
                ));
            }
        }
    }
    Ok(())
}
