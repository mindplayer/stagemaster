use super::wire::identity;
use crate::media::OutputKind;
use serde::Deserialize;
use stagemaster_live::{PlaybackSelection, SourceSpec};
use stagemaster_project::PackageSelection;
use std::{fs::File, io::Read, path::Path};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    version: u8,
    sources: Vec<Source>,
    pub audio: Option<Audio>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Audio {
    pub output: OutputKind,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    id: String,
    priority: i16,
    selection: Selection,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum Selection {
    Scene { id: String },
    Sequence { id: String },
    Manual {},
    AudioTimeline {},
}
impl Manifest {
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(|e| e.to_string())?
            .take(65_537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65_536 {
            return Err("来源清单超过 64 KiB".into());
        }
        let manifest: Self = serde_json::from_slice(&bytes).map_err(|_| "来源清单格式无效")?;
        let audio_count = manifest
            .sources
            .iter()
            .filter(|s| matches!(s.selection, Selection::AudioTimeline {}))
            .count();
        if !(1..=2).contains(&manifest.version)
            || !(1..=64).contains(&manifest.sources.len())
            || audio_count > 1
            || (manifest.version == 1 && (manifest.audio.is_some() || audio_count != 0))
            || (manifest.audio.is_some() != (audio_count == 1))
        {
            return Err("来源清单版本、数量或音频输出配置不受支持".into());
        }
        Ok(manifest)
    }
    pub fn specs(&self) -> Result<Vec<SourceSpec>, String> {
        self.sources
            .iter()
            .map(|s| {
                let id = identity(&s.id).map_err(|_| "来源标识无效")?;
                let playback = match &s.selection {
                    Selection::Scene { id } => {
                        Some(PackageSelection::Scene { id: id.clone() }.into())
                    }
                    Selection::Sequence { id } => {
                        Some(PackageSelection::Sequence { id: id.clone() }.into())
                    }
                    Selection::Manual {} => None,
                    Selection::AudioTimeline {} => Some(PlaybackSelection::AudioTimeline),
                };
                Ok(SourceSpec {
                    id: *id.as_bytes(),
                    priority: s.priority,
                    playback,
                })
            })
            .collect()
    }
}
