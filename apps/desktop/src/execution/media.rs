use super::{files, manager::Manager};
use stagemaster_audio::Resources;
use stagemaster_execution_client::{AudioOutput, MediaAction, Selection};
use stagemaster_project::Document;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub(crate) struct AudioInput {
    pub source: PathBuf,
    pub output: AudioOutput,
}
pub(super) fn validate(
    document: &Document,
    selections: &[Selection],
    output: Option<AudioOutput>,
) -> Result<(), String> {
    let count = selections
        .iter()
        .filter(|s| matches!(s, Selection::AudioTimeline {}))
        .count();
    if count > 1 || (count == 1) != output.is_some() {
        return Err("音乐编排须选择一个明确的声音输出，且不能重复载入".into());
    }
    if count == 1 {
        document
            .audio_timeline()
            .ok_or("工程还没有音乐")?
            .compile_loops(1000)?;
        document.compile_audio_segment(None)?;
    }
    Ok(())
}
pub(super) fn copy(document: &Document, run: &Path, audio: &AudioInput) -> Result<(), String> {
    let track = document.audio_timeline().ok_or("工程还没有音乐")?;
    Resources::new(run.join("project.json.assets")).import(
        &audio.source,
        &track.asset.extension,
        Some(&track.asset.digest),
        &AtomicBool::new(false),
    )?;
    Ok(())
}
impl Manager {
    /// Keep this file lock through the editor operation, including slow native preparation.
    pub(super) fn reserve_editor_audio(&mut self) -> Result<File, String> {
        files::private_directory(&self.root)?;
        let lock = files::lock(&self.root)?;
        if let Some(run) = files::read_run(&self.root)?
            && !files::ended(&run)
        {
            let mut bytes = Vec::new();
            File::open(run.join("sources.json"))
                .map_err(|_| "无法核对后台音乐，请先连接后台")?
                .take(65_537)
                .read_to_end(&mut bytes)
                .map_err(|_| "无法读取后台节目配置")?;
            if bytes.len() > 65_536 {
                return Err("后台节目配置超出限制".into());
            }
            let manifest: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|_| "后台节目配置无效")?;
            let sources = manifest["sources"].as_array().ok_or("后台节目配置无效")?;
            if !matches!(manifest["version"].as_u64(), Some(1 | 2)) {
                return Err("无法识别后台节目配置版本".into());
            }
            if !manifest["audio"].is_null()
                || sources
                    .iter()
                    .any(|s| s["selection"]["kind"] == "audioTimeline")
            {
                return Err("后台已载入音乐，请在执行页控制；关闭后台后可恢复编辑试听".into());
            }
        }
        Ok(lock)
    }
    pub async fn apply_media(
        &mut self,
        host: &str,
        revision: &str,
        group: &str,
        generation: &str,
        action: MediaAction,
    ) -> Result<super::manager::Status, String> {
        if self.closing {
            return Err("后台正在关闭".into());
        }
        self.client
            .as_mut()
            .ok_or("请先连接后台")?
            .apply_media(host, revision, group, generation, action)
            .await?;
        Ok(self.status())
    }
}
