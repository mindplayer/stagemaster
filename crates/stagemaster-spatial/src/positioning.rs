//! Static geometry for an explicitly restricted, intersecting orthogonal two-axis head.
//! It is not a general GDTF solver, path planner, collision checker or device controller.
use crate::Installation;
use glam::{DQuat, DVec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisRange {
    pub min_degrees: f64,
    pub max_degrees: f64,
}

impl AxisRange {
    fn valid(self) -> bool {
        self.min_degrees.is_finite()
            && self.max_degrees.is_finite()
            && self.min_degrees >= -3600.0
            && self.max_degrees <= 3600.0
            && self.min_degrees < self.max_degrees
    }

    fn contains(self, value: f64) -> bool {
        value.is_finite() && value >= self.min_degrees && value <= self.max_degrees
    }

    fn numerical_boundary(self, value: f64) -> Option<f64> {
        // Only repair floating-point roundoff at a mechanical boundary. This is
        // 1e-10 degree, not a policy to clamp unreachable targets into travel.
        (value >= self.min_degrees - 1e-10 && value <= self.max_degrees + 1e-10)
            .then(|| value.clamp(self.min_degrees, self.max_degrees))
    }
}

/// Unwrapped physical angles before instance zero corrections; not DMX percentages.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointAngles {
    pub pan_degrees: f64,
    pub tilt_degrees: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZeroCorrection {
    pub pan_degrees: f64,
    pub tilt_degrees: f64,
}

/// At corrected angles (0, 0), the beam points along local -Z.
/// Pan rotates around local +Z; tilt rotates around pan's local +X.
/// The installation origin is the common pivot AND emission origin.
/// Profiles with offset axes/emission origins must use a different solver.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntersectingHead {
    pub pan: AxisRange,
    pub tilt: AxisRange,
    pub zero_correction: ZeroCorrection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInstallation,
    InvalidModel,
    InvalidAngles,
    InvalidTarget,
    TargetAtPivot,
    Unreachable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin_meters: [f64; 3],
    pub direction: [f64; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Solution {
    pub angles: JointAngles,
    pub branch: Branch,
    /// At a pole, pan is indeterminate and is held at the supplied previous angle.
    pub singular: bool,
    /// Distance to the target ray, measured perpendicular to the emitted ray.
    pub residual_meters: f64,
}

/// World-space joint bases; each pair is local +X and +Z. Renderer independent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointPose {
    pub base: [[f64; 3]; 2],
    pub pan: [[f64; 3]; 2],
    pub head: [[f64; 3]; 2],
    pub direction: [f64; 3],
}
impl IntersectingHead {
    /// # Errors
    /// Rejects invalid installations, models and out-of-travel setpoints.
    pub fn pose(self, installation: Installation, angles: JointAngles) -> Result<JointPose, Error> {
        self.validate()?;
        installation.validate()?;
        self.validate_angles(angles)?;
        let base = installation.rotation();
        let pan = base
            * DQuat::from_rotation_z(
                (angles.pan_degrees + self.zero_correction.pan_degrees).to_radians(),
            );
        let head = pan
            * DQuat::from_rotation_x(
                (angles.tilt_degrees + self.zero_correction.tilt_degrees).to_radians(),
            );
        let basis = |q: DQuat| [(q * DVec3::X).to_array(), (q * DVec3::Z).to_array()];
        Ok(JointPose {
            base: basis(base),
            pan: basis(pan),
            head: basis(head),
            direction: (head * DVec3::NEG_Z).to_array(),
        })
    }

    /// # Errors
    /// Rejects unsupported/unbounded ranges or corrections. No implicit home values.
    pub fn validate(self) -> Result<(), Error> {
        if !self.pan.valid()
            || !self.tilt.valid()
            || [
                self.zero_correction.pan_degrees,
                self.zero_correction.tilt_degrees,
            ]
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 360.0)
        {
            return Err(Error::InvalidModel);
        }
        Ok(())
    }

    fn validate_angles(self, angles: JointAngles) -> Result<(), Error> {
        if !self.pan.contains(angles.pan_degrees) || !self.tilt.contains(angles.tilt_degrees) {
            return Err(Error::InvalidAngles);
        }
        Ok(())
    }

    /// Forward kinematics, shared by visualization and inverse-solution validation.
    /// # Errors
    /// Rejects invalid models, installations and out-of-range angles.
    pub fn ray(self, installation: Installation, angles: JointAngles) -> Result<Ray, Error> {
        self.validate()?;
        installation.validate()?;
        self.validate_angles(angles)?;
        let pan = (angles.pan_degrees + self.zero_correction.pan_degrees).to_radians();
        let tilt = (angles.tilt_degrees + self.zero_correction.tilt_degrees).to_radians();
        let direction = installation.rotation()
            * DQuat::from_rotation_z(pan)
            * DQuat::from_rotation_x(tilt)
            * DVec3::NEG_Z;
        Ok(Ray {
            origin_meters: installation.position_meters,
            direction: direction.to_array(),
        })
    }

    /// Solve a static world-space target, choosing least angular travel from `previous`.
    /// `branch` optionally restricts the solution family; no silently clamped targets.
    /// A valid `previous` is required even at the first sample (explicit caller policy).
    /// This does NOT guarantee a safe/continuous path to the returned angles.
    /// # Errors
    /// Rejects invalid input, coincident targets and targets outside available travel.
    pub fn solve(
        self,
        installation: Installation,
        target_meters: [f64; 3],
        previous: JointAngles,
        branch: Option<Branch>,
    ) -> Result<Solution, Error> {
        self.validate()?;
        installation.validate()?;
        self.validate_angles(previous)?;
        if target_meters
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1_000_000.0)
        {
            return Err(Error::InvalidTarget);
        }
        let delta = DVec3::from_array(target_meters) - installation.position();
        let distance = delta.length();
        if distance < 1e-6 {
            return Err(Error::TargetAtPivot);
        }
        let local = installation.rotation().inverse() * (delta / distance);
        let singular = local.x.hypot(local.y) < 1e-12;
        let pan = if singular {
            previous.pan_degrees + self.zero_correction.pan_degrees
        } else {
            (-local.x).atan2(local.y).to_degrees()
        };
        let tilt = local.x.hypot(local.y).atan2(-local.z).to_degrees();
        let mut best: Option<(f64, Solution)> = None;
        for (family, p, t) in [
            (Branch::Front, pan, tilt),
            (
                Branch::Back,
                if singular { pan } else { pan + 180.0 },
                -tilt,
            ),
        ] {
            if branch.is_some_and(|wanted| wanted != family) {
                continue;
            }
            // Validated bounds guarantee a small, fixed search; no target-sized allocation.
            for pan_turn in -21..=21 {
                let Some(candidate_pan) = self.pan.numerical_boundary(
                    p - self.zero_correction.pan_degrees + 360.0 * f64::from(pan_turn),
                ) else {
                    continue;
                };
                // At a pole, changing pan cannot improve the pointing and needlessly moves a motor.
                if singular && (candidate_pan - previous.pan_degrees).abs() > 1e-9 {
                    continue;
                }
                for tilt_turn in -11..=11 {
                    let Some(candidate_tilt) = self.tilt.numerical_boundary(
                        t - self.zero_correction.tilt_degrees + 360.0 * f64::from(tilt_turn),
                    ) else {
                        continue;
                    };
                    let angles = JointAngles {
                        pan_degrees: candidate_pan,
                        tilt_degrees: candidate_tilt,
                    };
                    let direction = DVec3::from_array(self.ray(installation, angles)?.direction);
                    let residual = delta.cross(direction).length();
                    if delta.dot(direction) <= 0.0 || residual > distance * 1e-9 {
                        continue;
                    }
                    let travel = (angles.pan_degrees - previous.pan_degrees)
                        .hypot(angles.tilt_degrees - previous.tilt_degrees);
                    if best.as_ref().is_none_or(|(cost, _)| travel < *cost) {
                        best = Some((
                            travel,
                            Solution {
                                angles,
                                branch: family,
                                singular,
                                residual_meters: residual,
                            },
                        ));
                    }
                }
            }
        }
        best.map(|(_, solution)| solution).ok_or(Error::Unreachable)
    }
}
