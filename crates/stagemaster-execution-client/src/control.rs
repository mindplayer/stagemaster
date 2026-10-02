use super::{Action, Client, Record, View};
use reqwest::Method;
use serde_json::{Value, json};

impl Client {
    pub async fn acquire(&mut self, takeover: bool) -> Result<View, String> {
        if self.pending {
            return Err("上一操作尚未确认，请先查询回执或明确重新连接".into());
        }
        if self.session.is_none() {
            let result: Value = self
                .transport
                .request(Method::POST, "/sessions", None)
                .await?;
            let id = result["sessionId"].as_str().ok_or("后台未返回会话")?;
            let parsed = uuid::Uuid::parse_str(id).map_err(|_| "后台会话身份无效")?;
            if parsed.is_nil()
                || parsed.to_string() != id
                || result["hostId"] != self.transport.discovery.host_id
                || result["nextSerial"] != "1"
            {
                return Err("后台会话响应不一致".into());
            }
            self.session = Some(id.into());
            self.next = 1;
        }
        self.send(json!({"kind":"acquire","durationMs":60000,"takeover":takeover}))
            .await
    }
    pub async fn release(&mut self) -> Result<View, String> {
        self.send(json!({"kind":"release"})).await
    }
    pub async fn apply(
        &mut self,
        host: &str,
        revision: &str,
        source: &str,
        action: Action,
    ) -> Result<View, String> {
        if host != self.transport.discovery.host_id
            || !self.catalog.sources.iter().any(|s| s.id == source)
        {
            return Err("操作对应的后台或节目已更换，请重新选择".into());
        }
        self.send(json!({"kind":"submit","expectedRevision":revision,"action":{"kind":"source","source":source,"action":action}})).await
    }
    pub(super) async fn send(&mut self, command: Value) -> Result<View, String> {
        if self.pending {
            return Err("上一操作尚未确认，请先查询回执或明确重新连接".into());
        }
        let id = self.session.as_ref().ok_or("请先取得运行控制权")?;
        let serial = self.next.to_string();
        let suffix = format!("/sessions/{id}/commands");
        // Retained before awaiting: cancellation and lost HTTP replies cannot free the next serial.
        self.pending = true;
        self.record = Some(Record {
            serial: serial.clone(),
            status: "pending".into(),
            outcome: None,
        });
        let record = self
            .transport
            .request(
                Method::POST,
                &suffix,
                Some(json!({"serial":serial,"ttlMs":5000,"command":command})),
            )
            .await?;
        self.accept(record)?;
        self.refresh().await
    }
    pub(super) async fn resolve(&mut self) -> Result<(), String> {
        let id = self.session.as_ref().ok_or("缺少控制会话")?;
        let serial = &self.record.as_ref().ok_or("缺少操作回执")?.serial;
        let record = self
            .transport
            .request(
                Method::GET,
                &format!("/sessions/{id}/receipts/{serial}"),
                None,
            )
            .await?;
        self.accept(record)
    }
    fn accept(&mut self, record: Record) -> Result<(), String> {
        if self
            .record
            .as_ref()
            .is_none_or(|r| r.serial != record.serial)
            || !matches!(record.status.as_str(), "pending" | "complete")
            || (record.status == "complete" && record.outcome.is_none())
        {
            return Err("后台操作回执不匹配，请重新连接核对".into());
        }
        if record.status == "complete" {
            // Unknown is a completed observation, but still blocks further action in this session.
            self.pending = record.outcome.as_ref().is_some_and(|o| o.kind == "unknown");
            if !self.pending {
                self.next = self.next.checked_add(1).ok_or("控制序号已耗尽")?;
            }
        }
        self.record = Some(record);
        Ok(())
    }
}
