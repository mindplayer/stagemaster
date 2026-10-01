//! Continuous straight targets with a fixed solution family and winding.
//! Geometry only: no timing, motor limits, collision policy, quantization or I/O.
use crate::{
    Installation,
    positioning::{AxisRange, Branch, IntersectingHead, JointAngles},
};
use glam::DVec3;
mod bounds;
pub use bounds::DerivativeBounds;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Geometry(crate::positioning::Error),
    EmptyPath,
    NearAxis,
    Unreachable,
    InvalidProgress,
}

/// Immutable, allocation-free curve. Sampling never reselects a static solution.
#[derive(Clone, Copy, Debug)]
pub struct LineTrajectory {
    local_start: DVec3,
    local_delta: DVec3,
    initial: JointAngles,
    branch: Branch,
}

impl LineTrajectory {
    /// Select the least starting travel among families/windings that fit the WHOLE line.
    /// A preference restricts the family; it never permits an intermediate flip.
    /// # Errors
    /// Rejects invalid geometry, coincident ends, near-axis paths and full-path travel violations.
    pub fn new(
        head: IntersectingHead,
        installation: Installation,
        from: [f64; 3],
        to: [f64; 3],
        previous: JointAngles,
        preferred_branch: Option<Branch>,
    ) -> Result<Self, Error> {
        // Reuse authoritative model, installation and explicit previous-angle validation.
        head.ray(installation, previous).map_err(Error::Geometry)?;
        if from
            .into_iter()
            .chain(to)
            .any(|v| !v.is_finite() || v.abs() > 1_000_000.0)
        {
            return Err(Error::Geometry(crate::positioning::Error::InvalidTarget));
        }
        let rotation = installation.rotation().inverse();
        let start = rotation * (DVec3::from_array(from) - installation.position());
        let end = rotation * (DVec3::from_array(to) - installation.position());
        let delta = end - start;
        if delta.length() < 1e-6 {
            return Err(Error::EmptyPath);
        }
        let xy_delta = delta.truncate();
        let closest = if xy_delta.length_squared() > 0.0 {
            (-start.truncate().dot(xy_delta) / xy_delta.length_squared()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let clearance = (start + closest * delta).truncate().length();
        if clearance <= 1e-6 + start.length().max(end.length()) * 1e-6 {
            return Err(Error::NearAxis);
        }
        let pan = (-start.x).atan2(start.y).to_degrees();
        let tilt = tilt(start);
        let mut best: Option<(f64, Self)> = None;
        for branch in [Branch::Front, Branch::Back] {
            if preferred_branch.is_some_and(|wanted| wanted != branch) {
                continue;
            }
            let mut path = Self {
                local_start: start,
                local_delta: delta,
                initial: JointAngles {
                    pan_degrees: pan + if branch == Branch::Back { 180.0 } else { 0.0 }
                        - head.zero_correction.pan_degrees,
                    tilt_degrees: sign(branch) * tilt - head.zero_correction.tilt_degrees,
                },
                branch,
            };
            let end = path.at(1.0);
            let extreme = path.at(tilt_extremum(start, delta).unwrap_or(0.0));
            let Some(pan_shift) = winding(
                head.pan,
                path.initial.pan_degrees,
                end.pan_degrees,
                path.initial.pan_degrees,
                previous.pan_degrees,
            ) else {
                continue;
            };
            let Some(tilt_shift) = winding(
                head.tilt,
                path.initial
                    .tilt_degrees
                    .min(end.tilt_degrees)
                    .min(extreme.tilt_degrees),
                path.initial
                    .tilt_degrees
                    .max(end.tilt_degrees)
                    .max(extreme.tilt_degrees),
                path.initial.tilt_degrees,
                previous.tilt_degrees,
            ) else {
                continue;
            };
            path.initial.pan_degrees += pan_shift;
            path.initial.tilt_degrees += tilt_shift;
            let cost = (path.initial.pan_degrees - previous.pan_degrees)
                .hypot(path.initial.tilt_degrees - previous.tilt_degrees);
            if best.as_ref().is_none_or(|(old, _)| cost < *old) {
                best = Some((cost, path));
            }
        }
        best.map(|(_, path)| path).ok_or(Error::Unreachable)
    }

    #[must_use]
    pub fn branch(self) -> Branch {
        self.branch
    }

    /// # Errors
    /// Rejects non-finite progress or values outside the closed interval 0–1.
    pub fn sample(self, progress: f64) -> Result<JointAngles, Error> {
        if !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
            return Err(Error::InvalidProgress);
        }
        Ok(self.at(progress))
    }

    fn at(self, progress: f64) -> JointAngles {
        let point = self.local_start + self.local_delta * progress;
        let start_xy = self.local_start.truncate();
        let point_xy = point.truncate();
        // A line avoiding the origin subtends strictly less than 180 degrees.
        // Relative atan2 therefore remains continuous across the global +/-180 seam.
        let pan_delta = start_xy
            .perp_dot(point_xy)
            .atan2(start_xy.dot(point_xy))
            .to_degrees();
        JointAngles {
            pan_degrees: self.initial.pan_degrees + pan_delta,
            tilt_degrees: self.initial.tilt_degrees
                + sign(self.branch) * (tilt(point) - tilt(self.local_start)),
        }
    }
}

fn tilt(point: DVec3) -> f64 {
    point.x.hypot(point.y).atan2(-point.z).to_degrees()
}
fn sign(branch: Branch) -> f64 {
    if branch == Branch::Front { 1.0 } else { -1.0 }
}

fn winding(range: AxisRange, a: f64, b: f64, initial: f64, previous: f64) -> Option<f64> {
    let lower = ((range.min_degrees - a.min(b)) / 360.0).ceil();
    let upper = ((range.max_degrees - a.max(b)) / 360.0).floor();
    (lower <= upper).then(|| ((previous - initial) / 360.0).round().clamp(lower, upper) * 360.0)
}

fn tilt_extremum(start: DVec3, delta: DVec3) -> Option<f64> {
    // For beta=atan2(r,-z), its derivative's sign is that of q*z' - z*q'/2.
    // q=|xy|² is quadratic, z is linear; the quadratic terms cancel exactly.
    let q0 = start.truncate().length_squared();
    let q1 = start.truncate().dot(delta.truncate());
    let q2 = delta.truncate().length_squared();
    let constant = q0 * delta.z - start.z * q1;
    let slope = q1 * delta.z - start.z * q2;
    if slope.abs() > 0.0 {
        let progress = -constant / slope;
        (progress > 0.0 && progress < 1.0).then_some(progress)
    } else {
        None
    }
}
