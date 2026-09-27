//! Generic editable support geometry, not manufacturer/structural load models.
use crate::Triangle;
use stagemaster_project::{RigKind, RigShape};

pub(super) fn mesh(rig: &RigShape) -> Result<Vec<Triangle>, String> {
    let center = rig.position_meters.numbers(100_000.0)?;
    let length = crate::number(&rig.length_meters)?;
    let width = crate::number(&rig.width_meters)?;
    let height = crate::number(&rig.height_meters)?;
    let (sin, cos) = crate::number(&rig.yaw_degrees)?.to_radians().sin_cos();
    let mut triangles = Vec::new();
    match rig.rig_kind {
        RigKind::Pipe => {
            tube(
                &mut triangles,
                [-length / 2.0, 0.0, 0.0],
                [length / 2.0, 0.0, 0.0],
                width / 2.0,
            );
            for p in triangles.iter_mut().flatten() {
                p[2] *= height / width;
            }
        }
        RigKind::Truss => {
            let radius = (width.min(height) * 0.07).clamp(0.005, 0.03);
            // Dimensions describe the outside envelope, not tube centre lines.
            let span = length - radius * 1.4;
            let half_width = width / 2.0 - radius;
            let half_height = height / 2.0 - radius;
            let corners = [
                [-half_width, -half_height],
                [half_width, -half_height],
                [half_width, half_height],
                [-half_width, half_height],
            ];
            for c in corners {
                tube(
                    &mut triangles,
                    [-span / 2.0, c[0], c[1]],
                    [span / 2.0, c[0], c[1]],
                    radius,
                );
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let sections = length.ceil().clamp(1.0, 32.0) as u32;
            for i in 0..=sections {
                let x = -span / 2.0 + span * f64::from(i) / f64::from(sections);
                for j in 0..4 {
                    let a = corners[j];
                    let b = corners[(j + 1) % 4];
                    tube(
                        &mut triangles,
                        [x, a[0], a[1]],
                        [x, b[0], b[1]],
                        radius * 0.7,
                    );
                    if i < sections {
                        let next = x + span / f64::from(sections);
                        let (a, b) = if i % 2 == 0 { (a, b) } else { (b, a) };
                        tube(
                            &mut triangles,
                            [x, a[0], a[1]],
                            [next, b[0], b[1]],
                            radius * 0.55,
                        );
                    }
                }
            }
        }
    }
    for p in triangles.iter_mut().flatten() {
        let [x, y, z] = *p;
        *p = [
            center[0] + x * cos - y * sin,
            center[1] + x * sin + y * cos,
            center[2] + z,
        ];
    }
    Ok(triangles)
}
fn tube(mesh: &mut Vec<Triangle>, start: [f64; 3], end: [f64; 3], radius: f64) {
    let axis = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    let length = axis.iter().map(|x| x * x).sum::<f64>().sqrt();
    let direction = axis.map(|bitangent| bitangent / length);
    let helper = if direction[2].abs() > 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let tangent = cross(direction, helper);
    let scale = tangent.iter().map(|x| x * x).sum::<f64>().sqrt();
    let tangent = tangent.map(|bitangent| bitangent / scale);
    let bitangent = cross(direction, tangent);
    let ring = |p: [f64; 3], i: u32| {
        let (sin, cos) = (f64::from(i) * std::f64::consts::TAU / 8.0).sin_cos();
        std::array::from_fn(|k| p[k] + radius * (tangent[k] * cos + bitangent[k] * sin))
    };
    for i in 0..8 {
        let ai = ring(start, i);
        let aj = ring(start, (i + 1) % 8);
        let bi = ring(end, i);
        let bj = ring(end, (i + 1) % 8);
        mesh.extend([[ai, aj, bj], [ai, bj, bi], [start, aj, ai], [end, bi, bj]]);
    }
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
