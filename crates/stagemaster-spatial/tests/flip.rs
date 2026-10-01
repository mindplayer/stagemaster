use stagemaster_spatial::{
    Installation,
    positioning::{AxisRange, Error, IntersectingHead, JointAngles, ZeroCorrection},
};
fn head() -> IntersectingHead {
    IntersectingHead {
        pan: AxisRange {
            min_degrees: -540.0,
            max_degrees: 540.0,
        },
        tilt: AxisRange {
            min_degrees: -180.0,
            max_degrees: 180.0,
        },
        zero_correction: ZeroCorrection {
            pan_degrees: 13.0,
            tilt_degrees: -7.0,
        },
    }
}
#[test]
fn opposite_branch_keeps_world_ray_under_mount_rotation_and_zero() {
    let model = head();
    for pan in [-500.0, -270.0, 0.0, 270.0, 500.0] {
        for tilt in [-120.0, -30.0, 40.0, 120.0] {
            let old = JointAngles {
                pan_degrees: pan,
                tilt_degrees: tilt,
            };
            let other = model.flip(old).unwrap();
            assert!((other.angles.pan_degrees - pan).abs() <= 180.0 + 1e-8);
            assert!((other.angles.pan_degrees - pan).abs() > 179.99);
            assert!((other.angles.tilt_degrees - (-tilt + 14.0)).abs() < 1e-8);
            for rotation in [[0.0, 0.0, 0.0], [180.0, 0.0, 45.0], [90.0, 17.0, -60.0]] {
                let install = Installation {
                    position_meters: [4.0, 2.0, 6.5],
                    rotation_degrees_xyz: rotation,
                };
                let a = model.ray(install, old).unwrap();
                let b = model.ray(install, other.angles).unwrap();
                for (x, y) in a.direction.into_iter().zip(b.direction) {
                    assert!((x - y).abs() < 1e-9);
                }
                assert!(!other.singular);
            }
            assert_ne!(model.flip(other.angles).unwrap().branch, other.branch);
        }
    }
}
#[test]
fn singular_and_unavailable_branches_never_produce_a_clamped_flip() {
    let mut model = head();
    assert_eq!(
        model.flip(JointAngles {
            pan_degrees: 0.0,
            tilt_degrees: 7.0
        }),
        Err(Error::SingularFlip)
    );
    model.tilt = AxisRange {
        min_degrees: 10.0,
        max_degrees: 90.0,
    };
    assert_eq!(
        model.flip(JointAngles {
            pan_degrees: 0.0,
            tilt_degrees: 40.0
        }),
        Err(Error::Unreachable)
    );
    assert_eq!(
        model.flip(JointAngles {
            pan_degrees: f64::NAN,
            tilt_degrees: 40.0
        }),
        Err(Error::InvalidAngles)
    );
    model.tilt = head().tilt;
    model.pan = AxisRange {
        min_degrees: -20.0,
        max_degrees: 20.0,
    };
    assert_eq!(
        model.flip(JointAngles {
            pan_degrees: 0.0,
            tilt_degrees: 40.0
        }),
        Err(Error::Unreachable)
    );
}
