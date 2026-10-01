use stagemaster_spatial::{
    Installation,
    positioning::{AxisRange, Error, IntersectingHead, JointAngles, ZeroCorrection},
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
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}
const AXES: JointAngles = JointAngles {
    pan_degrees: 0.0,
    tilt_degrees: 0.0,
};
#[test]
fn forward_offset_and_backward_collinear_targets_have_distinct_errors() {
    let direct = head()
        .check_reference(INSTALL, AXES, [0.0, 0.0, 0.0])
        .unwrap();
    close(direct.miss_meters, 0.0);
    close(direct.distance_along_meters, 5.0);
    close(direct.angle_degrees, 0.0);
    let offset = head()
        .check_reference(INSTALL, AXES, [3.0, 0.0, 1.0])
        .unwrap();
    close(offset.miss_meters, 3.0);
    close(offset.distance_along_meters, 4.0);
    close(offset.angle_degrees, 36.869_897_645_844_02);
    assert_eq!(
        offset.closest_point_meters.map(f64::to_bits),
        [0.0, 0.0, 1.0].map(f64::to_bits)
    );
    let behind = head()
        .check_reference(INSTALL, AXES, [0.0, 0.0, 10.0])
        .unwrap();
    close(behind.miss_meters, 5.0);
    close(behind.distance_along_meters, -5.0);
    close(behind.angle_degrees, 180.0);
    assert_eq!(
        behind.closest_point_meters.map(f64::to_bits),
        INSTALL.position_meters.map(f64::to_bits)
    );
}
#[test]
fn installation_and_zero_correction_share_the_forward_model() {
    let mut h = head();
    h.zero_correction.tilt_degrees = 90.0;
    let install = Installation {
        rotation_degrees_xyz: [0.0, 0.0, 90.0],
        ..INSTALL
    };
    let r = h.check_reference(install, AXES, [-4.0, 0.0, 5.0]).unwrap();
    close(r.miss_meters, 0.0);
    close(r.distance_along_meters, 4.0);
    let floor = Installation {
        rotation_degrees_xyz: [180.0, 0.0, 0.0],
        ..INSTALL
    };
    close(
        head()
            .check_reference(floor, AXES, [0.0, 0.0, 10.0])
            .unwrap()
            .miss_meters,
        0.0,
    );
}
#[test]
fn invalid_targets_and_setpoints_are_rejected() {
    assert_eq!(
        head().check_reference(INSTALL, AXES, INSTALL.position_meters),
        Err(Error::TargetAtPivot)
    );
    for x in [f64::NAN, f64::INFINITY, 1_000_001.0] {
        assert_eq!(
            head().check_reference(INSTALL, AXES, [x, 0.0, 0.0]),
            Err(Error::InvalidTarget)
        );
    }
    assert_eq!(
        head().check_reference(
            INSTALL,
            JointAngles {
                pan_degrees: 271.0,
                ..AXES
            },
            [0.0; 3]
        ),
        Err(Error::InvalidAngles)
    );
}
