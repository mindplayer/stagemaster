use stagemaster_spatial::{
    Installation,
    positioning::{
        AxisRange, Branch, Error as GeometryError, IntersectingHead, JointAngles, ZeroCorrection,
    },
    trajectory::{Error, LineTrajectory},
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
fn target(pan: f64) -> [f64; 3] {
    [
        -pan.to_radians().sin() * 5.0,
        pan.to_radians().cos() * 5.0,
        0.0,
    ]
}
fn assert_line(
    model: IntersectingHead,
    install: Installation,
    from: [f64; 3],
    to: [f64; 3],
    path: LineTrajectory,
) {
    let mut previous = path.sample(0.0).unwrap();
    for i in 0..=1000 {
        let progress = f64::from(i) / 1000.0;
        let value = path.sample(progress).unwrap();
        let ray = model.ray(install, value).unwrap();
        let delta: [f64; 3] = std::array::from_fn(|j| {
            from[j] + progress * (to[j] - from[j]) - install.position_meters[j]
        });
        let distance = delta.into_iter().map(|v| v * v).sum::<f64>().sqrt();
        for (actual, expected) in ray.direction.into_iter().zip(delta) {
            assert!((actual - expected / distance).abs() < 1e-10);
        }
        assert!((value.pan_degrees - previous.pan_degrees).abs() < 5.0);
        assert!((value.tilt_degrees - previous.tilt_degrees).abs() < 5.0);
        previous = value;
    }
}

#[test]
fn common_world_line_obeys_each_installation_zero_and_fixed_branch() {
    let from = [-2.0, 1.0, 0.5];
    let to = [2.0, 3.0, 1.5];
    for install in [
        Installation {
            position_meters: [-3.0, -4.0, 5.0],
            ..INSTALL
        },
        Installation {
            position_meters: [3.0, -4.0, 5.0],
            ..INSTALL
        },
        Installation {
            rotation_degrees_xyz: [85.0, 20.0, 110.0],
            ..INSTALL
        },
    ] {
        let model = IntersectingHead {
            pan: AxisRange {
                min_degrees: -1080.0,
                max_degrees: 1080.0,
            },
            tilt: AxisRange {
                min_degrees: -1080.0,
                max_degrees: 1080.0,
            },
            zero_correction: ZeroCorrection {
                pan_degrees: 17.25,
                tilt_degrees: -3.5,
            },
        };
        for branch in [Branch::Front, Branch::Back] {
            let path = LineTrajectory::new(
                model,
                install,
                from,
                to,
                angles(600.0, -650.0),
                Some(branch),
            )
            .unwrap();
            assert_eq!(path.branch(), branch);
            assert_line(model, install, from, to, path);
            // Order of sampling cannot affect the answer or introduce hidden solver history.
            let middle = path.sample(0.5).unwrap();
            path.sample(1.0).unwrap();
            path.sample(0.0).unwrap();
            assert_eq!(middle, path.sample(0.5).unwrap());
        }
    }
}

#[test]
fn rejects_interior_travel_violation_even_when_both_endpoints_are_reachable() {
    let model = IntersectingHead {
        tilt: AxisRange {
            min_degrees: 30.0,
            max_degrees: 50.0,
        },
        ..head()
    };
    let from = [-4.0, 1.0, 0.0];
    let to = [4.0, 1.0, 0.0];
    let previous = angles(0.0, 40.0);
    for point in [from, to] {
        model
            .solve(INSTALL, point, previous, Some(Branch::Front))
            .unwrap();
    }
    assert_eq!(
        LineTrajectory::new(model, INSTALL, from, to, previous, None).unwrap_err(),
        Error::Unreachable
    );
}

#[test]
fn selects_a_different_winding_to_fit_the_entire_path_without_wrapping() {
    let model = IntersectingHead {
        pan: AxisRange {
            min_degrees: -200.0,
            max_degrees: 180.0,
        },
        ..head()
    };
    let path = LineTrajectory::new(
        model,
        INSTALL,
        target(170.0),
        target(190.0),
        angles(170.0, 45.0),
        Some(Branch::Front),
    )
    .unwrap();
    assert!((path.sample(0.0).unwrap().pan_degrees + 190.0).abs() < 1e-10);
    assert!((path.sample(1.0).unwrap().pan_degrees + 170.0).abs() < 1e-10);
    assert_line(model, INSTALL, target(170.0), target(190.0), path);
}

#[test]
fn chooses_other_family_only_before_start_and_respects_explicit_preference() {
    let model = IntersectingHead {
        pan: AxisRange {
            min_degrees: -180.0,
            max_degrees: 180.0,
        },
        ..head()
    };
    let from = target(170.0);
    let to = target(190.0);
    let previous = angles(170.0, 45.0);
    let path = LineTrajectory::new(model, INSTALL, from, to, previous, None).unwrap();
    assert_eq!(path.branch(), Branch::Back);
    assert_line(model, INSTALL, from, to, path);
    assert_eq!(
        LineTrajectory::new(model, INSTALL, from, to, previous, Some(Branch::Front)).unwrap_err(),
        Error::Unreachable
    );
}

#[test]
fn rejects_crossing_endpoint_and_numerically_near_axis_paths() {
    for (from, to) in [
        ([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
        ([-1.0, 0.000_001, 0.0], [1.0, 0.000_001, 0.0]),
        ([0.000_001, 0.0, 0.0], [0.000_001, 0.0, 3.0]),
        (INSTALL.position_meters, [0.0, 1.0, 0.0]),
    ] {
        assert_eq!(
            LineTrajectory::new(head(), INSTALL, from, to, angles(0.0, 0.0), None).unwrap_err(),
            Error::NearAxis
        );
    }
}

#[test]
fn vertical_line_and_changing_height_have_continuous_pointing() {
    for (from, to) in [
        ([2.0, 1.0, -2.0], [2.0, 1.0, 7.0]),
        ([-4.0, 1.0, 4.0], [4.0, 2.0, -4.0]),
    ] {
        let path = LineTrajectory::new(
            head(),
            INSTALL,
            from,
            to,
            angles(0.0, 0.0),
            Some(Branch::Front),
        )
        .unwrap();
        assert_line(head(), INSTALL, from, to, path);
    }
}

#[test]
fn rejects_invalid_inputs_without_panics_or_clamping() {
    for value in [f64::NAN, f64::INFINITY, 1e10] {
        assert_eq!(
            LineTrajectory::new(
                head(),
                INSTALL,
                [value, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                angles(0.0, 0.0),
                None
            )
            .unwrap_err(),
            Error::Geometry(GeometryError::InvalidTarget)
        );
    }
    assert_eq!(
        LineTrajectory::new(
            head(),
            INSTALL,
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            angles(0.0, 0.0),
            None
        )
        .unwrap_err(),
        Error::EmptyPath
    );
    assert_eq!(
        LineTrajectory::new(
            head(),
            INSTALL,
            [1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            angles(500.0, 0.0),
            None
        )
        .unwrap_err(),
        Error::Geometry(GeometryError::InvalidAngles)
    );
    let path = LineTrajectory::new(
        head(),
        INSTALL,
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        angles(0.0, 0.0),
        None,
    )
    .unwrap();
    for progress in [-0.001, 1.001, f64::NAN, f64::INFINITY] {
        assert_eq!(path.sample(progress).unwrap_err(), Error::InvalidProgress);
    }
}
