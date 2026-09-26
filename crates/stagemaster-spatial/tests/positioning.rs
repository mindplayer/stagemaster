use stagemaster_spatial::{
    Installation,
    positioning::{AxisRange, Branch, Error, IntersectingHead, JointAngles, ZeroCorrection},
};

const INSTALL: Installation = Installation {
    position_meters: [0.0, 0.0, 5.0],
    rotation_degrees_xyz: [0.0; 3],
};
fn head() -> IntersectingHead {
    IntersectingHead {
        pan: AxisRange {
            min_degrees: -270.0,
            max_degrees: 270.0,
        },
        tilt: AxisRange {
            min_degrees: -135.0,
            max_degrees: 135.0,
        },
        zero_correction: ZeroCorrection::default(),
    }
}
fn angles(pan: f64, tilt: f64) -> JointAngles {
    JointAngles {
        pan_degrees: pan,
        tilt_degrees: tilt,
    }
}
fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-8, "{actual} != {expected}");
}

#[test]
fn known_axes_obey_project_right_handed_basis() {
    for (input, direction) in [
        (angles(0.0, 0.0), [0.0, 0.0, -1.0]),
        (angles(0.0, 90.0), [0.0, 1.0, 0.0]),
        (angles(90.0, 90.0), [-1.0, 0.0, 0.0]),
    ] {
        let ray = head().ray(INSTALL, input).unwrap();
        assert_eq!(
            ray.origin_meters.map(f64::to_bits),
            INSTALL.position_meters.map(f64::to_bits)
        );
        for (a, b) in ray.direction.into_iter().zip(direction) {
            close(a, b);
        }
    }
}

#[test]
fn wall_and_floor_mounting_change_world_ray_without_changing_channels() {
    let side = Installation {
        rotation_degrees_xyz: [90.0, 0.0, 90.0],
        ..INSTALL
    };
    let ray = head().ray(side, angles(0.0, 0.0)).unwrap();
    for (a, b) in ray.direction.into_iter().zip([-1.0, 0.0, 0.0]) {
        close(a, b);
    }
    let floor = Installation {
        rotation_degrees_xyz: [180.0, 0.0, 0.0],
        ..INSTALL
    };
    close(
        head().ray(floor, angles(0.0, 0.0)).unwrap().direction[2],
        1.0,
    );
}

#[test]
fn common_target_requires_different_angles_for_each_installation() {
    let target = [0.0, 0.0, 0.0];
    let left = Installation {
        position_meters: [-3.0, 0.0, 4.0],
        ..INSTALL
    };
    let right = Installation {
        position_meters: [3.0, 0.0, 4.0],
        ..INSTALL
    };
    let a = head()
        .solve(left, target, angles(-90.0, 30.0), Some(Branch::Front))
        .unwrap();
    let b = head()
        .solve(right, target, angles(90.0, 30.0), Some(Branch::Front))
        .unwrap();
    close(a.angles.pan_degrees, -90.0);
    close(b.angles.pan_degrees, 90.0);
    close(a.angles.tilt_degrees, 36.869_897_645_844_02);
    assert!(a.residual_meters < 1e-10 && b.residual_meters < 1e-10);
}

#[test]
fn front_and_back_solutions_hit_same_point() {
    let a = head()
        .solve(
            INSTALL,
            [3.0, 4.0, 0.0],
            angles(0.0, 0.0),
            Some(Branch::Front),
        )
        .unwrap();
    let b = head()
        .solve(
            INSTALL,
            [3.0, 4.0, 0.0],
            angles(0.0, 0.0),
            Some(Branch::Back),
        )
        .unwrap();
    close(a.angles.tilt_degrees, 45.0);
    close(b.angles.tilt_degrees, -45.0);
    close((a.angles.pan_degrees - b.angles.pan_degrees).abs(), 180.0);
    for (x, y) in head()
        .ray(INSTALL, a.angles)
        .unwrap()
        .direction
        .into_iter()
        .zip(head().ray(INSTALL, b.angles).unwrap().direction)
    {
        close(x, y);
    }
}

