use super::{Catalog, Entry, projection};
use serde_json::{Value, json};
use stagemaster_live::{PlaybackSelection, Session, SourceSpec};
use stagemaster_project::{Document, PackageSelection};
use uuid::Uuid;
pub(super) fn build(
    document: &Document,
    specs: &[SourceSpec],
    session: &Session,
) -> Result<(Catalog, Value), String> {
    let layout = projection::identity(session.layout_id());
    let view = document.view();
    let mut entries = Vec::new();
    let mut description = Vec::new();
    let mut output = None;
    for (spec, info) in specs.iter().zip(session.sources()) {
        let compiled = match spec.playback.as_ref().and_then(PlaybackSelection::program) {
            Some(PackageSelection::Scene { id }) => Some(document.compile_scene(id)?),
            Some(PackageSelection::Sequence { id }) => Some(document.compile_sequence(id)?),
            None => None,
        };
        let audio = matches!(spec.playback, Some(PlaybackSelection::AudioTimeline));
        let (name, steps) = if let Some(compiled) = compiled {
            if output.is_none() {
                output = Some(compiled.output);
            }
            (compiled.name, compiled.steps)
        } else if audio {
            if output.is_none() {
                output = Some(document.compile_audio_segment(None)?.output);
            }
            ("音乐灯光编排".into(), Vec::new())
        } else {
            ("手动编程器".into(), Vec::new())
        };
        let selection = if audio {
            json!({"kind":"audioTimeline"})
        } else {
            spec.playback
                .as_ref()
                .and_then(PlaybackSelection::program)
                .map_or_else(|| json!({"kind":"manual"}), |s| json!(s))
        };
        description.push(json!({"id":Uuid::from_bytes(info.id).to_string(),"name":name,"priority":spec.priority,"selection":selection,"steps":steps}));
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
        "capabilities":["sourcePlayback","sourceLevel","semanticManualPatch","preparedProject","sourceProgress","manualOwnership"]});
    Ok((
        Catalog {
            entries,
            output: output.ok_or("来源组缺少节目")?,
            project: document.encode()?,
            #[cfg(feature = "audio")]
            media: None,
        },
        source,
    ))
}
