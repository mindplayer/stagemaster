use crate::{
    report_csv::literal,
    view::{ProjectView, SequenceView, StepView},
};
use std::collections::BTreeMap;

pub(super) const FORMAT: &str = "StageMaster 节目单/1";
pub(super) const HEADERS: [&str; 22] = [
    "资料格式",
    "工程名称",
    "工程标识",
    "来源保存修订",
    "工程快照 SHA256",
    "场景列表名称",
    "场景列表标识",
    "继承方式",
    "结束行为",
    "执行顺序",
    "步骤号",
    "步骤名称",
    "步骤标识",
    "幕场",
    "台词或动作提示",
    "备注",
    "场景名称",
    "场景标识",
    "延时（秒）",
    "渐变（秒）",
    "推进方式",
    "渐变完成后自动等待（秒）",
];
pub(super) struct SequenceRows<'a> {
    metadata: [String; 9],
    scenes: BTreeMap<&'a str, &'a str>,
}
impl<'a> SequenceRows<'a> {
    pub(super) fn new(
        project: &'a ProjectView,
        sequence: &SequenceView,
        revision: &str,
        digest: &str,
    ) -> Self {
        Self {
            metadata: [
                FORMAT.into(),
                literal(&project.name),
                project.id.clone(),
                revision.into(),
                digest.into(),
                literal(&sequence.name),
                sequence.id.clone(),
                if sequence.tracking == "inherited" {
                    "继承前序"
                } else {
                    "独立场景"
                }
                .into(),
                if sequence.repeat == "loop" {
                    "循环执行"
                } else {
                    "执行一遍"
                }
                .into(),
            ],
            scenes: project
                .scenes
                .iter()
                .map(|scene| (scene.id.as_str(), scene.name.as_str()))
                .collect(),
        }
    }
    pub(super) fn step(&self, position: usize, step: &StepView) -> Result<Vec<String>, String> {
        let scene = self
            .scenes
            .get(step.scene_id.as_str())
            .ok_or("节目单中的场景已不存在，请检查列表")?;
        let mut row = self.metadata.to_vec();
        row.extend([
            (position + 1).to_string(),
            literal(&step.number),
            literal(&step.name),
            step.id.clone(),
            literal(step.script.as_ref().map_or("", |s| &s.section)),
            literal(step.script.as_ref().map_or("", |s| &s.trigger)),
            literal(step.script.as_ref().map_or("", |s| &s.notes)),
            literal(scene),
            step.scene_id.clone(),
            seconds(step.delay_ms),
            seconds(step.fade_ms),
            if step.wait_ms.is_some() {
                "自动推进"
            } else {
                "手动推进"
            }
            .into(),
            step.wait_ms.map_or_else(String::new, seconds),
        ]);
        Ok(row)
    }
}
fn seconds(ms: u64) -> String {
    format!("{}.{:03}", ms / 1000, ms % 1000)
}
