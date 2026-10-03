#![cfg(feature = "application")]
#[path = "../../stagemaster-device-session/tests/support/mod.rs"]
mod support;
use stagemaster_device_auth::application::{DevelopmentPermit, Permissions, Scope, Session};
use stagemaster_device_link::management::ApplicationReceipt;
use stagemaster_device_session::{Channel, Kind, PLAINTEXT_BYTES};
use stagemaster_install_worker::{Completion, Epoch, Reply, secure::Gateway};

fn access(permissions: Permissions) -> (Channel, Session) {
    let (client, server) = support::channels();
    let permit = DevelopmentPermit::scoped(
        support::context().device,
        support::key(3).public(),
        [9; 16],
        7,
        1000,
        permissions,
    )
    .unwrap();
    (
        client,
        Session::admit(server, permit, support::context(), 0).unwrap(),
    )
}

#[test]
fn observe_and_control_cannot_open_the_installation_worker() {
    for permissions in [
        Permissions::only(Scope::Observe),
        Permissions::only(Scope::Control),
        Permissions::only(Scope::Observe).with(Scope::Control),
    ] {
        let (_, session) = access(permissions);
        // No Open command exists on rejection, so no storage work can be queued.
        assert!(matches!(
            Gateway::open(session, Epoch::new(1).unwrap(), 0),
            Err(stagemaster_install_worker::secure::Error::Permission(
                stagemaster_device_auth::application::Error::Denied
            ))
        ));
    }
}

#[test]
fn explicit_combination_still_issues_only_the_v1_installation_receipt() {
    let permissions = Permissions::only(Scope::Installation)
        .with(Scope::Observe)
        .with(Scope::Control);
    let (mut client, session) = access(permissions);
    let (mut gateway, _) = Gateway::open(session, Epoch::new(1).unwrap(), 0).unwrap();
    assert!(gateway.outbound(0).unwrap().is_none());
    gateway
        .complete(
            Completion {
                epoch: Epoch::new(1).unwrap(),
                result: Ok(Reply::Opened),
            },
            0,
        )
        .unwrap();
    let mut plain = [0; PLAINTEXT_BYTES];
    let opened = client
        .open(gateway.outbound(0).unwrap().unwrap(), &mut plain, 0)
        .unwrap();
    assert_eq!(opened.kind, Kind::Message);
    assert_eq!(&opened.payload[88..92], &1_u32.to_le_bytes());
    ApplicationReceipt::decode(opened.payload).unwrap();
    gateway.sent(0).unwrap();
    assert!(gateway.live_epoch(0).is_some());
    assert_eq!(gateway.live_epoch(1000), None);
}
