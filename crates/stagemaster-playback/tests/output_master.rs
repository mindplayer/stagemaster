use stagemaster_playback::OutputMaster;
#[test]
fn master_is_exact_bounded_and_blackout_preserves_level() {
    let mut full = OutputMaster::default();
    let mut half = full;
    half.set_percent(50).unwrap();
    let mut dark = half;
    dark.set_blackout(true);
    for raw in 0..=u16::MAX {
        assert_eq!(full.scale(raw), raw);
        assert_eq!(half.scale(raw), raw / 2 + raw % 2);
        assert_eq!(dark.scale(raw), 0);
    }
    assert_eq!(dark.percent(), 50);
    dark.set_percent(37).unwrap();
    assert_eq!(dark.scale(65535), 0);
    dark.set_blackout(false);
    assert_eq!(dark.scale(65535), 24248);
    assert!(dark.set_percent(101).is_err());
    assert_eq!(dark.percent(), 37);
    full.set_percent(0).unwrap();
    assert_eq!(full.scale(65535), 0);
}
