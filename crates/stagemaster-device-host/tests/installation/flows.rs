use super::support::*;
use stagemaster_device_host::{Phase, ProblemCode as C};
use stagemaster_install::{Installer, Phase as InstallPhase};
use stagemaster_install_store::FileStore;
use stagemaster_package::Archive;
use stagemaster_project::{Document, PackageSelection};
use stagemaster_transfer::{Command, Outcome, Request as WireRequest, Upload};
use std::time::Duration;

fn package() -> Vec<u8> {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    let first = json["lighting"]["scenes"][0].clone();
    for n in 0..20 {
        let mut scene = first.clone();
        scene["id"] = format!("00000000-0000-4000-8000-{:012}", 100 + n).into();
        scene["name"] = format!("主机传输验收 {n}").into();
        json["lighting"]["scenes"]
            .as_array_mut()
            .unwrap()
            .push(scene);
    }
    let doc = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    let selected: Vec<_> = doc
        .view()
        .scenes
        .iter()
        .map(|s| PackageSelection::Scene { id: s.id.clone() })
        .collect();
    let bytes = doc.build_package(&selected).unwrap().bytes;
    assert!(bytes.len() > 2048);
    bytes
}
async fn advance(host: &Host, upload: &mut Upload<&[u8]>) -> Option<Command> {
    let frame = upload.outbound().unwrap()?.clone();
    let command = WireRequest::decode(frame.bytes()).unwrap().action.command();
    let response = host
        .exchange_installation(status(host).epoch, frame)
        .await
        .unwrap();
    upload.accept(response.bytes()).unwrap();
    Some(command)
}
async fn finish(host: &Host, upload: &mut Upload<&[u8]>) {
    for _ in 0..100 {
        if advance(host, upload).await.is_none() {
            return;
        }
    }
    panic!("安装未完成");
}

#[tokio::test(start_paused = true)]
async fn real_export_crosses_fragmented_service_and_reopens_identically() {
    let bytes = package();
    for payload in [1, 20, 244, 1280] {
        let (host, state) = setup(payload);
        let connected = connect(&host).await;
        assert_eq!(connected.phase, Phase::Connected);
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        upload
            .connect(
                host.installation_peer(connected.epoch)
                    .unwrap()
                    .unwrap()
                    .session,
            )
            .unwrap();
        finish(&host, &mut upload).await;
        let Some(Outcome::Installed(commit)) = upload.outcome() else {
            panic!("无已安装结果")
        };
        assert_eq!(commit.generation, 1);
        let directory = state.lock().unwrap().directory.clone();
        let path = directory.path();
        {
            let s = state.lock().unwrap();
            let snapshot = s.server.snapshot().unwrap();
            assert_eq!(snapshot.commit(), commit);
            assert_eq!(
                snapshot.archive().entries(),
                Archive::open(bytes.as_slice()).unwrap().entries()
            );
            let actual =
                std::fs::read(path.join(format!("slot-{}.smpkg", commit.slot.index()))).unwrap();
            assert_eq!(actual, bytes);
        }
        disconnect(&host).await;
        assert!(host.installation_peer(connected.epoch).is_err());
        let reconnected = connect(&host).await;
        assert_eq!(
            host.exchange_installation(connected.epoch, query(&host))
                .await
                .unwrap_err()
                .code,
            C::Stale
        );
        upload.disconnect();
        upload
            .connect(
                host.installation_peer(reconnected.epoch)
                    .unwrap()
                    .unwrap()
                    .session,
            )
            .unwrap();
        finish(&host, &mut upload).await;
        assert!(matches!(upload.outcome(),Some(Outcome::Installed(c)) if c==commit));
        host.shutdown().await.unwrap();
        drop(host);
        drop(state);
        let (reopened, report) = Installer::open(FileStore::open(path).unwrap(), [3; 16]).unwrap();
        assert_eq!(report.selected, Some(commit));
        assert_eq!(
            reopened.snapshot().unwrap().archive().entries(),
            Archive::open(bytes.as_slice()).unwrap().entries()
        );
    }
}

#[tokio::test(start_paused = true)]
async fn slow_fragments_interleave_heartbeat_and_preserve_upload_result() {
    let (host, state) = setup(244);
    connect(&host).await;
    state.lock().unwrap().fragment_delay = Duration::from_millis(300);
    let bytes = package();
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    upload
        .connect(
            host.installation_peer(status(&host).epoch)
                .unwrap()
                .unwrap()
                .session,
        )
        .unwrap();
    finish(&host, &mut upload).await;
    assert!(matches!(upload.outcome(), Some(Outcome::Installed(_))));
    {
        let s = state.lock().unwrap();
        assert!(s.beats.len() > 2);
        assert!(
            s.beats
                .windows(2)
                .all(|pair| pair[1] - pair[0] <= Duration::from_millis(1801))
        );
    }
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn cancelled_upload_cleans_staging_after_partial_transfer() {
    let (host, _state) = setup(20);
    connect(&host).await;
    let bytes = package();
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    upload
        .connect(
            host.installation_peer(status(&host).epoch)
                .unwrap()
                .unwrap()
                .session,
        )
        .unwrap();
    while advance(&host, &mut upload).await != Some(Command::Write) {}
    upload.request_cancel();
    finish(&host, &mut upload).await;
    assert_eq!(upload.outcome(), Some(Outcome::Cancelled));
    assert_eq!(
        upload.state().unwrap().progress.unwrap().phase,
        InstallPhase::Cancelled
    );
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn lost_commit_response_then_cancel_reconnect_reports_installed() {
    let (host, state) = setup(20);
    connect(&host).await;
    let bytes = package();
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    upload
        .connect(
            host.installation_peer(status(&host).epoch)
                .unwrap()
                .unwrap()
                .session,
        )
        .unwrap();
    loop {
        let frame = upload.outbound().unwrap().unwrap().clone();
        if WireRequest::decode(frame.bytes()).unwrap().action.command() == Command::Commit {
            state.lock().unwrap().fault = Fault::DropCommit;
            assert_eq!(
                host.exchange_installation(status(&host).epoch, frame)
                    .await
                    .unwrap_err()
                    .code,
                C::Timeout
            );
            break;
        }
        advance(&host, &mut upload).await;
    }
    settle().await;
    assert_eq!(status(&host).phase, Phase::Fault);
    assert!(state.lock().unwrap().beats.len() >= 10);
    upload.disconnect();
    upload.request_cancel();
    state.lock().unwrap().fault = Fault::None;
    connect(&host).await;
    upload
        .connect(
            host.installation_peer(status(&host).epoch)
                .unwrap()
                .unwrap()
                .session,
        )
        .unwrap();
    finish(&host, &mut upload).await;
    assert!(matches!(upload.outcome(),Some(Outcome::Installed(c)) if c.generation==1));
    host.shutdown().await.unwrap();
}
