use super::manager::{Manager, Status};
use stagemaster_execution_client::BatchAction;

impl Manager {
    pub async fn batch(
        &mut self,
        host: &str,
        revision: &str,
        sources: &[String],
        action: BatchAction,
    ) -> Result<Status, String> {
        if self.closing {
            return Err("后台正在关闭".into());
        }
        self.client
            .as_mut()
            .ok_or("请先连接后台")?
            .batch(host, revision, sources, action)
            .await?;
        Ok(self.status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::manager::tests::{Cleanup, binary, connected, document, runtime};
    use stagemaster_execution_client::{Action, Selection};
    #[test]
    fn desktop_bridge_controls_one_batch_and_reopened_manager_does_not_acquire() {
        runtime().block_on(async {
            let dir = tempfile::tempdir_in(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp"),
            )
            .unwrap();
            let doc = document();
            let view = doc.view();
            let root = dir.path().join("execution");
            let mut manager = Manager::new(root.clone(), binary());
            manager
                .prepare(
                    doc,
                    vec![
                        Selection::Scene {
                            id: view.scenes[0].id.clone(),
                        },
                        Selection::Sequence {
                            id: view.sequences[0].id.clone(),
                        },
                    ],
                    None,
                )
                .await
                .unwrap();
            let _cleanup = Cleanup(manager.take_test_child());
            connected(&mut manager).await;
            manager.acquire(false).await.unwrap();
            let initial = connected(&mut manager).await.runtime.unwrap();
            let host = initial.host_id;
            for source in &initial.catalog.sources[..2] {
                let latest = connected(&mut manager).await.runtime.unwrap();
                manager
                    .apply(
                        &host,
                        &latest.observation.snapshot.unwrap().state.revision,
                        &source.id,
                        Action::Start {
                            step: source.steps[0].id.clone(),
                        },
                    )
                    .await
                    .unwrap();
            }
            let latest = connected(&mut manager).await.runtime.unwrap();
            let ids: Vec<_> = latest.catalog.sources[..2]
                .iter()
                .map(|s| s.id.clone())
                .collect();
            manager
                .batch(
                    &host,
                    &latest.observation.snapshot.unwrap().state.revision,
                    &ids,
                    BatchAction::Pause {},
                )
                .await
                .unwrap();
            let paused = connected(&mut manager).await.runtime.unwrap();
            assert_eq!(paused.record.unwrap().outcome.unwrap().kind, "applied");
            assert!(
                paused.observation.snapshot.unwrap().state.sources[..2]
                    .iter()
                    .all(|s| s.status.as_deref() == Some("Paused"))
            );
            let mut reopened = Manager::new(root, binary());
            let readonly = connected(&mut reopened).await.runtime.unwrap();
            assert!(!readonly.controlling);
            assert!(
                reopened
                    .batch(
                        &host,
                        &readonly.observation.snapshot.unwrap().state.revision,
                        &ids,
                        BatchAction::Stop {}
                    )
                    .await
                    .is_err()
            );
            manager.shutdown(&host).await.unwrap();
            let end = std::time::Instant::now() + std::time::Duration::from_secs(8);
            while manager.poll().await.phase != "empty" {
                assert!(std::time::Instant::now() < end);
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }
        });
    }
    #[test]
    fn desktop_wire_only_accepts_the_explicit_batch_envelope_and_actions() {
        let valid = serde_json::json!({"kind":"batch","hostId":"host","revision":"1","sources":["source"],"action":{"kind":"pause"}});
        assert!(serde_json::from_value::<crate::execution::Request>(valid.clone()).is_ok());
        for action in [
            serde_json::json!({"kind":"start","step":"x"}),
            serde_json::json!({"kind":"stop","extra":true}),
        ] {
            let mut v = valid.clone();
            v["action"] = action;
            assert!(serde_json::from_value::<crate::execution::Request>(v).is_err());
        }
        let mut v = valid;
        v["layout"] = "fake".into();
        assert!(serde_json::from_value::<crate::execution::Request>(v).is_err());
    }
}
