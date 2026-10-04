mod support;
use stagemaster_execution_client::{Action, Client};
use std::sync::atomic::Ordering;
use support::{
    client_observation::{BUSY, settled},
    client_proxy::install,
    *,
};

#[test]
fn refused_http_admission_keeps_the_same_unknown_serial_and_never_resends() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        client.acquire(false).await.unwrap();
        let view = settled(&mut client).await;
        let state = view.observation.snapshot.unwrap().state;
        let source = &view.catalog.sources[0].id;
        let reads_before = proxy.receipt_reads.load(Ordering::SeqCst);
        proxy.reject_commands.store(true, Ordering::SeqCst);
        let error = client.apply(&view.host_id, &state.revision, source, Action::Level { value: 1234 }).await.unwrap_err();
        assert_eq!(error, BUSY);
        let original = client.view().record.unwrap();
        assert_eq!(original.serial, "2");
        assert!(original.outcome.is_none());
        assert!(client.view().pending);
        proxy.reject_commands.store(false, Ordering::SeqCst);
        assert_eq!(client.refresh().await.unwrap_err(), "后台请求未成功（409），请核对连接与原回执");
        assert_eq!(client.view().record.unwrap().serial, original.serial);
        assert!(client.view().pending);
        assert!(client.release().await.is_err());
        assert!(client.apply(&view.host_id, &state.revision, source, Action::Level { value: 1234 }).await.is_err());
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 1);
        assert_eq!(proxy.receipt_reads.load(Ordering::SeqCst), reads_before + 1);
        let actual = h.state().await;
        assert_eq!(actual["revision"], state.revision);
        assert_eq!(actual["sources"][0]["level"], state.sources[0].level);
        eprintln!("EXEC-016 pre-admission: serial=2 pending, attempts=2 forwarded=1, receipt=409, no runtime change");
        task.abort();h.close().await;
    });
}

#[test]
fn a_real_stale_revision_is_rejected_without_mutation_or_a_new_media_intent() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        client.acquire(false).await.unwrap();
        let view = settled(&mut client).await;
        let old = view.observation.snapshot.unwrap().state.revision;
        let source = &view.catalog.sources[0].id;
        client.apply(&view.host_id, &old, source, Action::Level { value: 1234 }).await.unwrap();
        let applied = settled(&mut client).await;
        assert_eq!(applied.record.as_ref().unwrap().outcome.as_ref().unwrap().kind, "applied");
        let changed = applied.observation.snapshot.unwrap().state;
        assert_ne!(changed.revision, old);
        client.apply(&view.host_id, &old, source, Action::Level { value: 5678 }).await.unwrap();
        let rejected = settled(&mut client).await;
        let record = rejected.record.unwrap();
        assert_eq!(record.serial, "3");
        let outcome = record.outcome.unwrap();
        assert_eq!(outcome.kind, "rejected");
        assert_eq!(outcome.code.as_deref(), Some("Revision"));
        assert_eq!(outcome.state.as_ref().unwrap().revision, changed.revision);
        assert_eq!(outcome.state.unwrap().sources[0].level, 1234);
        let actual = h.state().await;
        assert_eq!(actual["revision"], changed.revision);
        assert_eq!(actual["sources"][0]["level"], 1234);
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 3);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 3);
        eprintln!("EXEC-016 final-refusal: serial=3 rejected/Revision, attempts=3 forwarded=3, authoritative level=1234");
        task.abort();h.close().await;
    });
}
