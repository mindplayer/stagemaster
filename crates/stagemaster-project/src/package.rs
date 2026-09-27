//! Host compilation adapter. Packages own no project editing, storage or transport concerns.
use crate::{CheckLocation, CompiledSequence, Document, Severity, text};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stagemaster_package::{Archive, Builder, Kind, Program, Source, StepLabel};
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum PackageSelection {
    Scene { id: String },
    Sequence { id: String },
}
impl PackageSelection {
    fn id(&self) -> &str {
        match self {
            Self::Scene { id } | Self::Sequence { id } => id,
        }
    }
    const fn kind(&self) -> Kind {
        match self {
            Self::Scene { .. } => Kind::Scene,
            Self::Sequence { .. } => Kind::Sequence,
        }
    }
    fn location(&self) -> CheckLocation {
        match self {
            Self::Scene { id } => CheckLocation::Scene { id: id.clone() },
            Self::Sequence { id } => CheckLocation::Sequence { id: id.clone() },
        }
    }
}
#[derive(Debug, Serialize)]
pub struct PackageIssue {
    pub message: String,
    pub location: Option<CheckLocation>,
}
impl PackageIssue {
    fn global(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageProgram {
    pub name: String,
    pub location: CheckLocation,
    pub encoded_bytes: usize,
    pub attributes: usize,
    pub steps: usize,
    pub effect_channels: usize,
    pub keyframes: usize,
    pub value_bytes: usize,
    pub resident_bytes: usize,
    pub loader_peak_bytes: usize,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageReport {
    pub project_id: String,
    pub revision_id: String,
    pub source_digest: String,
    pub package_digest: String,
    pub profile: &'static str,
    pub bytes: usize,
    pub universe: u16,
    pub catalog_resident_bytes: usize,
    pub max_loader_bytes: usize,
    pub programs: Vec<PackageProgram>,
}
pub struct PackageBuild {
    pub bytes: Vec<u8>,
    pub report: PackageReport,
}

impl Document {
    /// Compile a selection from the current complete snapshot; unsaved edits are included.
    /// No document mutation, external resources, physical output, storage or authorization.
    /// # Errors
    /// Returns all selected-program compile failures; never exports a partially successful selection.
    pub fn build_package(
        &self,
        selection: &[PackageSelection],
    ) -> Result<PackageBuild, Vec<PackageIssue>> {
        if selection.is_empty() || selection.len() > stagemaster_package::MAX_PROGRAMS {
            return Err(vec![PackageIssue::global("请选择 1–64 个场景或场景列表")]);
        }
        let mut unique = BTreeSet::new();
        for item in selection {
            if uuid(item.id()).is_err() || !unique.insert((item.kind(), item.id())) {
                return Err(vec![PackageIssue {
                    message: "所选节目标识无效或重复".into(),
                    location: Some(item.location()),
                }]);
            }
        }
        let view = self.view();
        let patch_issues: Vec<_> = crate::check::output_issues(&view)
            .into_iter()
            .filter(|issue| issue.severity == Severity::Error)
            .map(|issue| PackageIssue {
                message: issue.message,
                location: Some(issue.location),
            })
            .collect();
        if !patch_issues.is_empty() {
            return Err(patch_issues);
        }
        let snapshot_digest =
            snapshot_digest(&self.root).map_err(|e| vec![PackageIssue::global(e)])?;
        let revision = text(&self.root["project"], "revisionId");
        let source = (|| {
            Ok::<_, String>(Source {
                project_id: uuid(&view.id)?,
                revision_id: uuid(revision)?,
                snapshot_digest,
                project_name: view.name.clone(),
            })
        })()
        .map_err(|e| vec![PackageIssue::global(e)])?;
        let mut builder = Builder::new(source);
        let mut issues = Vec::new();
        for item in selection {
            let result = (|| {
                let compiled = match item {
                    PackageSelection::Scene { id } => self.compile_scene_view(id, &view)?,
                    PackageSelection::Sequence { id } => self.compile_sequence_view(id, &view)?,
                };
                let name = compiled.name.clone();
                let id = uuid(&compiled.id)?;
                let program = portable(compiled)?;
                builder
                    .add(item.kind(), id, &name, &program)
                    .map_err(|e| e.to_string())
            })();
            if let Err(message) = result {
                issues.push(PackageIssue {
                    message,
                    location: Some(item.location()),
                });
            }
        }
        if !issues.is_empty() {
            return Err(issues);
        }
        let (bytes, archive) = builder.finish().map_err(|error| {
            let location = if let stagemaster_package::Error::AtProgram { index, .. } = &error {
                let mut sorted = selection.iter().collect::<Vec<_>>();
                sorted.sort_by_key(|s| (s.kind(), uuid(s.id()).unwrap_or_default()));
                sorted.get(*index).map(|s| s.location())
            } else {
                None
            };
            vec![PackageIssue {
                message: error.to_string(),
                location,
            }]
        })?;
        Ok(PackageBuild {
            report: report(&archive),
            bytes,
        })
    }
}
fn uuid(value: &str) -> Result<[u8; 16], String> {
    Uuid::parse_str(value)
        .map(|id| *id.as_bytes())
        .map_err(|_| "工程标识无效".into())
}
fn portable(compiled: CompiledSequence) -> Result<Program, String> {
    let labels = compiled
        .steps
        .into_iter()
        .map(|s| {
            Ok(StepLabel {
                id: uuid(&s.id)?,
                name: s.name,
                number: s.number,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Program {
        output: compiled.output.portable_output()?,
        labels,
        plan: compiled.plan,
    })
}
fn report(archive: &Archive) -> PackageReport {
    use std::fmt::Write;
    let hex = |bytes: &[u8]| {
        let mut result = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            write!(result, "{byte:02x}").expect("write String");
        }
        result
    };
    let source = archive.source();
    PackageReport {
        project_id: Uuid::from_bytes(source.project_id).to_string(),
        revision_id: Uuid::from_bytes(source.revision_id).to_string(),
        source_digest: hex(&source.snapshot_digest),
        package_digest: hex(archive.digest()),
        profile: stagemaster_package::PROFILE,
        bytes: archive.total_bytes(),
        universe: archive.universe(),
        catalog_resident_bytes: archive.catalog_resident_bytes(),
        max_loader_bytes: stagemaster_package::MAX_LOADER_BYTES,
        programs: archive
            .entries()
            .iter()
            .map(|p| {
                let id = Uuid::from_bytes(p.id).to_string();
                PackageProgram {
                    name: p.name.clone(),
                    location: match p.kind {
                        Kind::Scene => CheckLocation::Scene { id },
                        Kind::Sequence => CheckLocation::Sequence { id },
                    },
                    encoded_bytes: p.usage.encoded_bytes,
                    attributes: p.usage.attributes,
                    steps: p.usage.steps,
                    effect_channels: p.usage.effect_channels,
                    keyframes: p.usage.keyframes,
                    value_bytes: p.usage.value_bytes,
                    resident_bytes: p.usage.resident_bytes,
                    loader_peak_bytes: p.usage.loader_peak_bytes,
                }
            })
            .collect(),
    }
}

// Hash the canonical compact snapshot without building another multi-megabyte JSON buffer.
// This intentionally does not use the pretty-printed file-save size admission path.
fn snapshot_digest(root: &serde_json::Value) -> Result<[u8; 32], String> {
    struct Sink(Sha256);
    impl std::io::Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut sink = Sink(Sha256::new());
    serde_json::to_writer(&mut sink, root).map_err(|_| "无法计算工程快照摘要".to_string())?;
    Ok(sink.0.finalize().into())
}
