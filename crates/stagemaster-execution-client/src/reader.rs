//! Observation-only client. It never opens or renews a control session.
use crate::{Catalog, State, discovery::Discovery, freshness::Freshness, http::Transport};
use reqwest::Method;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub kind: String,
    pub boot: String,
    pub layout: String,
    pub universe: u16,
    pub revision: String,
    pub sampled_ms: String,
    pub composition_version: String,
    pub slots: Vec<u8>,
}
#[derive(Deserialize)]
struct Observation {
    phase: String,
    fault: Option<String>,
    snapshot: Option<Snapshot>,
}
#[derive(Deserialize)]
struct Snapshot {
    state: State,
    cycles: String,
    frame: Option<Sample>,
}
pub struct Reader {
    transport: Transport,
    catalog: Catalog,
    freshness: Freshness,
}
impl Reader {
    pub async fn open(path: &Path) -> Result<Self, String> {
        let mut discovery = Discovery::read(path)?;
        // This object retains no credential capable of creating a control session.
        discovery.control_token.clear();
        let transport = Transport::new(discovery)?;
        let catalog: Catalog = transport.request(Method::GET, "/source", None).await?;
        crate::validation::catalog(&catalog)?;
        Ok(Self {
            transport,
            catalog,
            freshness: Freshness::default(),
        })
    }
    #[must_use]
    pub fn host_id(&self) -> &str {
        &self.transport.discovery.host_id
    }
    #[must_use]
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub async fn project(&self) -> Result<Vec<u8>, String> {
        let bytes = self.transport.bytes(Method::GET, "/project", None).await?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        if digest != self.catalog.layout {
            return Err("后台固定工程与运行布局不一致".into());
        }
        Ok(bytes)
    }
    pub async fn sample(&mut self) -> Result<Sample, String> {
        let requested = Instant::now();
        let observation: Observation = self.transport.request(Method::GET, "/state", None).await?;
        let now = Instant::now();
        if now.duration_since(requested) >= std::time::Duration::from_secs(2) {
            return Err("后台观察响应已过期，请重新读取".into());
        }
        if observation.phase != "running" || observation.fault.is_some() {
            return Err("后台已停止或发生故障，三维暂停显示灯光".into());
        }
        let snapshot = observation.snapshot.ok_or("后台尚无有效采样")?;
        let frame = snapshot.frame.ok_or("后台尚无有效输出")?;
        let state = snapshot.state;
        crate::progress::validate(&self.catalog, &state)?;
        crate::manual_validation::state(&self.catalog, &state)?;
        crate::output::state(&self.catalog, &state)?;
        crate::validation::media_state(&self.catalog, &state)?;
        let sampled = decimal(&frame.sampled_ms)?;
        let observed = decimal(&state.observed_ms)?;
        if state.fault
            || state.boot != self.host_id()
            || state.layout != self.catalog.layout
            || frame.kind != "softwareSample"
            || frame.boot != state.boot
            || frame.layout != state.layout
            || frame.revision != state.revision
            || frame.slots.len() != 512
            || frame.universe == 0
            || sampled > observed
            || observed - sampled >= 2000
        {
            return Err("后台采样身份、版本或完整性不一致".into());
        }
        decimal(&frame.revision)?;
        self.freshness.accept(
            decimal(&snapshot.cycles)?,
            sampled,
            decimal(&frame.composition_version)?,
            now,
        )?;
        Ok(frame)
    }
}
fn decimal(value: &str) -> Result<u64, String> {
    let result: u64 = value.parse().map_err(|_| "后台计数格式无效")?;
    if result.to_string() != value {
        return Err("后台计数格式无效".into());
    }
    Ok(result)
}
