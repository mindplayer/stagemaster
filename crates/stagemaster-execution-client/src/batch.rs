use crate::{Client, Selection, View};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum BatchAction {
    Pause {},
    Resume {},
    Stop {},
}
impl Client {
    /// Submit one bounded ordinary-program command through the original sequence/receipt channel.
    pub async fn batch(
        &mut self,
        host: &str,
        revision: &str,
        sources: &[String],
        action: BatchAction,
    ) -> Result<View, String> {
        if host != self.transport.discovery.host_id
            || !self.catalog.capabilities.iter().any(|c| c == "sourceBatch")
        {
            return Err("后台已更换或未提供节目批量操作，请重新核对".into());
        }
        crate::validation::decimal(revision)?;
        if sources.is_empty() || sources.len() > 64 {
            return Err("批量节目须为 1—64 项".into());
        }
        for (i, id) in sources.iter().enumerate() {
            crate::validation::identity(id)?;
            if sources[..i].contains(id)
                || !self.catalog.sources.iter().any(|s| {
                    s.id == *id
                        && matches!(
                            s.selection,
                            Selection::Scene { .. } | Selection::Sequence { .. }
                        )
                })
            {
                return Err("批量目标重复、不存在或不是普通节目；未提交修改".into());
            }
        }
        self.send(json!({"kind":"submit","expectedRevision":revision,"action":{"kind":"batch","sources":sources,"action":action}})).await
    }
}
