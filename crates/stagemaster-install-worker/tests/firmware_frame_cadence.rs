#[path = "../../../apps/esp32-player/src/installation/frame_cadence.rs"]
mod cadence;
use cadence::Cadence;

#[test]
fn recent_frames_are_reused_but_slow_commands_do_not_add_an_old_period() {
    let mut cadence = Cadence::default();
    assert!(cadence.before_command(0));
    cadence.sampled(0);
    assert!(!cadence.before_command(0));
    assert!(!cadence.before_command(9_999));
    assert!(cadence.before_command(10_000));
    // Observed 32.711 ms load at the end of a frame period failed at >50 ms.
    let load_started = 24_000;
    let load_done = load_started + 32_711;
    assert!(load_done > 50_000);
    assert!(cadence.before_command(load_started));
    cadence.sampled(load_started);
    assert!(load_done - load_started < 50_000);
    assert!(cadence.before_command(load_done));
    cadence.sampled(load_done);
    assert!(!cadence.before_command(load_done));
}

#[test]
fn command_checks_do_not_claim_a_sample_or_silently_accept_rollback() {
    let mut cadence = Cadence::default();
    cadence.sampled(20_000);
    assert!(cadence.before_command(30_000));
    assert!(cadence.before_command(30_000));
    assert!(cadence.before_command(19_999));
    cadence.sampled(u64::MAX - 1);
    assert!(!cadence.before_command(u64::MAX));
}
