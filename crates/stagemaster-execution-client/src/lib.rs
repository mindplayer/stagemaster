//! Bounded local controller for the independently running v2 source group.
#![allow(clippy::missing_errors_doc)]
mod batch;
mod control;
pub use batch::BatchAction;
mod discovery;
mod freshness;
mod http;
mod manual;
mod manual_validation;
mod output;
pub use output::{OutputAction, OutputCatalog, OutputState};
mod media;
pub use manual::*;
mod media_control;
mod progress;
mod validation;
pub use media::*;
pub use progress::{Phase, Progress};
mod reader;
mod types;
use http::Transport;
pub use reader::{Reader, Sample};
use reqwest::Method;
use serde_json::Value;
use std::path::Path;
pub use types::*;

pub struct Client {
    transport: Transport,
    catalog: Catalog,
    observation: Observation,
    session: Option<String>,
    next: u64,
    record: Option<Record>,
    operation_record: Option<Record>,
    pending: bool,
}
impl Client {
    pub async fn open(discovery: &Path) -> Result<Self, String> {
        let transport = Transport::new(discovery::Discovery::read(discovery)?)?;
        let catalog: Catalog = transport.request(Method::GET, "/source", None).await?;
        validation::catalog(&catalog)?;
        let observation = transport.request(Method::GET, "/state", None).await?;
        let client = Self {
            transport,
            catalog,
            observation,
            session: None,
            next: 1,
            record: None,
            operation_record: None,
            pending: false,
        };
        client.validate(&client.observation)?;
        Ok(client)
    }
    fn validate(&self, observation: &Observation) -> Result<(), String> {
        if let Some(snapshot) = &observation.snapshot {
            self.validate_state(&snapshot.state)?;
        }
        Ok(())
    }
    fn validate_state(&self, state: &State) -> Result<(), String> {
        let unique: std::collections::HashSet<_> = state.sources.iter().map(|s| &s.id).collect();
        if state.boot != self.transport.discovery.host_id
            || state.layout != self.catalog.layout
            || state.revision.parse::<u64>().is_err()
            || state.observed_ms.parse::<u64>().is_err()
            || unique.len() != self.catalog.sources.len()
            || unique.len() != state.sources.len()
            || state
                .sources
                .iter()
                .any(|s| !self.catalog.sources.iter().any(|entry| entry.id == s.id))
        {
            return Err("后台身份、来源或运行版本不一致，请重新连接".into());
        }
        progress::validate(&self.catalog, state)?;
        manual_validation::state(&self.catalog, state)?;
        output::state(&self.catalog, state)?;
        validation::media_state(&self.catalog, state)
    }
    fn accept_observation(&mut self, mut observation: Observation) -> Result<(), String> {
        self.validate(&observation)?;
        if let Some(latest) = self
            .record
            .as_ref()
            .and_then(|r| r.outcome.as_ref())
            .and_then(|o| o.state.as_ref())
        {
            self.validate_state(latest)?;
            if let Some(snapshot) = &mut observation.snapshot
                && latest.revision.parse::<u64>().unwrap_or(0)
                    > snapshot.state.revision.parse::<u64>().unwrap_or(0)
            {
                // A published cycle can lag a confirmed command. Keep authoritative receipt state;
                // cycle counters still describe the last published observation, never invented ticks.
                snapshot.state = latest.clone();
            }
        }
        self.observation = observation;
        Ok(())
    }
    pub async fn refresh(&mut self) -> Result<View, String> {
        if self.pending {
            self.resolve().await?;
        }
        let observation = self.transport.request(Method::GET, "/state", None).await?;
        self.accept_observation(observation)?;
        Ok(self.view())
    }
    #[must_use]
    pub fn view(&self) -> View {
        View {
            host_id: self.transport.discovery.host_id.clone(),
            catalog: self.catalog.clone(),
            observation: self.observation.clone(),
            session_id: self.session.clone(),
            controlling: self.controlling(),
            pending: self.pending,
            record: self.operation_record.clone(),
        }
    }
    fn controlling(&self) -> bool {
        self.observation.snapshot.as_ref().is_some_and(|s| {
            !s.state.fault
                && s.state.owner.as_ref().is_some_and(|o| {
                    Some(&o.session_id) == self.session.as_ref()
                        && o.expires_ms.parse::<u64>().unwrap_or(0)
                            > s.state.observed_ms.parse::<u64>().unwrap_or(u64::MAX)
                })
        })
    }
    pub async fn maintain(&mut self) -> Result<View, String> {
        self.refresh().await?;
        if !self.pending
            && self.controlling()
            && self.observation.snapshot.as_ref().is_some_and(|s| {
                s.state.owner.as_ref().is_some_and(|o| {
                    o.expires_ms
                        .parse::<u64>()
                        .unwrap_or(0)
                        .saturating_sub(s.state.observed_ms.parse::<u64>().unwrap_or(u64::MAX))
                        < 30_000
                })
            })
        {
            self.send_internal(
                serde_json::json!({"kind":"renew","durationMs":60000}),
                false,
            )
            .await?;
        }
        Ok(self.view())
    }
    pub async fn shutdown(&self) -> Result<(), String> {
        let _: Value = self
            .transport
            .request(Method::POST, "/shutdown", None)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod manual_tests;
