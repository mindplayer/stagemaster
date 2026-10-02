//! Frozen scene and read-only output adapter. No controller or editor clock is retained.
use super::protocol::Revision;
use stagemaster_execution_client::{Reader, Selection};
use stagemaster_previs::{Light, LightRig, Scene};
use stagemaster_project::{Document, OutputObserver};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

pub(super) type SharedBackground = Arc<Mutex<Binding>>;
#[derive(Default)]
pub(super) struct Binding {
    pub current: Option<Arc<Background>>,
    next: u64,
}
impl Binding {
    pub fn install(&mut self, mut value: Background) -> Result<(), String> {
        let next = self
            .next
            .checked_add(1)
            .ok_or("观察版本已耗尽，请重启应用")?;
        value.revision.content = next;
        self.current = Some(Arc::new(value));
        self.next = next;
        Ok(())
    }
}
pub(super) struct Background {
    pub host_id: String,
    pub revision: Revision,
    pub scene: Scene,
    reader: tokio::sync::Mutex<Reader>,
    output: OutputObserver,
    rig: LightRig,
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Summary {
    project_name: String,
    unmodeled_fixtures: Vec<String>,
}
impl Background {
    pub fn summary(&self) -> Summary {
        Summary {
            project_name: self.scene.project_name.clone(),
            unmodeled_fixtures: self
                .scene
                .fixtures
                .iter()
                .filter(|f| f.light_simulation != "dimmer-rgb")
                .map(|f| f.name.clone())
                .collect(),
        }
    }
    pub async fn connect(path: &Path) -> Result<Self, String> {
        let reader = Reader::open(path).await?;
        let bytes = reader.project().await?;
        tokio::task::spawn_blocking(move || Self::prepare(reader, &bytes))
            .await
            .map_err(|_| "后台场地准备失败")?
    }
    fn prepare(reader: Reader, bytes: &[u8]) -> Result<Self, String> {
        let document = Document::decode(bytes)?;
        if document.view().id != reader.catalog().project_id {
            return Err("后台工程身份不一致".into());
        }
        let selection = reader
            .catalog()
            .sources
            .iter()
            .find_map(|source| match &source.selection {
                Selection::Scene { id } => Some(document.compile_scene(id)),
                Selection::Sequence { id } => Some(document.compile_sequence(id)),
                Selection::Manual {} => None,
            })
            .ok_or("后台没有可观察的节目")??;
        Ok(Self {
            host_id: reader.host_id().into(),
            revision: Revision {
                generation: 0,
                content: 0,
            },
            scene: stagemaster_previs::scene(&document)?,
            rig: LightRig::new(&document),
            output: selection.output.observer()?,
            reader: tokio::sync::Mutex::new(reader),
        })
    }
    pub async fn lights(&self) -> Result<Vec<Light>, String> {
        let frame = self
            .reader
            .try_lock()
            .map_err(|_| "后台观察正在读取")?
            .sample()
            .await?;
        let slots: &[u8; 512] = frame
            .slots
            .as_slice()
            .try_into()
            .map_err(|_| "后台采样不完整")?;
        let output = self.output.observe(frame.universe, slots)?;
        Ok(self.rig.playback(&output))
    }
}
