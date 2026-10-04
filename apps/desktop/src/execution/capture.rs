//! Read-only recording adapter. Refreshes the original client, never sends a manual command.
use super::{Service, manager::Manager};
use stagemaster_execution_client::{ManualFixture, Selection, View};
use stagemaster_project::{Document, ManualSceneCapture, ManualSceneReading};
use std::{collections::HashSet, fs::File, io::Read};

pub(crate) struct Collected {
    pub capture: ManualSceneCapture,
    pub fixtures: Vec<ManualFixture>,
    pub source_name: String,
    pub revision: String,
}
impl Service {
    pub(crate) async fn capture(
        &self,
        host: String,
        source: String,
        selected: Option<Vec<String>>,
    ) -> Result<Collected, String> {
        let mut manager = self
            .0
            .clone()
            .try_lock_owned()
            .map_err(|_| "后台操作正在处理，请稍后重试")?;
        let runtime = tokio::runtime::Handle::current();
        tauri::async_runtime::spawn_blocking(move || {
            runtime.block_on(manager.capture(&host, &source, selected.as_deref()))
        })
        .await
        .map_err(|_| "手动值采集任务异常结束")?
    }
}
impl Manager {
    pub(super) async fn capture(
        &mut self,
        host: &str,
        source: &str,
        selected: Option<&[String]>,
    ) -> Result<Collected, String> {
        if self.closing {
            return Err("后台正在关闭".into());
        }
        let status = self.poll().await;
        if let Some(problem) = status.problem {
            return Err(problem);
        }
        let view = status.runtime.ok_or("请先连接后台")?;
        let path = self
            .run
            .as_ref()
            .ok_or("缺少后台固定工程")?
            .join("project.json");
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !meta.is_file()
            || meta.file_type().is_symlink()
            || meta.len() > stagemaster_project::MAX_BYTES as u64
        {
            return Err("后台固定工程文件无效".into());
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(|e| e.to_string())?
            .take(stagemaster_project::MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        collect(&Document::decode(&bytes)?, &view, host, source, selected)
    }
}

pub(super) fn collect(
    document: &Document,
    view: &View,
    host: &str,
    source: &str,
    selected: Option<&[String]>,
) -> Result<Collected, String> {
    if view.host_id != host
        || view.pending
        || view.observation.phase != "running"
        || view.observation.fault.is_some()
    {
        return Err("后台已变化或尚未确认操作，请刷新后重新采集".into());
    }
    if !view
        .catalog
        .capabilities
        .iter()
        .any(|c| c == "manualValues")
    {
        return Err("后台没有提供实际手动值，请重新载入节目".into());
    }
    let state = &view
        .observation
        .snapshot
        .as_ref()
        .ok_or("后台状态尚未就绪")?
        .state;
    if state.fault {
        return Err("后台执行发生故障，不能采集".into());
    }
    let entry = view
        .catalog
        .sources
        .iter()
        .find(|s| s.id == source && matches!(s.selection, Selection::Manual {}))
        .ok_or("请选择后台手动层")?;
    let state_source = state
        .sources
        .iter()
        .find(|s| s.id == source)
        .ok_or("后台缺少此手动层")?;
    let fixtures = view.catalog.fixtures.as_ref().ok_or("后台缺少灯具定义")?;
    if let Some(ids) = selected {
        let unique: HashSet<_> = ids.iter().collect();
        if ids.is_empty()
            || ids.len() > 512
            || unique.len() != ids.len()
            || ids.iter().any(|id| !fixtures.iter().any(|f| &f.id == id))
        {
            return Err("录入的灯具选择已失效".into());
        }
    }
    let held = state_source.held.as_ref().ok_or("后台缺少实际持有范围")?;
    let values = state_source
        .held_values
        .as_ref()
        .ok_or("后台缺少实际手动值")?;
    if held.len() != values.len() {
        return Err("后台持有范围和数值不一致".into());
    }
    let readings = held
        .iter()
        .zip(values)
        .filter(|(target, _)| selected.is_none_or(|ids| ids.contains(&target.fixture_id)))
        .map(|(target, value)| ManualSceneReading {
            fixture_id: target.fixture_id.clone(),
            attribute: target.attribute.clone(),
            value: *value,
        })
        .collect();
    let capture = document.capture_manual_scene(&view.catalog.layout, readings)?;
    Ok(Collected {
        fixtures: fixtures
            .iter()
            .filter(|f| capture.readings().iter().any(|r| r.fixture_id == f.id))
            .cloned()
            .collect(),
        capture,
        source_name: entry.name.clone(),
        revision: state.revision.clone(),
    })
}
