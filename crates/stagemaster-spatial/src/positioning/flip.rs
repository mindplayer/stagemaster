use super::{Branch, Error, IntersectingHead, JointAngles, Solution};
use crate::Installation;

impl IntersectingHead {
    /// Choose the nearest reachable opposite yoke branch for the same beam ray.
    /// The intervening motion does not preserve pointing and is not a safe path.
    /// # Errors
    /// Rejects invalid angles/models, axial singularities and unavailable branches.
    pub fn flip(self, previous: JointAngles) -> Result<Solution, Error> {
        // Branch equivalence is invariant under installation rotation/translation.
        let local = Installation {
            position_meters: [0.0; 3],
            rotation_degrees_xyz: [0.0; 3],
        };
        let target = self.ray(local, previous)?.direction;
        let current = self.solve(local, target, previous, None)?;
        if current.singular {
            return Err(Error::SingularFlip);
        }
        let opposite = match current.branch {
            Branch::Front => Branch::Back,
            Branch::Back => Branch::Front,
        };
        self.solve(local, target, previous, Some(opposite))
    }
}
