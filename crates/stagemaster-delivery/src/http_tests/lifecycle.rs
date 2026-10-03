use super::runtime::*;
use super::{expected, fixture, http, https};
use crate::{Error, Package, from_file};
use stagemaster_install::Installer;
use stagemaster_install_store::FileStore;
use stagemaster_playback::Player;
use stagemaster_runtime::{Action, Code, Denial, Status};
use std::{future::pending, sync::atomic::AtomicBool};

#[tokio::test]
async fn two_origins_share_install_identity_offline_execution_and_permission_boundary() {
    let bytes = fixture::bytes(12000);
    let identity = expected(&bytes);
    let server = https::Server::start(https::response("200 OK", "", &bytes)).await;
    let remote = http()
        .download(&server.url, identity, pending())
        .await
        .unwrap();
    drop(server);
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("removable.smpkg");
    std::fs::write(&source, &bytes).unwrap();
    let local = from_file(&source, Some(identity), &AtomicBool::new(false)).unwrap();
    std::fs::remove_file(source).unwrap();
    for (first, second) in [(&remote, &local), (&local, &remote)] {
        let target = tempfile::tempdir().unwrap();
        let mut installer = Installer::open(FileStore::open(target.path()).unwrap(), [1; 16])
            .unwrap()
            .0;
        let committed = install(&mut installer, first, 1);
        assert_eq!(
            install(&mut installer, second, 2),
            committed,
            "same content cannot create a new generation"
        );
        let (mut runtime, lease) = device(&installer);
        assert!(runtime.state().instance.is_none());
        let step = runtime.steps()[0].id;
        assert_eq!(
            apply(&mut runtime, lease, Action::Start { step }, 0),
            Err(Code::Permission(Denial::Missing))
        );
        assert!(runtime.state().instance.is_none());
        assert_eq!(runtime.policy_mut().calls[0].package, identity);
        runtime.policy_mut().allowed = true;
        apply(&mut runtime, lease, Action::Start { step }, 0).unwrap();
        let original_instance = runtime.state().instance;
        let program = first.archive().load(first, 0).unwrap();
        let mut reference = Player::new(program.plan, 0);
        reference.execute(0, 0).unwrap();
        for now in (0..10000).step_by(25) {
            runtime.tick(now).unwrap();
            reference.advance(now).unwrap();
            let mut actual = [0; 512];
            let mut wanted = [0; 512];
            runtime.render(&mut actual).unwrap().unwrap();
            program
                .output
                .render(reference.values(), &mut wanted)
                .unwrap();
            assert_eq!(actual, wanted);
        }
        let before = runtime.state();
        let broken = https::Server::start(https::response("200 OK", "", &bytes[..100])).await;
        assert!(
            http()
                .download(&broken.url, identity, pending())
                .await
                .is_err()
        );
        let abandoned = https::Server::hanging().await;
        assert!(matches!(
            http().download(&abandoned.url, identity, async {}).await,
            Err(Error::Cancelled)
        ));
        assert_eq!(installer.head(), Some(committed));
        assert_eq!(runtime.state(), before);
        let next = Package::from_bytes(fixture::bytes(30000).into(), None).unwrap();
        let update = install(&mut installer, &next, 3);
        assert_ne!(update.identity, identity);
        assert_eq!(runtime.state().bound_package, Some(committed));
        assert_eq!(runtime.state().instance, original_instance);
        assert_eq!(runtime.state().status, Some(Status::Running));
        // A new install is a candidate only; explicit maintenance is required to bind it.
        apply(&mut runtime, lease, Action::Stop, 10000).unwrap();
        apply(&mut runtime, lease, Action::BeginMaintenance, 10000).unwrap();
        bind(&mut runtime, &installer, 10000);
        assert_eq!(runtime.state().bound_package, Some(update));
        assert!(runtime.state().instance.is_none());
        // Maintenance retains the same operator; do not implicitly take control again.
        load(&mut runtime, lease, 10000);
        let step = runtime.steps()[0].id;
        apply(&mut runtime, lease, Action::Start { step }, 10000).unwrap();
        assert_eq!(
            runtime.policy_mut().calls.last().unwrap().package,
            next.identity()
        );
        drop(runtime);
        drop(installer);
        let (reopened, _) =
            Installer::open(FileStore::open(target.path()).unwrap(), [4; 16]).unwrap();
        assert_eq!(reopened.head(), Some(update));
        assert_eq!(
            reopened.snapshot().unwrap().archive().digest(),
            &next.identity().digest
        );
    }
}
