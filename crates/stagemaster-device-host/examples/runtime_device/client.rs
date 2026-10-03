use serde_json::{Value, json};
use stagemaster_device_host::{
    Ble, Phase, Request, Service, Snapshot,
    runtime_ui::{ExpectedAccess, Request as RuntimeRequest},
};
use std::{error::Error, time::Duration};
use tokio::time::{Instant, sleep};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub const DEVICE: &str = "534d4553503332533300288485569774";
pub struct Probe {
    pub link: Service<Ble>,
    pub access: ExpectedAccess,
}
impl Probe {
    pub fn snapshot(&self) -> Result<Snapshot> {
        Ok(self.link.request(Request::Status)?)
    }
    pub fn epoch(&self) -> Result<u32> {
        Ok(self.snapshot()?.epoch)
    }
    pub async fn settled(&self) -> Result<Snapshot> {
        let until = Instant::now() + Duration::from_secs(30);
        loop {
            let s = self.snapshot()?;
            match s.phase {
                Phase::Idle | Phase::Connected => return Ok(s),
                Phase::Fault | Phase::Blocked => {
                    return Err(format!("连接失败：{:?}", s.problem).into());
                }
                _ => {}
            }
            if Instant::now() >= until {
                return Err("连接状态等待超时".into());
            }
            sleep(Duration::from_millis(100)).await;
        }
    }
    pub async fn discover(&self, locator: Option<&str>) -> Result<String> {
        self.link.request(Request::Scan {
            epoch: self.epoch()?,
        })?;
        let found = self.settled().await?;
        println!("本次发现 {}", serde_json::to_string(&found.candidates)?);
        match locator {
            Some(id) if found.candidates.iter().any(|c| c.id == id) => Ok(id.into()),
            None if found.candidates.len() == 1 => Ok(found.candidates[0].id.clone()),
            _ => Err("请从本次发现结果明确选择连接标识".into()),
        }
    }
    pub async fn connect(&self, locator: &str) -> Result<()> {
        self.link
            .runtime_request(
                RuntimeRequest::Connect {
                    epoch: self.epoch()?,
                    id: locator.into(),
                },
                &self.access,
            )
            .await?;
        let state = self.settled().await?;
        if state.phase != Phase::Connected {
            return Err("运行连接未建立".into());
        }
        let d = state.description.as_ref().ok_or("缺少实际设备描述")?;
        assert_eq!(d.device_id, DEVICE);
        assert_eq!(d.authentication_method, 2);
        assert!(
            state
                .diagnostics
                .as_ref()
                .is_some_and(|d| d.self_test && d.output_disabled)
        );
        let view = self
            .json(json!({"kind":"snapshot","epoch":state.epoch}))
            .await?;
        assert_eq!(view["peer"]["device"], DEVICE);
        assert!(!view["pending"].as_bool().unwrap());
        println!("已确认禁止物理输出 {}", serde_json::to_string(&state)?);
        Ok(())
    }
    pub async fn disconnect(&self) -> Result<()> {
        self.link.request(Request::Cancel {
            epoch: self.epoch()?,
        })?;
        assert_eq!(self.settled().await?.phase, Phase::Idle);
        Ok(())
    }
    pub async fn json(&self, input: Value) -> Result<Value> {
        let request = serde_json::from_value(input)?;
        let view = serde_json::to_value(self.link.runtime_request(request, &self.access).await?)?;
        if let Some(error) = view["reply"]["body"]["error"].as_str() {
            return Err(error.into());
        }
        Ok(view)
    }
    pub async fn read(&self) -> Result<Value> {
        self.json(json!({"kind":"refresh","epoch":self.epoch()?}))
            .await
    }
    pub async fn apply(&self, action: Value) -> Result<Value> {
        let current = self.read().await?;
        let view = self.json(json!({"kind":"apply","epoch":self.epoch()?,"revision":current["reply"]["revision"],"action":action})).await?;
        println!("操作 {action}：{view}");
        Ok(view)
    }
}
pub fn state(view: &Value) -> &Value {
    &view["reply"]["body"]["state"]
}
pub fn elapsed(view: &Value) -> u64 {
    state(view)["elapsedMs"].as_str().unwrap().parse().unwrap()
}
