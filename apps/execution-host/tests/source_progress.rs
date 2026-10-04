mod support;
use serde_json::{Value, json};
use stagemaster_execution_client::{Action, Client, Phase, View};
use std::{
    fs,
    time::{Duration, Instant},
};
use support::*;
fn source(view: &View) -> &stagemaster_execution_client::SourceState {
    &view.observation.snapshot.as_ref().unwrap().state.sources[1]
}
async fn until(client: &mut Client, condition: impl Fn(&View) -> bool) -> View {
    let end = Instant::now() + Duration::from_secs(7);
    loop {
        let view = match client.refresh().await {
            Ok(view) => view,
            // Busy observation is explicitly transient; keep the same deadline and receipt.
            Err(error) if error == "后台请求未成功（503），请核对连接与原回执" =>
            {
                assert!(Instant::now() < end, "{error}");
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }
            Err(error) => panic!("{error}"),
        };
        if !view.pending && condition(&view) {
            return view;
        }
        assert!(Instant::now() < end, "state did not converge: {view:?}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
async fn apply(client: &mut Client, view: &View, action: Action) -> View {
    client
        .apply(
            &view.host_id,
            &view.observation.snapshot.as_ref().unwrap().state.revision,
            &view.catalog.sources[1].id,
            action,
        )
        .await
        .unwrap();
    until(client, |_| true).await
}
#[test]
fn real_process_projects_progress_through_client_receipts_pause_resume_and_next() {
    runtime().block_on(async {
        let h = Harness::prepared(|path| {
            let manifest = group::write(path);
            let mut raw: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            let sequence = &mut raw["lighting"]["sequences"][0];
            let mut hold = sequence["steps"][0].clone();
            hold["id"] = json!(group::id(77));
            hold["number"] = json!("2");
            let step = &mut sequence["steps"][0];
            step["delay"] = json!({"ticks":"1000","ticksPerSecond":"1000"});
            step["fade"] = json!({"ticks":"1000","ticksPerSecond":"1000"});
            step["advance"] =
                json!({"kind":"after","wait":{"ticks":"1000","ticksPerSecond":"1000"}});
            sequence["steps"].as_array_mut().unwrap().push(hold);
            sequence["repeat"] = json!("loop");
            fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
            Some(manifest)
        });
        let mut client = Client::open(&h.directory.path().join("run/discovery.json"))
            .await
            .unwrap();
        let view = client.view();
        assert!(
            view.catalog
                .capabilities
                .iter()
                .any(|c| c == "sourceProgress")
        );
        assert_eq!(source(&view).progress.as_ref().unwrap().phase, Phase::Idle);
        assert!(
            view.observation.snapshot.as_ref().unwrap().state.sources[2]
                .progress
                .is_none()
        );
        client.acquire(false).await.unwrap();
        let view = until(&mut client, |v| v.controlling).await;
        let _started = apply(
            &mut client,
            &view,
            Action::Start {
                step: view.catalog.sources[1].steps[0].id.clone(),
            },
        )
        .await;
        let view = until(&mut client, |v| {
            source(v).progress.as_ref().unwrap().phase == Phase::Delay
        })
        .await;
        let paused = apply(&mut client, &view, Action::Pause {}).await;
        let frozen = source(&paused).progress.clone();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let view = client.refresh().await.unwrap();
        assert_eq!(source(&view).progress, frozen);
        assert_eq!(source(&view).status.as_deref(), Some("Paused"));
        apply(&mut client, &view, Action::Resume {}).await;
        for phase in [Phase::Fade, Phase::Wait, Phase::Hold] {
            until(&mut client, |v| {
                source(v).progress.as_ref().unwrap().phase == phase
            })
            .await;
        }
        let view = client.refresh().await.unwrap();
        let p = source(&view).progress.as_ref().unwrap();
        assert!(p.next_wrap);
        assert_eq!(
            p.next_step.as_ref(),
            Some(&view.catalog.sources[1].steps[0].id)
        );
        assert_eq!(p.phase_duration_ms, None);
        let view = apply(&mut client, &view, Action::Next {}).await;
        assert_eq!(
            source(&view).step.as_ref(),
            Some(&view.catalog.sources[1].steps[0].id)
        );
        let stopped = apply(&mut client, &view, Action::Stop {}).await;
        assert_eq!(
            source(&stopped).progress.as_ref().unwrap().phase,
            Phase::Idle
        );
        assert_eq!(fs::read(&h.project).unwrap(), h.original);
    });
}
