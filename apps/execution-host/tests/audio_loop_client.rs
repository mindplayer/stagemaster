#![cfg(feature = "audio")]
mod support;
use serde_json::json;
use stagemaster_execution_client::{Client, MediaAction, Reader};
use support::{client_audio::*, *};

#[test]
fn shared_clients_observe_and_control_existing_loops_across_reconnects() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let path = h.directory.path().join("run/discovery.json");
        let mut client = Client::open(&path).await.unwrap();
        let mut reader = Reader::open(&path).await.unwrap();
        assert!(reader.catalog().audio.as_ref().unwrap().performance_loops);
        client.acquire(false).await.unwrap();
        let paused = apply(
            &mut client,
            MediaAction::Seek {
                position_ms: 2500,
                playing: false,
            },
        )
        .await;
        let native = paused.observation.snapshot.unwrap().state.audio.unwrap();
        let current = native.loop_state.unwrap();
        assert_eq!(current.pass, "1");
        assert_eq!(reader.sample().await.unwrap().slots[1], 40);
        let instance = native.instance.unwrap();
        apply(
            &mut client,
            MediaAction::ExitLoop {
                instance: instance.clone(),
                region: current.region,
                pass: current.pass.clone(),
                requested: true,
            },
        )
        .await;
        wait(&mut client, |v| {
            v.observation
                .snapshot
                .as_ref()
                .unwrap()
                .state
                .audio
                .as_ref()
                .unwrap()
                .loop_state
                .as_ref()
                .unwrap()
                .exit_requested
        })
        .await;
        client.release().await.unwrap();
        settled(&mut client).await;
        drop(client);
        let mut client = Client::open(&path).await.unwrap();
        assert!(!client.view().controlling);
        let actual = client
            .view()
            .observation
            .snapshot
            .unwrap()
            .state
            .audio
            .unwrap();
        assert_eq!(actual.instance.as_deref(), Some(instance.as_str()));
        assert!(actual.loop_state.unwrap().exit_requested);
        client.acquire(false).await.unwrap();
        apply(
            &mut client,
            MediaAction::ExitLoop {
                instance,
                region: current.region,
                pass: current.pass,
                requested: false,
            },
        )
        .await;
        apply(&mut client, MediaAction::Play {}).await;
        wait(&mut client, |v| {
            v.observation
                .snapshot
                .as_ref()
                .unwrap()
                .state
                .audio
                .as_ref()
                .unwrap()
                .loop_state
                .as_ref()
                .is_some_and(|r| r.pass == "2")
        })
        .await;
        reader.sample().await.unwrap();
        apply(&mut client, MediaAction::Stop {}).await;
        h.close().await;
    });
}
