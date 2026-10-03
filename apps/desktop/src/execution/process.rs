use super::files;
use stagemaster_execution_client::Selection;
use stagemaster_project::Document;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};
use tauri::Manager as _;
use uuid::Uuid;

pub(super) fn prepare(
    root: &Path,
    document: &Document,
    selections: Vec<Selection>,
) -> Result<PathBuf, String> {
    if selections.is_empty() || selections.len() > 63 {
        return Err("请选择 1 至 63 个场景或场景列表".into());
    }
    let mut keys = std::collections::HashSet::new();
    for selection in &selections {
        let (kind, id) = match selection {
            Selection::Scene { id } => ("scene", id),
            Selection::Sequence { id } => ("sequence", id),
            Selection::Manual {} => return Err("手动层由后台自动准备".into()),
            Selection::AudioTimeline {} => {
                return Err("音乐后台载入入口尚未接入，请保留当前后台".into());
            }
        };
        if !keys.insert((kind, id)) {
            return Err("同一节目不能重复载入".into());
        }
        // Preparation errors are shown before creating any background process.
        match selection {
            Selection::Scene { id } => {
                document.compile_scene(id)?;
            }
            Selection::Sequence { id } => {
                document.compile_sequence(id)?;
            }
            Selection::Manual {} | Selection::AudioTimeline {} => unreachable!(),
        }
    }
    let id = Uuid::new_v4().to_string();
    let run = root.join(&id);
    files::private_directory(&run)?;
    let mut sources: Vec<_> = selections
        .into_iter()
        .map(|selection| {
            serde_json::json!({
                "id":Uuid::new_v4().to_string(),"priority":0,"selection":selection
            })
        })
        .collect();
    sources.push(serde_json::json!({"id":Uuid::new_v4().to_string(),"priority":100,"selection":{"kind":"manual"}}));
    files::create(&run.join("project.json"), &document.encode()?)?;
    files::create(
        &run.join("sources.json"),
        &serde_json::to_vec(&serde_json::json!({"version":1,"sources":sources}))
            .map_err(|e| e.to_string())?,
    )?;
    files::record(root, &id)?;
    Ok(run)
}
pub(super) fn launch(binary: &Path, run: &Path) -> Result<Child, String> {
    files::create(&run.join("process.log"), &[])?;
    let log = fs::OpenOptions::new()
        .append(true)
        .open(run.join("process.log"))
        .map_err(|e| e.to_string())?;
    let error = log.try_clone().map_err(|e| e.to_string())?;
    let mut command = Command::new(binary);
    command
        .arg(run.join("project.json"))
        .arg("group")
        .arg(run.join("sources.json"))
        .arg(run.join("host"))
        .arg("--software-output")
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(error);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command.spawn().map_err(|e| format!("无法启动后台：{e}"))
}
pub(super) fn binary(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let path = tauri::process::current_binary(&app.env()).map_err(|e| e.to_string())?;
    let binary = path
        .parent()
        .ok_or("缺少应用目录")?
        .join("stagemaster-execution-host");
    if !binary.is_file() {
        return Err("应用缺少后台程序，请通过项目桌面构建入口重新构建".into());
    }
    Ok(binary)
}
