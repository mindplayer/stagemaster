#![cfg(feature = "audio")]
mod support;
use stagemaster_execution_client::Client;
use std::sync::atomic::Ordering;
use support::{client_observation::confirmed_control, *};

#[test]
fn accepted_control_with_busy_observation_resolves_original_receipt_without_replay() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(audio::write(p)));
        let discovery = h.directory.path().join("proxy.json");
        let (proxy, task) = client_proxy::install(&h, &discovery).await;
        private_discovery(&discovery);
        let mut client = Client::open(&discovery).await.unwrap();
        proxy.reject_observations.store(true, Ordering::SeqCst);
        let submitted = client.acquire(false).await;
        assert_eq!(submitted.as_ref().unwrap_err(), client_observation::BUSY);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 1);
        proxy.reject_observations.store(false, Ordering::SeqCst);
        let acquired = confirmed_control(&mut client, submitted, "acquired").await;
        assert!(acquired.controlling);
        assert_eq!(acquired.record.as_ref().unwrap().serial, "1");
        assert_eq!(
            h.state().await["owner"]["sessionId"].as_str(),
            acquired.session_id.as_deref()
        );
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 1);
        proxy.reject_observations.store(true, Ordering::SeqCst);
        let submitted = client.release().await;
        assert_eq!(submitted.as_ref().unwrap_err(), client_observation::BUSY);
        proxy.reject_observations.store(false, Ordering::SeqCst);
        let released = confirmed_control(&mut client, submitted, "released").await;
        assert!(!released.controlling);
        assert_eq!(released.record.as_ref().unwrap().serial, "2");
        assert!(h.state().await["owner"].is_null());
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 2);
        task.abort();
        h.close().await;
    });
}

#[test]
fn refused_admission_cannot_be_passed_as_confirmed_control_or_replayed() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(audio::write(p)));
        let discovery = h.directory.path().join("proxy.json");
        let (proxy, task) = client_proxy::install(&h, &discovery).await;
        private_discovery(&discovery);
        let mut client = Client::open(&discovery).await.unwrap();
        proxy.reject_commands.store(true, Ordering::SeqCst);
        let submitted = client.acquire(false).await;
        assert_eq!(submitted.as_ref().unwrap_err(), client_observation::BUSY);
        let result = tokio::spawn(async move {
            confirmed_control(&mut client, submitted, "acquired").await;
        })
        .await;
        assert!(result.unwrap_err().is_panic());
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 1);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 0);
        assert!(h.state().await["owner"].is_null());
        task.abort();
        h.close().await;
    });
}

fn private_discovery(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            path.parent().unwrap(),
            std::fs::Permissions::from_mode(0o700),
        )
        .unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}
