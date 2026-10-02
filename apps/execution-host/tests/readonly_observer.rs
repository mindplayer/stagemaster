mod support;
use stagemaster_execution_client::Reader;
use stagemaster_project::Document;
use std::{fs, time::Duration};
use support::*;

#[test]
fn readonly_observer_uses_fixed_project_without_claiming_control_or_source_files() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let mut reader = Reader::open(&path).await.unwrap();
        let bytes = reader.project().await.unwrap();
        let expected = Document::decode(&h.original).unwrap().encode().unwrap();
        assert_eq!(bytes, expected);
        fs::remove_file(&h.project).unwrap();
        assert_eq!(reader.project().await.unwrap(), expected);
        let first = reader.sample().await.unwrap();
        assert!(h.state().await["owner"].is_null());
        let session = h.session().await;
        let acquired = h.acquire(&session, false).await;
        let started =
            support::group::start(&h, &session, 2, &acquired["state"]["revision"], 0).await;
        assert_eq!(started["kind"], "applied");
        tokio::time::sleep(Duration::from_millis(70)).await;
        let next = reader.sample().await.unwrap();
        assert_ne!(first.composition_version, next.composition_version);
        drop(reader);
        let mut reopened = Reader::open(&path).await.unwrap();
        assert_eq!(reopened.project().await.unwrap(), expected);
        reopened.sample().await.unwrap();
        let state = h.state().await;
        assert_eq!(state["owner"]["sessionId"], session);
        assert_eq!(state["sources"][0]["status"], "Running");
        assert_eq!(state["revision"], started["state"]["revision"]);
        fs::write(&h.project, &h.original).unwrap();
        h.close().await;
        assert!(reopened.sample().await.is_err());
    });
}
