use super::{Error, IntersectingHead, JointAngles};
use crate::Installation;
use glam::DVec3;

/// Geometric comparison with a forward ray, not a measured fixture feedback result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReferenceCheck {
    pub distance_along_meters: f64,
    pub miss_meters: f64,
    pub angle_degrees: f64,
    pub closest_point_meters: [f64; 3],
}
impl IntersectingHead {
    /// Compare an authored target with the current physical model and setpoint.
    /// # Errors
    /// Rejects invalid models, installations, angles, unbounded targets or a target at the pivot.
    pub fn check_reference(
        self,
        installation: Installation,
        angles: JointAngles,
        target_meters: [f64; 3],
    ) -> Result<ReferenceCheck, Error> {
        let ray = self.ray(installation, angles)?;
        if target_meters
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1_000_000.0)
        {
            return Err(Error::InvalidTarget);
        }
        let origin = DVec3::from_array(ray.origin_meters);
        let delta = DVec3::from_array(target_meters) - origin;
        if delta.length() < 1e-6 {
            return Err(Error::TargetAtPivot);
        }
        let direction = DVec3::from_array(ray.direction).normalize();
        let distance_along_meters = delta.dot(direction);
        let closest = origin + direction * distance_along_meters.max(0.0);
        Ok(ReferenceCheck {
            distance_along_meters,
            miss_meters: (DVec3::from_array(target_meters) - closest).length(),
            angle_degrees: delta
                .cross(direction)
                .length()
                .atan2(distance_along_meters)
                .to_degrees(),
            closest_point_meters: closest.to_array(),
        })
    }
}
