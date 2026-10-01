use stagemaster_spatial::{
    Installation,
    positioning::{AxisRange, Branch, IntersectingHead, JointAngles, ZeroCorrection},
    trajectory::{Error, LineTrajectory},
};

fn line(from: [f64; 3], to: [f64; 3], branch: Branch) -> LineTrajectory {
    let model = IntersectingHead {
        pan: AxisRange {
            min_degrees: -720.0,
            max_degrees: 720.0,
        },
        tilt: AxisRange {
            min_degrees: -720.0,
            max_degrees: 720.0,
        },
        zero_correction: ZeroCorrection {
            pan_degrees: 13.0,
            tilt_degrees: -4.0,
        },
    };
    LineTrajectory::new(
        model,
        Installation {
            position_meters: [0.0; 3],
            rotation_degrees_xyz: [0.0; 3],
        },
        from,
        to,
        JointAngles {
            pan_degrees: 360.0,
            tilt_degrees: -360.0,
        },
        Some(branch),
    )
    .unwrap()
}
fn array(angles: JointAngles) -> [f64; 2] {
    [angles.pan_degrees, angles.tilt_degrees]
}

#[test]
fn analytic_envelope_bounds_dense_linear_interpolation_and_local_rate() {
    // Vary elevation, length and distance to the pan axis, keeping y>0 to avoid poles.
    for i in 1..=40 {
        let n = f64::from(i);
        let from = [-n / 4.0, 0.1 + n / 20.0, -5.0 + n / 10.0];
        let to = [n / 3.0, 1.0 + n / 30.0, -4.0 - n / 20.0];
        for branch in [Branch::Front, Branch::Back] {
            let path = line(from, to, branch);
            for part in 0..16 {
                let start = f64::from(part) / 16.0;
                let end = start + 1.0 / 16.0;
                let bounds = path.derivative_bounds(start, end).unwrap();
                let a = array(path.sample(start).unwrap());
                let b = array(path.sample(end).unwrap());
                let mut previous = a;
                for j in 1..=64 {
                    let fraction = f64::from(j) / 64.0;
                    let actual = array(path.sample(start + (end - start) * fraction).unwrap());
                    for axis in 0..2 {
                        let interpolation = a[axis] + (b[axis] - a[axis]) * fraction;
                        let envelope = array(bounds.second)[axis] * (end - start).powi(2) / 8.0;
                        assert!((actual[axis] - interpolation).abs() <= envelope + 1e-10);
                        assert!(
                            (actual[axis] - previous[axis]).abs()
                                <= array(bounds.first)[axis] * (end - start) / 64.0 + 1e-10
                        );
                    }
                    previous = actual;
                }
            }
        }
    }
}

#[test]
fn stationary_direction_has_zero_bounds_and_interval_validation_is_strict() {
    let path = line([0.0, 1.0, -2.0], [0.0, 2.0, -4.0], Branch::Front);
    let bounds = path.derivative_bounds(0.0, 1.0).unwrap();
    assert!(array(bounds.first).into_iter().all(|v| v.abs() < 1e-12));
    assert!(array(bounds.second).into_iter().all(|v| v.abs() < 1e-12));
    for (start, end) in [(0.9, 0.1), (-0.1, 0.5), (0.0, 1.1), (0.0, f64::NAN)] {
        assert_eq!(
            path.derivative_bounds(start, end).unwrap_err(),
            Error::InvalidProgress
        );
    }
}
