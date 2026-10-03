#![cfg(feature = "application")]
#[path = "../../stagemaster-device-session/tests/support/mod.rs"]
mod support;
use stagemaster_device_auth::application::{DevelopmentPermit, Session};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Kind, PLAINTEXT_BYTES};

#[test]
fn published_cutoff_is_exact_and_queries_and_sends_never_extend_it() {
    let (mut client, server) = support::channels();
    let permit = DevelopmentPermit::installation(
        support::context().device,
        support::key(3).public(),
        [9; 16],
        1,
        10_000,
    )
    .unwrap();
    let mut access = Session::admit(server, permit, support::context(), 100).unwrap();
    assert_eq!(access.valid_until(100).unwrap(), 6000);
    assert_eq!(access.valid_until(4000).unwrap(), 6000);
    let mut cipher = [0; CIPHERTEXT_BYTES];
    access
        .seal(Kind::Message, b"status", &mut cipher, 4000)
        .unwrap();
    assert_eq!(access.valid_until(4000).unwrap(), 6000);
    let n = client
        .seal(Kind::Heartbeat, &[], &mut cipher, 5000)
        .unwrap();
    access
        .open(&cipher[..n], &mut [0; PLAINTEXT_BYTES], 5000)
        .unwrap();
    assert_eq!(access.valid_until(5000).unwrap(), 10_100);
    assert_eq!(access.valid_until(10_099).unwrap(), 10_100);
    assert!(access.valid_until(10_100).is_err());
    assert!(access.grant(10_100).is_err());
}

#[test]
fn receive_lease_expires_at_its_cutoff_even_before_the_application_permit() {
    let (_, server) = support::channels();
    let permit = DevelopmentPermit::installation(
        support::context().device,
        support::key(3).public(),
        [9; 16],
        1,
        10_000,
    )
    .unwrap();
    let mut access = Session::admit(server, permit, support::context(), 0).unwrap();
    assert_eq!(access.valid_until(5999).unwrap(), 6000);
    assert!(access.valid_until(6000).is_err());
    assert!(access.grant(6000).is_err());
}
