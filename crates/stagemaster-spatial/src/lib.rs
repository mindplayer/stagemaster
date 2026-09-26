//! Renderer-independent spatial calculations. No DMX, clocks, I/O or UI ownership.
pub mod polygon;
pub mod positioning;
use glam::{DQuat, DVec3};

/// Right-handed meters and fixed-axis XYZ degrees, matching the project contract.
/// No scale is allowed on a physical fixture installation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Installation {
    pub position_meters: [f64; 3],
    pub rotation_degrees_xyz: [f64; 3],
}

impl Installation {
    /// # Errors
    /// Rejects non-finite or unbounded transforms before any matrix calculation.
    pub fn validate(self) -> Result<(), positioning::Error> {
        if self
            .position_meters
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1_000_000.0)
            || self
                .rotation_degrees_xyz
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 3600.0)
        {
            return Err(positioning::Error::InvalidInstallation);
        }
        Ok(())
    }

    pub(crate) fn rotation(self) -> DQuat {
        let [x, y, z] = self.rotation_degrees_xyz.map(f64::to_radians);
        // Explicit order avoids intrinsic/extrinsic Euler naming ambiguity.
        DQuat::from_rotation_z(z) * DQuat::from_rotation_y(y) * DQuat::from_rotation_x(x)
    }

    pub(crate) fn position(self) -> DVec3 {
        DVec3::from_array(self.position_meters)
    }
}
