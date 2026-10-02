use super::{Catalog, Entry, projection, wire::identity};
use crate::preparation::Prepared;
use serde::Deserialize;
use serde_json::json;
use stagemaster_live::{Session, SourceSpec};
use stagemaster_live_host::{Live, LiveBackend};
use stagemaster_project::PackageSelection;
use stagemaster_project_store::DiskFile;
use stagemaster_runtime_host::{Configuration, Host};
use std::{fs::File, io::Read, path::Path};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u8,
    sources: Vec<Source>,
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
}
pub(crate) fn prepare(project: &Path, manifest: &Path) -> Result<Prepared<Live>, String> {
    let mut bytes = Vec::new();
    File::open(manifest)
        .map_err(|e| e.to_string())?
        .take(65_537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65_536 {
        return Err("来源清单超过 64 KiB".into());
    }
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|_| "来源清单格式无效")?;
    if manifest.version != 1 || !(1..=64).contains(&manifest.sources.len()) {
        return Err("来源清单版本或数量不受支持".into());
    }
    let specs = manifest
        .sources
        .iter()
        .map(|s| {
            let id = identity(&s.id).map_err(|_| "来源标识无效")?;
            let playback = match &s.selection {
                Selection::Scene { id } => Some(PackageSelection::Scene { id: id.clone() }),
                Selection::Sequence { id } => Some(PackageSelection::Sequence { id: id.clone() }),
                Selection::Manual {} => None,
            };
            Ok(SourceSpec {
                id: *id.as_bytes(),
                priority: s.priority,
                playback,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let (document, _) = DiskFile::open(project)?;
    let boot = Uuid::new_v4();
    let session = Session::prepare(&document, *boot.as_bytes(), &specs, 0)?;
    let layout = projection::identity(session.layout_id());
    let view = document.view();
    let mut entries = Vec::new();
    let mut description = Vec::new();
    let mut output = None;
    for (spec, info) in specs.iter().zip(session.sources()) {
        let compiled = match &spec.playback {
            Some(PackageSelection::Scene { id }) => Some(document.compile_scene(id)?),
            Some(PackageSelection::Sequence { id }) => Some(document.compile_sequence(id)?),
            None => None,
        };
        let (name, steps) = if let Some(compiled) = compiled {
            if output.is_none() {
                output = Some(compiled.output);
            }
            (compiled.name, compiled.steps)
        } else {
            ("手动编程器".into(), Vec::new())
        };
        description.push(json!({"id":Uuid::from_bytes(info.id).to_string(),"name":name,"priority":spec.priority,
            "selection":spec.playback.as_ref().map_or_else(||json!({"kind":"manual"}),|s|json!(s)),"steps":steps}));
        entries.push(Entry {
            id: Uuid::from_bytes(info.id),
            key: info.key,
            manual: spec.playback.is_none(),
            steps,
        });
    }
    let source = json!({"mode":"softwareOutput","physicalOutput":false,"execution":"sourceGroup","protocol":2,
        "layout":layout,"projectId":view.id,"sources":description,"fixtures":view.fixtures,
        "limits":{"sources":64,"manualChanges":512,"requestBytes":8192,"commandTtlMs":5000,"outputUniverses":1},
        "capabilities":["sourcePlayback","sourceLevel","semanticManualPatch","preparedProject"]});
    let adapter = Catalog {
        entries,
        output: output.ok_or("来源组缺少节目")?,
        project: document.encode()?,
    };
    let host = Host::start_backend(
        LiveBackend::new(session).map_err(|e| e.to_string())?,
        Configuration::default(),
    )
    .map_err(|e| e.to_string())?;
    Ok(Prepared {
        host,
        adapter,
        source,
        boot,
    })
}
