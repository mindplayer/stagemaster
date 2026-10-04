use super::{files, process};
use serde::Serialize;
use stagemaster_execution_client::{Action, Client, Selection, View};
use stagemaster_project::Document;
use std::{path::PathBuf, process::Child, time::Duration};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    pub phase: &'static str,
    pub problem: Option<String>,
    pub runtime: Option<View>,
}
pub(crate) struct Manager {
    pub(super) root: PathBuf,
    binary: PathBuf,
    pub(super) run: Option<PathBuf>,
    child: Option<Child>,
    pub(super) client: Option<Client>,
    problem: Option<String>,
    pub(super) closing: bool,
}
impl Manager {
    #[cfg(test)]
    pub(super) fn take_test_child(&mut self) -> Child {
        self.child.take().unwrap()
    }
    pub fn new(root: PathBuf, binary: PathBuf) -> Self {
        Self {
            root,
            binary,
            run: None,
            child: None,
            client: None,
            problem: None,
            closing: false,
        }
    }
    fn discover(&mut self) -> Result<(), String> {
        files::private_directory(&self.root)?;
        if self.run.is_none() {
            self.run = files::read_run(&self.root)?;
        }
        Ok(())
    }
    pub fn discovery_path(&self) -> Result<PathBuf, String> {
        files::read_run(&self.root)?
            .map(|run| run.join("host/discovery.json"))
            .ok_or_else(|| "请先在执行视图载入后台节目".into())
    }
    fn terminal(&mut self) -> Result<bool, String> {
        if let Some(child) = &mut self.child {
            if child.try_wait().map_err(|e| e.to_string())?.is_some() {
                return Ok(true);
            }
            return Ok(false);
        }
        Ok(self.run.as_ref().is_some_and(|run| files::ended(run)))
    }
    pub async fn poll(&mut self) -> Status {
        self.problem = self.refresh().await.err();
        self.status()
    }
    async fn refresh(&mut self) -> Result<(), String> {
        self.discover()?;
        if self.terminal()? {
            self.client = None;
            if self.closing {
                let _lock = files::lock(&self.root)?;
                if files::read_run(&self.root)? == self.run {
                    files::clear(&self.root)?;
                }
                self.run = None;
                self.child = None;
                self.closing = false;
                return Ok(());
            }
            return Err("后台进程已结束，可以重新载入节目".into());
        }
        if self.client.is_none() {
            if let Some(run) = &self.run {
                self.client = Some(Client::open(&run.join("host/discovery.json")).await?);
            }
        } else if let Some(client) = &mut self.client {
            client.maintain().await?;
        }
        Ok(())
    }
    pub fn status(&self) -> Status {
        Status {
            phase: if self.run.is_none() {
                "empty"
            } else if self.closing {
                "closing"
            } else if self.client.is_some() {
                "connected"
            } else {
                "unavailable"
            },
            problem: self.problem.clone(),
            runtime: self.client.as_ref().map(Client::view),
        }
    }
    pub async fn prepare(
        &mut self,
        document: Document,
        selection: Vec<Selection>,
        audio: Option<super::media::AudioInput>,
    ) -> Result<Status, String> {
        self.discover()?;
        let _lock = files::lock(&self.root)?;
        // Recheck shared record under the cross-process lock; never infer death from HTTP silence.
        if let Some(run) = files::read_run(&self.root)? {
            if self.run.as_ref() != Some(&run) {
                self.child = None;
                self.client = None;
            }
            self.run = Some(run);
            if !self.terminal()? {
                return Err("已有后台记录，请先连接并关闭原后台".into());
            }
            files::clear(&self.root)?;
        }
        // The manager lock prevents cooperating editor starts while the child takes ownership.
        // A prior editor keeps its voice reservation after the command guard has been dropped.
        if audio.is_some() {
            drop(stagemaster_audio::OutputScope::new(self.root.clone())?.reserve()?);
        }
        self.client = None;
        self.child = None;
        self.run = None;
        self.closing = false;
        let run = process::prepare(&self.root, &document, selection, audio)?;
        self.run = Some(run.clone());
        match process::launch(&self.binary, &run, &self.root) {
            Ok(child) => self.child = Some(child),
            Err(error) => {
                files::clear(&self.root)?;
                self.run = None;
                return Err(error);
            }
        }
        for _ in 0..40 {
            let status = self.poll().await;
            if self.client.is_some() || self.terminal()? {
                return Ok(status);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        self.problem = Some("后台仍在准备或连接未确认；请刷新原后台，不要重复载入".into());
        Ok(self.status())
    }
    pub async fn reconnect(&mut self) -> Status {
        self.client = None;
        self.poll().await
    }
    pub async fn acquire(&mut self, takeover: bool) -> Result<Status, String> {
        self.client
            .as_mut()
            .ok_or("请先连接后台")?
            .acquire(takeover)
            .await?;
        Ok(self.status())
    }
    pub async fn release(&mut self) -> Result<Status, String> {
        self.client
            .as_mut()
            .ok_or("请先连接后台")?
            .release()
            .await?;
        Ok(self.status())
    }
    pub async fn apply(
        &mut self,
        host: &str,
        revision: &str,
        source: &str,
        action: Action,
    ) -> Result<Status, String> {
        if self.closing {
            return Err("后台正在关闭".into());
        }
        self.client
            .as_mut()
            .ok_or("请先连接后台")?
            .apply(host, revision, source, action)
            .await?;
        Ok(self.status())
    }
    pub async fn output(
        &mut self,
        host: &str,
        revision: &str,
        action: stagemaster_execution_client::OutputAction,
    ) -> Result<Status, String> {
        if self.closing {
            return Err("后台正在关闭".into());
        }
        self.client
            .as_mut()
            .ok_or("请先连接后台")?
            .output(host, revision, action)
            .await?;
        Ok(self.status())
    }
    pub async fn shutdown(&mut self, host: &str) -> Result<Status, String> {
        let client = self.client.as_ref().ok_or("请先连接后台并核对状态")?;
        if client.view().host_id != host {
            return Err("后台已更换，请重新核对".into());
        }
        self.closing = true;
        client.shutdown().await?;
        for _ in 0..30 {
            if self.terminal()? {
                let _lock = files::lock(&self.root)?;
                if files::read_run(&self.root)? == self.run {
                    files::clear(&self.root)?;
                }
                self.client = None;
                self.child = None;
                self.run = None;
                self.closing = false;
                return Ok(self.status());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        self.problem = Some("已请求关闭，正在等待原进程结束".into());
        Ok(self.status())
    }
}

#[cfg(test)]
#[path = "capture_tests.rs"]
mod capture_tests;
#[cfg(test)]
#[path = "tests.rs"]
pub(super) mod tests;

#[cfg(test)]
#[path = "media_tests.rs"]
mod media_tests;

#[cfg(test)]
#[path = "audio_ownership_tests.rs"]
mod audio_ownership_tests;
