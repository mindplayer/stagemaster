//! Boundary inputs pass the real encoder, complete archive admission, installer and Runtime.
#[path = "../../stagemaster-package/examples/capacity/corpus.rs"]
mod corpus;
#[path = "../../stagemaster-package/examples/capacity/programs.rs"]
mod programs;
use programs::{Shape, program};
use stagemaster_install::{Identity, Installer};
use stagemaster_install_store::FileStore;
use stagemaster_package::{Archive, MAX_LOADER_BYTES};
use stagemaster_playback::Player;
use stagemaster_runtime::*;

struct Permit;
impl PlaybackPolicy for Permit {
    fn authorize(&mut self, _: Permission) -> Result<(), Denial> {
        Ok(())
    }
}
fn apply<R: stagemaster_package::ReadAt>(
    runtime: &mut Runtime<R, Permit>,
    lease: Lease,
    action: Action,
    now: u64,
) {
    let request = Request {
        lease,
        serial: runtime.state().owner.unwrap().serial + 1,
        expected_revision: runtime.state().revision,
        action,
    };
    assert_eq!(runtime.submit(request, now).unwrap().result, Ok(()));
}
#[test]
fn admitted_boundaries_replay_complete_frames_and_reject_the_next_shape() {
    let mut plans = Vec::new();
    for shape in Shape::ALL {
        let (scale, plan) = corpus::frontier(shape, 4);
        assert!(matches!(
            corpus::archive(&[program(shape, scale + 1)], 4),
            Err(stagemaster_package::Error::Limit(_) | stagemaster_package::Error::AtProgram { .. })
        ));
        assert!(!shape.name().is_empty());
        plans.push(plan);
    }
    assert_eq!(plans[0].output.mappings.len(), 512);
    assert_eq!(plans[1].output.mappings.last().unwrap().fine, Some(512));
    assert_eq!(plans[2].plan.steps().len(), 128);
    assert_eq!(plans[3].plan.effects()[0].len(), 128);
    let (bytes, archive) = corpus::archive(&plans, 4).unwrap();
    assert!(archive.entries()[2].usage.loader_peak_bytes > 64_000);
    assert!(archive.entries()[3].usage.loader_peak_bytes > 64_000);
    replay(&bytes, &plans);
}
#[test]
fn full_catalogue_is_admitted_jointly_and_does_not_claim_all_maxima_at_once() {
    assert!(corpus::archive(&[program(Shape::Steps, 1)], 64).is_err());
    let (_, plan) = corpus::frontier(Shape::Keyframes, 64);
    let (bytes, archive) = corpus::archive(std::slice::from_ref(&plan), 64).unwrap();
    assert_eq!(archive.entries().len(), 64);
    assert!(archive.entries().iter().all(
        |e| e.usage.loader_peak_bytes > 64_000 && e.usage.loader_peak_bytes <= MAX_LOADER_BYTES
    ));
    replay(&bytes, &[plan]);
}
fn replay(bytes: &[u8], plans: &[stagemaster_package::Program]) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    std::fs::create_dir_all(&root).unwrap();
    let directory = tempfile::tempdir_in(root).unwrap();
    let mut installer = Installer::open(FileStore::open(directory.path()).unwrap(), [1; 16])
        .unwrap()
        .0;
    let archive = Archive::open(bytes).unwrap();
    let transaction = installer.transaction(1);
    installer
        .begin(transaction, Identity::from_archive(&archive))
        .unwrap();
    for (index, chunk) in bytes.chunks(1024).enumerate() {
        installer.write(transaction, index * 1024, chunk).unwrap();
    }
    installer.verify(transaction).unwrap();
    installer.commit(transaction).unwrap();
    let mut runtime = Runtime::new([2; 16], 0, MAX_LOADER_BYTES, Permit).unwrap();
    let maintenance = runtime
        .confirm_quiescent(runtime.quiescence_request().unwrap(), 0)
        .unwrap();
    runtime
        .finish_maintenance(maintenance, 0, || installer.snapshot().map(Some))
        .unwrap();
    let mut now = 0;
    for (index, entry) in archive.entries().iter().enumerate() {
        let expected = &plans[index % plans.len()];
        let lease = runtime
            .acquire(
                Grant {
                    principal: [3; 16],
                    origin: Origin::Panel,
                    duration_ms: 60_000,
                },
                false,
                now,
            )
            .unwrap();
        apply(
            &mut runtime,
            lease,
            Action::Select(ProgramKey {
                kind: entry.kind,
                id: entry.id,
            }),
            now,
        );
        apply(&mut runtime, lease, Action::Load, now);
        let step = runtime.steps()[0].id;
        apply(&mut runtime, lease, Action::Start { step }, now);
        let mut reference = Player::try_new(expected.plan.clone(), now).unwrap();
        reference.execute(0, now).unwrap();
        for _ in 0..80 {
            now += 25;
            runtime.tick(now).unwrap();
            reference.advance(now).unwrap();
            let mut actual = [0; 512];
            let mut wanted = [0; 512];
            assert!(runtime.render(&mut actual).unwrap().is_some());
            expected
                .output
                .render(reference.values(), &mut wanted)
                .unwrap();
            assert_eq!(actual, wanted, "program={index} time={now}");
        }
        apply(&mut runtime, lease, Action::Stop, now);
        runtime.release(lease, now).unwrap();
    }
}
