//! Own the renderer and the upstream signalling service. Neither controls the show clock.
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};
use tauri::Manager;

struct OwnedChild(Child);
impl OwnedChild {
    fn exited(&mut self) -> bool {
        !matches!(self.0.try_wait(), Ok(None))
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if matches!(self.0.try_wait(), Ok(None)) {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}
pub(super) struct Renderer {
    engine: OwnedChild,
    signalling: OwnedChild,
    viewer_url: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Ports {
    renderer_port: u16,
    viewer_port: u16,
}
impl Renderer {
    pub(super) fn start(
        app: &tauri::AppHandle,
        server: &super::server::Server,
    ) -> Result<Self, String> {
        let paths = launch_paths(app)?;
        let logs = paths.root.join("logs");
        std::fs::create_dir_all(&logs).map_err(|_| "无法创建预演日志目录")?;
        let renderer_token = token();
        let viewer_token = token();
        let signal_errors =
            File::create(logs.join("previs-signalling.log")).map_err(|_| "无法写入预演连接日志")?;
        let mut signalling = OwnedChild(
            Command::new(&paths.node)
                .arg(&paths.signalling)
                .env("STAGEMASTER_STREAM_RENDERER_TOKEN", &renderer_token)
                .env("STAGEMASTER_STREAM_VIEWER_TOKEN", &viewer_token)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::from(signal_errors))
                .spawn()
                .map_err(|_| "三维画面连接服务无法启动")?,
        );
        let output = signalling.0.stdout.take().ok_or("无法读取三维连接状态")?;
        let (sender, receiver) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            let mut line = String::new();
            let result = BufReader::new(output.take(512))
                .read_line(&mut line)
                .map(|_| line);
            let _ = sender.send(result);
        });
        let ready = receiver.recv_timeout(Duration::from_secs(5));
        if ready.is_err() {
            drop(signalling);
            let _ = reader.join();
            return Err("三维画面连接服务未就绪".into());
        }
        let _ = reader.join();
        let line = ready
            .map_err(|_| "无法读取三维连接状态")?
            .map_err(|_| "三维画面连接服务未就绪")?;
        let ports: Ports = serde_json::from_str(&line).map_err(|_| "三维画面连接状态无效")?;
        if ports.renderer_port == 0 || ports.viewer_port == 0 {
            return Err("三维连接端口无效".into());
        }
        let output = File::create(logs.join("previs-renderer-console.log"))
            .map_err(|_| "无法写入预演日志")?;
        let errors = output.try_clone().map_err(|_| "无法初始化预演日志")?;
        let mut command = Command::new(paths.program);
        if let Some(project) = paths.project {
            command.arg(project).arg("-game");
        }
        command
            .args([
                "-RenderOffscreen",
                "-ForceRes",
                "-ResX=1280",
                "-ResY=720",
                "-NoSplash",
                "-NoSound",
                "-NoP4",
                "-NoTraceServer",
            ])
            .arg(format!(
                "-abslog={}",
                logs.join("previs-renderer.log").display()
            ))
            .env(
                "UE-LocalDataCachePath",
                paths.root.join("data/previs-derived-cache"),
            )
            .env("UE-ZenDataPath", paths.root.join("data/previs-zen"))
            .env(
                "STAGEMASTER_STREAM_URL",
                format!("ws://127.0.0.1:{}/{}", ports.renderer_port, renderer_token),
            )
            .stdin(Stdio::null())
            .stdout(Stdio::from(output))
            .stderr(Stdio::from(errors));
        server.configure_renderer(&mut command);
        let engine = OwnedChild(
            command
                .spawn()
                .map_err(|_| "三维预演启动失败，请检查组件是否完整")?,
        );
        Ok(Self {
            engine,
            signalling,
            viewer_url: format!("ws://127.0.0.1:{}/{}", ports.viewer_port, viewer_token),
        })
    }
    pub(super) fn viewer_url(&self) -> &str {
        &self.viewer_url
    }
    pub(super) fn exited(&mut self) -> bool {
        self.engine.exited() || self.signalling.exited()
    }
}
fn token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
struct LaunchPaths {
    program: PathBuf,
    project: Option<PathBuf>,
    root: PathBuf,
    node: PathBuf,
    signalling: PathBuf,
}
fn executable(base: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        base.join("StageMasterPreview.app/Contents/MacOS/StageMasterPreview")
    } else if cfg!(target_os = "windows") {
        base.join("StageMasterPreview.exe")
    } else {
        base.join("StageMasterPreview")
    }
}
fn launch_paths(app: &tauri::AppHandle) -> Result<LaunchPaths, String> {
    if let Ok(resources) = app.path().resource_dir() {
        let base = resources.join("previs");
        let program = executable(&base);
        let node = base.join(if cfg!(target_os = "windows") {
            "node.exe"
        } else {
            "node"
        });
        let signalling = base.join("signalling.mjs");
        if program.is_file() && node.is_file() && signalling.is_file() {
            return Ok(LaunchPaths {
                program,
                project: None,
                node,
                signalling,
                root: app
                    .path()
                    .app_local_data_dir()
                    .map_err(|_| "无法定位预演数据目录")?,
            });
        }
    }
    #[cfg(all(debug_assertions, target_os = "macos"))]
    {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|_| "无法定位本地预演工程")?;
        let project = root.join("apps/previs-unreal/StageMasterPreview.uproject");
        let engine = std::env::var_os("STAGEMASTER_UE_ROOT").map_or_else(
            || PathBuf::from("/Users/Shared/Epic Games/UE_5.8"),
            PathBuf::from,
        );
        let program =
            engine.join("Engine/Binaries/Mac/UnrealEditor.app/Contents/MacOS/UnrealEditor");
        let module =
            root.join("apps/previs-unreal/Binaries/Mac/libUnrealEditor-StageMasterPreview.dylib");
        let node = PathBuf::from(
            option_env!("STAGEMASTER_NODE_BINARY").unwrap_or("/opt/homebrew/bin/node"),
        );
        let signalling = root.join("tools/previs/signalling.mjs");
        if program.is_file()
            && project.is_file()
            && module.is_file()
            && node.is_file()
            && signalling.is_file()
        {
            return Ok(LaunchPaths {
                program,
                project: Some(project),
                root,
                node,
                signalling,
            });
        }
    }
    Err("尚未安装可运行的三维预演组件".into())
}
