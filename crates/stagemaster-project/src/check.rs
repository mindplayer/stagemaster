//! Read-only diagnostics. Compilation remains the authority for playable programs.
use crate::{Document, ProjectView, text};
use serde::Serialize;
use stagemaster_playback::{self as playback, Plan};
use std::collections::BTreeSet;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CheckLocation {
    Fixtures,
    Scenes,
    Fixture { id: String },
    Placement { id: String },
    Scene { id: String },
    Sequence { id: String },
}

#[derive(Debug, Serialize)]
pub struct CheckIssue {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub location: CheckLocation,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProgramStatus {
    Passed,
    Failed,
    Blocked,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanUsage {
    pub attributes: usize,
    pub steps: usize,
    pub target_values: usize,
    pub effect_channels: usize,
    pub keyframes: usize,
    pub snap_buffer_bytes: usize,
    pub value_buffer_bytes: usize,
    pub effect_buffer_bytes: usize,
}
impl From<&Plan> for PlanUsage {
    fn from(plan: &Plan) -> Self {
        Self {
            attributes: plan.defaults().len(),
            steps: plan.steps().len(),
            target_values: plan.defaults().len() * plan.steps().len(),
            effect_channels: plan.effect_channel_count(),
            keyframes: plan.keyframe_count(),
            snap_buffer_bytes: plan.snap_buffer_bytes(),
            value_buffer_bytes: plan.value_buffer_bytes(),
            effect_buffer_bytes: plan.effect_buffer_bytes(),
        }
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanLimits {
    pub attributes: usize,
    pub steps: usize,
    pub target_values: usize,
    pub effect_channels: usize,
    pub keyframes: usize,
}
impl Default for PlanLimits {
    fn default() -> Self {
        Self {
            attributes: playback::MAX_ATTRIBUTES,
            steps: playback::MAX_STEPS,
            target_values: playback::MAX_TARGET_VALUES,
            effect_channels: playback::MAX_EFFECT_CHANNELS,
            keyframes: playback::MAX_KEYFRAMES,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct ProgramCheck {
    pub name: String,
    pub location: CheckLocation,
    pub status: ProgramStatus,
    pub usage: Option<PlanUsage>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub project_id: String,
    pub revision_id: String,
    pub desktop_ready: bool,
    pub issues: Vec<CheckIssue>,
    pub programs: Vec<ProgramCheck>,
    pub limits: PlanLimits,
}

impl Document {
    /// Check a validated snapshot without loading players, accessing files or changing history.
    /// Every program is compiled independently; only statistics outlive each temporary plan.
    #[must_use]
    pub fn check(&self) -> CheckReport {
        let view = self.view();
        let mut issues = output_issues(&view);
        let blocked = issues.iter().any(|i| i.severity == Severity::Error);
        let mut programs = Vec::new();
        for (id, name, is_sequence) in view
            .scenes
            .iter()
            .map(|s| (&s.id, &s.name, false))
            .chain(view.sequences.iter().map(|s| (&s.id, &s.name, true)))
        {
            let location = || {
                if is_sequence {
                    CheckLocation::Sequence { id: id.clone() }
                } else {
                    CheckLocation::Scene { id: id.clone() }
                }
            };
            let (status, usage) = if blocked {
                (ProgramStatus::Blocked, None)
            } else {
                let compiled = if is_sequence {
                    self.compile_sequence_view(id, &view)
                } else {
                    self.compile_scene_view(id, &view)
                };
                match compiled {
                    Ok(compiled) => (ProgramStatus::Passed, Some(PlanUsage::from(&compiled.plan))),
                    Err(message) => {
                        issues.push(issue(
                            "program.compile",
                            format!("“{name}”：{message}"),
                            location(),
                        ));
                        (ProgramStatus::Failed, None)
                    }
                }
            };
            programs.push(ProgramCheck {
                name: name.clone(),
                location: location(),
                status,
                usage,
            });
        }
        if programs.is_empty() {
            issues.push(issue(
                "program.empty",
                "请先创建要播放的场景",
                CheckLocation::Scenes,
            ));
        }
        // Blocking errors must not disappear behind pages of optional placement warnings.
        issues.sort_by_key(|issue| issue.severity == Severity::Warning);
        CheckReport {
            project_id: view.id,
            revision_id: text(&self.root["project"], "revisionId").into(),
            desktop_ready: !issues.iter().any(|i| i.severity == Severity::Error),
            issues,
            programs,
            limits: PlanLimits::default(),
        }
    }
}
fn issue(code: &'static str, message: impl Into<String>, location: CheckLocation) -> CheckIssue {
    CheckIssue {
        code,
        severity: Severity::Error,
        message: message.into(),
        location,
    }
}

pub(super) fn output_issues(view: &ProjectView) -> Vec<CheckIssue> {
    let mut issues = Vec::new();
    if view.fixtures.is_empty() {
        issues.push(issue(
            "patch.empty",
            "请先添加并配适灯具",
            CheckLocation::Fixtures,
        ));
    }
    let lines: BTreeSet<_> = view
        .fixtures
        .iter()
        .filter_map(|f| f.universe.map(|u| (f.domain_id.as_str(), u)))
        .collect();
    let placed: BTreeSet<_> = view
        .stage
        .placements
        .iter()
        .map(|p| p.fixture_id.as_str())
        .collect();
    for fixture in &view.fixtures {
        let location = || CheckLocation::Fixture {
            id: fixture.id.clone(),
        };
        match fixture.universe {
            None => {
                issues.push(issue(
                    "patch.missing",
                    format!("“{}”尚未配适，请设置线路与地址", fixture.name),
                    location(),
                ));
            }
            Some(universe) if lines.len() > 1 => {
                issues.push(issue(
                    "patch.multipleLines",
                    format!(
                        "“{}”位于 {} / 线路 {}；当前播放内核要求所有灯具位于同一输出域、同一线路",
                        fixture.name, fixture.domain_name, universe
                    ),
                    location(),
                ));
            }
            _ => {}
        }
        if !placed.contains(fixture.id.as_str()) {
            issues.push(CheckIssue {
                code: "stage.unplaced",
                severity: Severity::Warning,
                message: format!(
                    "“{}”尚未布置灯位，三维预演不会显示这台灯；不影响电脑播放",
                    fixture.name
                ),
                location: CheckLocation::Placement {
                    id: fixture.id.clone(),
                },
            });
        }
    }
    if view
        .fixtures
        .iter()
        .map(|f| f.attributes.len())
        .sum::<usize>()
        > playback::MAX_ATTRIBUTES
    {
        issues.push(issue(
            "plan.attributes",
            format!("工程属性总数超过当前计划上限 {}", playback::MAX_ATTRIBUTES),
            CheckLocation::Fixtures,
        ));
    }
    issues
}
