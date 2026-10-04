use super::*;
use stagemaster_execution_client::Action;
use std::{fs, process::Child, time::Instant};
pub(in crate::execution) struct Cleanup(pub(in crate::execution) Child);
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
pub(in crate::execution) fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}
pub(in crate::execution) fn binary() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("stagemaster-execution-host")
}
pub(in crate::execution) fn document() -> Document {
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    value["entryPoints"] = serde_json::json!([]);
    value["lighting"]["sequences"][0]["repeat"] = "loop".into();
    Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap()
}

pub(in crate::execution) async fn connected(manager: &mut Manager) -> Status {
    let end = Instant::now() + Duration::from_secs(8);
    loop {
        let status = manager.poll().await;
        if status.problem.is_none() && status.runtime.as_ref().is_some_and(|v| !v.pending) {
            return status;
        }
        assert!(Instant::now() < end, "{:?}", status.problem);
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}
#[test]
fn desktop_manager_keeps_fixed_project_and_reopens_existing_background() {
    runtime().block_on(async {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
        let temp = tempfile::tempdir_in(root).unwrap();
        let data = temp.path().join("execution");
        let document = document();
        let selection = Selection::Sequence {
            id: document.view().sequences[0].id.clone(),
        };
        let mut manager = Manager::new(data.clone(), binary());
        manager
            .prepare(document.clone(), vec![selection.clone()], None)
            .await
            .unwrap();
        let _cleanup = Cleanup(manager.child.take().unwrap());
        let status = connected(&mut manager).await;
        let run = manager.run.clone().unwrap();
        assert!(!files::ended(&run));
        let host = status.runtime.unwrap().host_id;
        manager.acquire(false).await.unwrap();
        let view = connected(&mut manager).await.runtime.unwrap();
        let source = &view.catalog.sources[0];
        let revision = &view.observation.snapshot.unwrap().state.revision;
        manager
            .apply(
                &host,
                revision,
                &source.id,
                Action::Start {
                    step: source.steps[0].id.clone(),
                },
            )
            .await
            .unwrap();
        let playing = connected(&mut manager).await.runtime.unwrap();
        assert_eq!(
            playing.observation.snapshot.unwrap().state.sources[0]
                .status
                .as_deref(),
            Some("Running")
        );
        drop(manager);
        fs::remove_file(run.join("project.json")).unwrap();
        let mut restored = Manager::new(data.clone(), binary());
        let view = connected(&mut restored).await.runtime.unwrap();
        assert_eq!(view.host_id, host);
        assert!(!view.controlling);
        assert!(view.session_id.is_none());
        assert_eq!(
            view.observation.snapshot.unwrap().state.sources[0]
                .status
                .as_deref(),
            Some("Running")
        );
        assert!(
            restored
                .prepare(document, vec![selection], None)
                .await
                .is_err()
        );
        restored.shutdown(&host).await.unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while restored.poll().await.phase != "empty" {
            assert!(Instant::now() < end);
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
        assert!(files::ended(&run));
        assert!(!data.join("current").exists());
    });
}
#[test]
fn failed_spawn_and_bad_selection_do_not_leave_a_live_record() {
    runtime().block_on(async {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
        let temp = tempfile::tempdir_in(root).unwrap();
        let data = temp.path().join("execution");
        let mut manager = Manager::new(data.clone(), temp.path().join("missing"));
        assert!(manager.prepare(document(), vec![], None).await.is_err());
        assert!(!data.join("current").exists());
        let doc = document();
        let selected = Selection::Sequence {
            id: doc.view().sequences[0].id.clone(),
        };
        assert!(manager.prepare(doc, vec![selected], None).await.is_err());
        assert!(!data.join("current").exists());
    });
}
