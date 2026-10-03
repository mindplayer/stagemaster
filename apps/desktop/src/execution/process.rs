use super::{files, media};
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
    audio: Option<media::AudioInput>,
) -> Result<PathBuf, String> {
    if selections.is_empty() || selections.len() > 63 {
        return Err("请选择 1 至 63 个场景、场景列表或音乐编排".into());
    }
    media::validate(document, &selections, audio.as_ref().map(|a| a.output))?;
    let mut keys = std::collections::HashSet::new();
    for selection in &selections {
        let (kind, id) = match selection {
            Selection::Scene { id } => ("scene", id.as_str()),
            Selection::Sequence { id } => ("sequence", id.as_str()),
            Selection::Manual {} => return Err("手动层由后台自动准备".into()),
            Selection::AudioTimeline {} => ("audioTimeline", ""),
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
            Selection::AudioTimeline {} => {}
            Selection::Manual {} => unreachable!(),
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
    if let Some(audio) = &audio {
        media::copy(document, &run, audio)?;
    }
    files::create(&run.join("project.json"), &document.encode()?)?;
    let mut manifest = serde_json::json!({"version":1,"sources":sources});
    if let Some(audio) = audio {
        manifest["version"] = 2.into();
        manifest["audio"] = serde_json::json!({"output":audio.output});
    }
    files::create(
        &run.join("sources.json"),
        &serde_json::to_vec(&manifest).map_err(|e| e.to_string())?,
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
