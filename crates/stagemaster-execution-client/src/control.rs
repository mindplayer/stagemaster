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
        if self
            .catalog
            .sources
            .iter()
            .any(|s| s.id == source && matches!(s.selection, crate::Selection::AudioTimeline {}))
            && !matches!(action, Action::Level { .. })
        {
            return Err("音乐请使用播放、暂停、停止或定位操作".into());
        }
        if let Action::Patch { changes } = &action {
            crate::manual_validation::edits(&self.catalog, source, changes)?;
        }
        self.send(json!({"kind":"submit","expectedRevision":revision,"action":{"kind":"source","source":source,"action":action}})).await
    }
    pub(super) async fn send(&mut self, command: Value) -> Result<View, String> {
        self.send_internal(command, true).await
    }
    pub(super) async fn send_internal(
        &mut self,
        command: Value,
        explicit: bool,
    ) -> Result<View, String> {
        if self.pending {
            return Err("上一操作尚未确认，请先查询回执或明确重新连接".into());
        }
        let id = self.session.as_ref().ok_or("请先取得运行控制权")?;
        let serial = self.next.to_string();
        let suffix = format!("/sessions/{id}/commands");
        let body = json!({"serial":serial,"ttlMs":5000,"command":command});
        let limit = self
            .catalog
            .limits
            .as_ref()
            .map_or(8192, |v| v.request_bytes.min(8192));
        if serde_json::to_vec(&body).map_err(|e| e.to_string())?.len() > limit {
            return Err("本次操作超过后台请求容量，请减少目标后重试；未提交任何修改".into());
        }
        // Retained before awaiting: cancellation and lost HTTP replies cannot free the next serial.
        self.pending = true;
        self.record = Some(Record {
            serial: serial.clone(),
            status: "pending".into(),
            outcome: None,
        });
        if explicit {
            self.operation_record.clone_from(&self.record);
        }
        let evidence = self
            .media_operation
            .as_mut()
            .filter(|_| explicit && body["command"]["action"]["kind"] == "media");
        let record = if let Some(evidence) = evidence {
            evidence.serial = Some(serial);
            evidence.attempted = true;
            self.transport
                .request_traced(Method::POST, &suffix, Some(body), &mut evidence.submission)
                .await?
        } else {
            self.transport
                .request(Method::POST, &suffix, Some(body))
                .await?
        };
        self.accept(record)?;
        self.refresh().await
    }
    pub(super) async fn resolve(&mut self) -> Result<(), String> {
        let id = self.session.as_ref().ok_or("缺少控制会话")?;
        let serial = &self.record.as_ref().ok_or("缺少操作回执")?.serial;
        let suffix = format!("/sessions/{id}/receipts/{serial}");
        let evidence = self
            .media_operation
            .as_mut()
            .filter(|e| e.serial.as_ref() == Some(serial));
        let record = if let Some(evidence) = evidence {
            let trace = evidence
                .receipt_read
                .insert(crate::MediaHttpEvidence::default());
            self.transport
                .request_traced(Method::GET, &suffix, None, trace)
                .await?
        } else {
            self.transport.request(Method::GET, &suffix, None).await?
        };
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
            if let Some(evidence) = &mut self.media_operation
                && evidence.serial.as_ref() == self.record.as_ref().map(|r| &r.serial)
            {
                evidence
                    .receipt_read
                    .as_mut()
                    .unwrap_or(&mut evidence.submission)
                    .problem = Some(crate::ResponseProblem::ReceiptMismatch);
            }
            return Err("后台操作回执不匹配，请重新连接核对".into());
        }
        if let Some(evidence) = &mut self.media_operation {
            evidence.receive(&record);
        }
        if record.status == "complete" {
            // Unknown is a completed observation, but still blocks further action in this session.
            self.pending = record.outcome.as_ref().is_some_and(|o| o.kind == "unknown");
            if !self.pending {
                self.next = self.next.checked_add(1).ok_or("控制序号已耗尽")?;
            }
        }
        // Heartbeats may complete between UI polls. They must not erase the explicit operation
        // being awaited by a manual or media editor; wire sequencing remains unchanged.
        if self
            .operation_record
            .as_ref()
            .is_some_and(|r| r.serial == record.serial)
        {
            self.operation_record = Some(record.clone());
        }
        self.record = Some(record);
        Ok(())
    }
}
