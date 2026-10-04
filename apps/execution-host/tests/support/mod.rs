#![allow(dead_code)] // Shared by independently compiled integration-test executables.
#[cfg(feature = "audio")]
pub mod audio;
#[cfg(feature = "audio")]
pub mod client_audio;
pub mod client_observation;
pub mod client_proxy;
pub mod group;
#[cfg(feature = "audio")]
pub mod loops;
pub mod rejection;
use reqwest::{Client, RequestBuilder};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub const SEQUENCE: &str = "00000000-0000-4000-8000-000000000040";
pub fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}
pub fn client() -> Client {
    Client::builder()
        .no_proxy()
        .pool_max_idle_per_host(0)
        .timeout(Duration::from_secs(7))
        .build()
        .unwrap()
}
pub fn temporary() -> tempfile::TempDir {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}
pub fn project(path: &Path) {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    value["entryPoints"] = json!([]);
    value["lighting"]["sequences"][0]["repeat"] = "loop".into();
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}
pub struct Process(pub Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
pub struct Harness {
    pub process: Process,
    pub directory: tempfile::TempDir,
    pub discovery: Value,
    pub http: Client,
    pub project: PathBuf,
    pub original: Vec<u8>,
}
impl Harness {
    pub fn start() -> Self {
        Self::prepared(|path| {
            project(path);
            None
        })
    }
    pub fn start_group() -> Self {
        Self::prepared(|path| Some(group::write(path)))
    }
    pub fn prepared(setup: impl FnOnce(&Path) -> Option<PathBuf>) -> Self {
        Self::prepared_output(setup, false)
    }
    pub fn prepared_scoped(setup: impl FnOnce(&Path) -> Option<PathBuf>) -> Self {
        Self::prepared_output(setup, true)
    }
    fn prepared_output(setup: impl FnOnce(&Path) -> Option<PathBuf>, scoped: bool) -> Self {
        let directory = temporary();
        let project_path = directory.path().join("show.json");
        let manifest = setup(&project_path);
        let original = fs::read(&project_path).unwrap();
        let run = directory.path().join("run");
        let stderr = directory.path().join("stderr.log");
        let mut command = Command::new(env!("CARGO_BIN_EXE_stagemaster-execution-host"));
        command.arg(&project_path);
        if let Some(manifest) = manifest {
            command.arg("group").arg(manifest);
        } else {
            command.args(["sequence", SEQUENCE]);
        }
        command.arg(&run).arg("--software-output");
        if scoped {
            command
                .arg("--audio-scope")
                .arg(directory.path().join("output"));
        }
        let mut process = Process(
            command
                .stdout(Stdio::null())
                .stderr(fs::File::create(&stderr).unwrap())
                .spawn()
                .unwrap(),
        );
        let discovery_path = run.join("discovery.json");
        let deadline = Instant::now() + Duration::from_secs(10);
        let discovery = loop {
            if let Ok(bytes) = fs::read(&discovery_path) {
                break serde_json::from_slice(&bytes).unwrap();
            }
            assert!(
                process.0.try_wait().unwrap().is_none(),
                "执行子进程提前退出：{}",
                fs::read_to_string(&stderr).unwrap()
            );
            assert!(Instant::now() < deadline, "独立执行进程没有就绪");
            std::thread::sleep(Duration::from_millis(10));
        };
        Self {
            directory,
            process,
            discovery,
            http: client(),
            project: project_path,
            original,
        }
    }
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.discovery["url"].as_str().unwrap())
    }
    pub fn get(&self, path: &str) -> RequestBuilder {
        self.http
            .get(self.url(path))
            .bearer_auth(self.discovery["readToken"].as_str().unwrap())
    }
    pub fn post(&self, path: &str) -> RequestBuilder {
        self.http
            .post(self.url(path))
            .bearer_auth(self.discovery["controlToken"].as_str().unwrap())
    }
    pub async fn state(&self) -> Value {
        self.snapshot().await["state"].clone()
    }
    pub async fn snapshot(&self) -> Value {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let response = self.get("/state").send().await.unwrap();
            if response.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE {
                assert!(Instant::now() < deadline, "观察快照一直繁忙");
                tokio::time::sleep(Duration::from_millis(5)).await;
                continue;
            }
            assert!(response.status().is_success());
            return response.json::<Value>().await.unwrap()["snapshot"].clone();
        }
    }
    pub async fn session(&self) -> String {
        ok(self.post("/sessions")).await["sessionId"]
            .as_str()
            .unwrap()
            .to_owned()
    }
    pub async fn command(&self, id: &str, serial: u64, command: Value) -> Value {
        self::command(&self.http, &self.discovery, id, serial, command).await
    }
    pub async fn acquire(&self, id: &str, takeover: bool) -> Value {
        self.command(
            id,
            1,
            json!({"kind":"acquire","durationMs":60_000,"takeover":takeover}),
        )
        .await
    }
    pub async fn start_program(&self, id: &str, revision: &Value, serial: u64) -> Value {
        let step = ok(self.get("/source")).await["steps"][0]["id"].clone();
        self.command(id,serial,json!({"kind":"submit","expectedRevision":revision,"action":{"kind":"start","step":step}})).await
    }
    pub async fn close(&mut self) {
        assert_eq!(ok(self.post("/shutdown")).await["status"], "stopping");
        let deadline = Instant::now() + Duration::from_secs(9);
        loop {
            if let Some(status) = self.process.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < deadline, "独立执行进程未正常退出");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(!self.directory.path().join("run/discovery.json").exists());
        assert_eq!(fs::read(&self.project).unwrap(), self.original);
    }
}
pub async fn ok(request: RequestBuilder) -> Value {
    let response = request.send().await.unwrap();
    let status = response.status();
    let body = response.text().await.unwrap();
    assert!(status.is_success(), "接口拒绝：{status} {body}");
    serde_json::from_str(&body).unwrap()
}
pub async fn command(
    http: &Client,
    discovery: &Value,
    id: &str,
    serial: u64,
    command: Value,
) -> Value {
    let base = discovery["url"].as_str().unwrap();
    let control = discovery["controlToken"].as_str().unwrap();
    let first = ok(http
        .post(format!("{base}/sessions/{id}/commands"))
        .bearer_auth(control)
        .json(&json!({"serial":serial.to_string(),"ttlMs":5000,"command":command})))
    .await;
    if first["status"] == "complete" {
        return first["outcome"].clone();
    }
    let deadline = Instant::now() + Duration::from_secs(7);
    loop {
        let view = ok(http
            .get(format!("{base}/sessions/{id}/receipts/{serial}"))
            .bearer_auth(control))
        .await;
        if view["status"] == "complete" {
            return view["outcome"].clone();
        }
        assert!(Instant::now() < deadline, "操作回执未返回");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
