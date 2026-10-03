use super::{OutputKind, output::SoftwareOutput};
use crate::directory::Directory;
use stagemaster_audio::{Resources, Transport};
use stagemaster_live::media::{GroupSpec, Limits};
use stagemaster_project::Document;
use stagemaster_time::Clock;
use std::{path::Path, sync::atomic::AtomicBool};
use uuid::Uuid;

pub(crate) struct Setup {
    pub group: GroupSpec,
    pub duration_ms: u64,
    pub output: OutputKind,
    pub(super) transport: Transport,
    pub(super) software: Option<SoftwareOutput>,
}
impl Setup {
    pub fn prepare(
        doc: &Document,
        project: &Path,
        directory: &Directory,
        source: [u8; 16],
        output: OutputKind,
    ) -> Result<Self, String> {
        let track = doc.audio_timeline().ok_or("工程没有音乐轨道")?;
        if track.loop_regions.iter().any(|r| r.enabled) {
            return Err("独立后台暂未接入演出循环，请保留在编辑预演中执行".into());
        }
        let resources = Resources::new(directory.store().join("media"));
        let original =
            resources.resolve(&track.asset.digest, &track.asset.extension, Some(project))?;
        let cancel = AtomicBool::new(false);
        let (_, path) = resources.import(
            &original,
            &track.asset.extension,
            Some(&track.asset.digest),
            &cancel,
        )?;
        let (mut transport, software) = SoftwareOutput::prepare(output);
        let prepared = transport
            .load_performance_request(path, track.in_ms, track.out_ms, None)?
            .prepare(&cancel)?;
        transport.apply_load(prepared)?;
        Ok(Self {
            group: GroupSpec {
                id: source,
                clock: Clock::new(*Uuid::new_v4().as_bytes(), 1).map_err(|e| e.to_string())?,
                sources: vec![source],
                limits: Limits {
                    max_age_ns: 500_000_000,
                    max_uncertainty_ns: 1_000_000,
                    max_gap_ms: 500,
                    max_rate_percent: 400,
                    position_tolerance_ms: 2,
                },
            },
            duration_ms: track.duration_ms(),
            output,
            transport,
            software,
        })
    }
}
