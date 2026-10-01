//! Fixture identities, addresses and world placement joined for crew handoff.
use super::csv_format::FORMAT;
use crate::report_csv::literal;
use crate::{
    FixturePlacement,
    view::{FixtureView, ProfileView, ProjectView},
};
use std::collections::BTreeMap;

pub(super) struct ReportRows<'a> {
    project: &'a ProjectView,
    revision: &'a str,
    digest: &'a str,
    profiles: BTreeMap<&'a String, &'a ProfileView>,
    placements: BTreeMap<&'a String, &'a FixturePlacement>,
    spaces: BTreeMap<&'a String, &'a String>,
    constructions: BTreeMap<&'a String, &'a String>,
    attachments: BTreeMap<&'a String, &'a String>,
    groups: BTreeMap<&'a String, Vec<&'a str>>,
}
impl<'a> ReportRows<'a> {
    pub(super) fn new(project: &'a ProjectView, revision: &'a str, digest: &'a str) -> Self {
        let profiles: BTreeMap<_, _> = project.profiles.iter().map(|p| (&p.id, p)).collect();
        let placements: BTreeMap<_, _> = project
            .stage
            .placements
            .iter()
            .map(|p| (&p.fixture_id, p))
            .collect();
        let spaces: BTreeMap<_, _> = project
            .stage
            .spaces
            .iter()
            .map(|s| (&s.id, &s.name))
            .collect();
        let constructions: BTreeMap<_, _> = project
            .stage
            .constructions
            .iter()
            .map(|c| (&c.id, &c.name))
            .collect();
        let attachments: BTreeMap<_, _> = project
            .stage
            .attachments
            .iter()
            .map(|a| (&a.fixture_id, &a.construction_id))
            .collect();
        let mut groups: BTreeMap<&String, Vec<&str>> = BTreeMap::new();
        for group in &project.groups {
            for id in &group.fixture_ids {
                groups.entry(id).or_default().push(&group.name);
            }
        }
        Self {
            project,
            revision,
            digest,
            profiles,
            placements,
            spaces,
            constructions,
            attachments,
            groups,
        }
    }
    pub(super) fn fixture(&self, f: &FixtureView) -> [String; 28] {
        let p = self.profiles[&f.profile_id];
        let placed = self.placements.get(&f.id);
        let space = placed
            .and_then(|p| p.space_id.as_ref())
            .and_then(|id| self.spaces.get(id));
        let support = self
            .attachments
            .get(&f.id)
            .and_then(|id| self.constructions.get(id));
        let position = placed.map(|p| &p.position_meters);
        let rotation = placed.map(|p| &p.rotation_degrees_xyz);
        [
            FORMAT.into(),
            literal(&self.project.name),
            self.project.id.clone(),
            self.revision.into(),
            self.digest.into(),
            f.id.clone(),
            literal(&f.name),
            literal(&p.manufacturer),
            literal(&p.model),
            literal(&p.mode),
            literal(&p.revision),
            literal(&f.domain_name),
            f.domain_id.clone(),
            number(f.universe),
            number(f.address),
            number(f.address.map(|a| a + f.footprint - 1)),
            f.footprint.to_string(),
            if f.address.is_some() {
                "已配适"
            } else {
                "未配适"
            }
            .into(),
            if placed.is_some() {
                "已布置"
            } else {
                "未布置"
            }
            .into(),
            space.map_or_else(
                || {
                    if placed.is_some() {
                        "未归属空间".into()
                    } else {
                        String::new()
                    }
                },
                |name| literal(name),
            ),
            support.map_or_else(String::new, |name| literal(name)),
            position.map_or_else(String::new, |v| v.x.clone()),
            position.map_or_else(String::new, |v| v.y.clone()),
            position.map_or_else(String::new, |v| v.z.clone()),
            rotation.map_or_else(String::new, |v| v.x.clone()),
            rotation.map_or_else(String::new, |v| v.y.clone()),
            rotation.map_or_else(String::new, |v| v.z.clone()),
            literal(
                &self
                    .groups
                    .get(&f.id)
                    .map_or_else(String::new, |names| names.join("；")),
            ),
        ]
    }
}
fn number(value: Option<u64>) -> String {
    value.map_or_else(String::new, |n| n.to_string())
}
