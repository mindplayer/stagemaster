//! Typed viewport proposals reuse the authoritative mixed-object transaction.
use super::Session;
use crate::previs::protocol::Revision;
use serde::Deserialize;
use stagemaster_project::{EditCommand, SpatialVector3, StageEdit, StageEditLock, StageLockKind};

#[cfg(test)]
#[path = "previs_objects_tests.rs"]
mod tests;

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum ViewportTarget {
    Placement { id: String },
    Construction { id: String },
}
impl Session {
    pub(crate) fn translate_objects_from_viewport(
        &mut self,
        generation: u32,
        version: &str,
        targets: Vec<ViewportTarget>,
        delta_meters: SpatialVector3,
    ) -> Result<Revision, String> {
        self.guard_viewport_edit(generation, version)?;
        let targets = targets
            .into_iter()
            .map(|target| match target {
                ViewportTarget::Placement { id } => StageEditLock {
                    kind: StageLockKind::Placement,
                    target_id: id,
                },
                ViewportTarget::Construction { id } => StageEditLock {
                    kind: StageLockKind::Construction,
                    target_id: id,
                },
            })
            .collect();
        self.edit(
            generation,
            EditCommand::Stage {
                command: StageEdit::TranslateObjects {
                    targets,
                    delta_meters,
                },
            },
        )?;
        self.previs_edit_allowed = false;
        Ok(self.previs_revision())
    }
}
