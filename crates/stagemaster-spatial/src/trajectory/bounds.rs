//! Conservative derivative envelopes for certified linear approximation.
use super::{Error, LineTrajectory};
use crate::positioning::JointAngles;
use glam::DVec3;

/// Absolute derivative bounds with respect to dimensionless line progress.
/// Bounds are in degrees/progress and degrees/progress², not motor speeds.
#[derive(Clone, Copy, Debug)]
pub struct DerivativeBounds {
    pub first: JointAngles,
    pub second: JointAngles,
}

impl LineTrajectory {
    /// Enclose derivatives throughout a closed interval, including interior extrema.
    /// Linear interpolation error in each axis is at most `second * span² / 8`.
    /// # Errors
    /// Rejects reversed, non-finite or out-of-range intervals.
    pub fn derivative_bounds(self, from: f64, to: f64) -> Result<DerivativeBounds, Error> {
        self.sample(from)?;
        self.sample(to)?;
        if from > to {
            return Err(Error::InvalidProgress);
        }
        let a = self.local_start + self.local_delta * from;
        let b = self.local_start + self.local_delta * to;
        let delta = self.local_delta;
        let r_min = distance_to_segment(a.with_z(0.0), b.with_z(0.0));
        let d_min = distance_to_segment(a, b);
        let r2 = r_min * r_min;
        let d2 = d_min * d_min;
        let cross = self.local_start.truncate().perp_dot(delta.truncate()).abs();
        let xy_dot_max = a
            .truncate()
            .dot(delta.truncate())
            .abs()
            .max(b.truncate().dot(delta.truncate()).abs());
        let dot_max = a.dot(delta).abs().max(b.dot(delta).abs());
        let z_max = a.z.abs().max(b.z.abs());
        let pan_first = cross / r2;
        let pan_second = 2.0 * cross * xy_dot_max / (r2 * r2);
        // q*z' - z*q'/2 is linear, so its absolute maximum occurs at an endpoint.
        let tilt_numerator = |p: DVec3| {
            (p.truncate().length_squared() * delta.z - p.z * p.truncate().dot(delta.truncate()))
                .abs()
        };
        let tilt_first = tilt_numerator(a).max(tilt_numerator(b)) / (r_min * d2);
        // beta'=(r*z' - z*r')/d²; r''=cross²/r³.
        let tilt_second =
            z_max * cross * cross / (r_min * r2 * d2) + 2.0 * tilt_first * dot_max / d2;
        Ok(DerivativeBounds {
            first: JointAngles {
                pan_degrees: pan_first.to_degrees(),
                tilt_degrees: tilt_first.to_degrees(),
            },
            second: JointAngles {
                pan_degrees: pan_second.to_degrees(),
                tilt_degrees: tilt_second.to_degrees(),
            },
        })
    }
}

fn distance_to_segment(a: DVec3, b: DVec3) -> f64 {
    let delta = b - a;
    let t = if delta.length_squared() > 0.0 {
        (-a.dot(delta) / delta.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (a + delta * t).length()
}