#[test]
fn does_not_wrap_540_degree_pan_or_choose_a_longer_flip() {
    let previous = angles(179.0, 45.0);
    let desired = angles(181.0, 45.0);
    let ray = head().ray(INSTALL, desired).unwrap();
    let target = std::array::from_fn(|i| ray.origin_meters[i] + ray.direction[i] * 10.0);
    let result = head().solve(INSTALL, target, previous, None).unwrap();
    close(result.angles.pan_degrees, 181.0);
    close(result.angles.tilt_degrees, 45.0);
}

#[test]
fn pole_retains_previous_pan_instead_of_spinning_to_zero() {
    let result = head()
        .solve(INSTALL, [0.0, 0.0, 0.0], angles(231.0, 10.0), None)
        .unwrap();
    assert!(result.singular);
    close(result.angles.pan_degrees, 231.0);
    close(result.angles.tilt_degrees, 0.0);
}

#[test]
fn calibration_is_applied_once_and_inverted_by_solver() {
    let corrected = IntersectingHead {
        zero_correction: ZeroCorrection {
            pan_degrees: 17.0,
            tilt_degrees: -4.0,
        },
        ..head()
    };
    let result = corrected
        .solve(
            INSTALL,
            [0.0, 5.0, 0.0],
            angles(0.0, 0.0),
            Some(Branch::Front),
        )
        .unwrap();
    close(result.angles.pan_degrees, -17.0);
    close(result.angles.tilt_degrees, 49.0);
    assert!(result.residual_meters < 1e-10);
}

#[test]
fn impossible_target_is_reported_not_clamped_to_a_wrong_point() {
    let restricted = IntersectingHead {
        tilt: AxisRange {
            min_degrees: -10.0,
            max_degrees: 10.0,
        },
        ..head()
    };
    assert_eq!(
        restricted.solve(INSTALL, [0.0, 10.0, 0.0], angles(0.0, 0.0), None),
        Err(Error::Unreachable)
    );
    assert_eq!(
        head().solve(INSTALL, INSTALL.position_meters, angles(0.0, 0.0), None),
        Err(Error::TargetAtPivot)
    );
}

#[test]
fn arbitrary_mounts_and_zero_corrections_round_trip_at_travel_limits() {
    for correction in [
        ZeroCorrection::default(),
        ZeroCorrection {
            pan_degrees: 17.5,
            tilt_degrees: -3.25,
        },
    ] {
        let model = IntersectingHead {
            zero_correction: correction,
            ..head()
        };
        for rotation in [[0.0; 3], [180.0, 0.0, 0.0], [80.0, 21.0, -173.0]] {
            let installation = Installation {
                rotation_degrees_xyz: rotation,
                ..INSTALL
            };
            for p in [-270.0, -180.0, -10.0, 0.0, 170.0, 270.0] {
                for t in [-135.0, -90.0, -5.0, 0.0, 55.0, 135.0] {
                    let expected = angles(p, t);
                    let ray = model.ray(installation, expected).unwrap();
                    let target =
                        std::array::from_fn(|i| ray.origin_meters[i] + ray.direction[i] * 10.0);
                    let solved = model.solve(installation, target, expected, None).unwrap();
                    close(solved.angles.pan_degrees, p);
                    close(solved.angles.tilt_degrees, t);
                    assert!(solved.residual_meters < 1e-8);
                }
            }
        }
    }
}

#[test]
fn rejects_nonfinite_and_unbounded_inputs_before_geometry() {
    for value in [f64::NAN, f64::INFINITY, 1e100] {
        assert_eq!(
            head().solve(INSTALL, [value, 0.0, 0.0], angles(0.0, 0.0), None),
            Err(Error::InvalidTarget)
        );
        assert_eq!(
            head().ray(
                Installation {
                    position_meters: [value, 0.0, 0.0],
                    ..INSTALL
                },
                angles(0.0, 0.0)
            ),
            Err(Error::InvalidInstallation)
        );
        assert_eq!(
            head().ray(INSTALL, angles(value, 0.0)),
            Err(Error::InvalidAngles)
        );
        assert_eq!(
            IntersectingHead {
                zero_correction: ZeroCorrection {
                    pan_degrees: value,
                    tilt_degrees: 0.0
                },
                ..head()
            }
            .validate(),
            Err(Error::InvalidModel)
        );
    }
    assert_eq!(
        IntersectingHead {
            pan: AxisRange {
                min_degrees: 1.0,
                max_degrees: 1.0
            },
            ..head()
        }
        .validate(),
        Err(Error::InvalidModel)
    );
}
