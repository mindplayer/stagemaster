use stagemaster_spatial::polygon::floor_plan;
#[test]
fn concave_floor_preserves_area_and_positive_triangle_winding() {
    let mut ring = vec![
        [0.0, 0.0],
        [8.0, 0.0],
        [8.0, 3.0],
        [3.0, 3.0],
        [3.0, 7.0],
        [0.0, 7.0],
    ];
    for _ in 0..2 {
        let plan = floor_plan(&ring).unwrap();
        assert!((plan.area_square_meters - 36.0).abs() < 1e-10);
        assert_eq!(plan.triangles.len(), 4);
        let mut sum = 0.0;
        for [a, b, c] in plan.triangles {
            let area = ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) / 2.0;
            assert!(area > 0.0);
            sum += area;
        }
        assert!((sum - 36.0).abs() < 1e-10);
        ring.reverse();
    }
}
#[test]
fn rejects_crossing_repeated_degenerate_nonfinite_and_oversize_rings() {
    for points in [
        vec![[0.0, 0.0], [4.0, 4.0], [0.0, 4.0], [4.0, 0.0]],
        vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]],
        vec![[0.0, 0.0], [2.0, 0.0], [4.0, 0.0]],
        vec![[0.0, 0.0], [0.01, 0.0], [0.0, 0.01]],
        vec![[0.0, 0.0], [f64::NAN, 0.0], [0.0, 4.0]],
        vec![[0.0, 0.0], [f64::INFINITY, 0.0], [0.0, 4.0]],
        vec![[0.0, 0.0], [100_001.0, 0.0], [0.0, 4.0]],
        vec![[0.0, 0.0]; 129],
    ] {
        assert!(floor_plan(&points).is_err());
    }
}
