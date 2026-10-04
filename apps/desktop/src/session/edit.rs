//! Shared edit installation, history and playback invalidation for all authoring adapters.
use super::{Document, EditCommand, Session};
impl Session {
    pub(crate) fn merge_manual_scene(
        &mut self,
        generation: u32,
        merge: &stagemaster_project::ManualSceneMerge,
    ) -> Result<(), String> {
        self.guard(generation)?;
        let mut next = self.document.clone().ok_or("请先新建或打开工程")?;
        next.merge_manual_scene(merge)?;
        self.install_edit(next)
    }

    pub(crate) fn record_manual_scene(
        &mut self,
        generation: u32,
        capture: &stagemaster_project::ManualSceneCapture,
        name: &str,
    ) -> Result<(), String> {
        self.guard(generation)?;
        let mut next = self.document.clone().ok_or("请先新建或打开工程")?;
        next.record_manual_scene(capture, name)?;
        self.install_edit(next)
    }
    pub(crate) fn edit(&mut self, generation: u32, command: EditCommand) -> Result<(), String> {
        self.guard(generation)?;
        let mut next = self.document.clone().ok_or("请先新建或打开工程")?;
        next.edit(command)?;
        self.install_edit(next)
    }
    pub(crate) fn apply_effect_template(
        &mut self,
        generation: u32,
        review: stagemaster_project::EffectTemplateReview,
    ) -> Result<(), String> {
        self.guard(generation)?;
        let mut next = self.document.clone().ok_or("请先新建或打开工程")?;
        next.apply_effect_template(review)?;
        self.install_edit(next)
    }
    fn install_edit(&mut self, next: Document) -> Result<(), String> {
        let current = self.document.as_ref().ok_or("请先新建或打开工程")?;
        if next != *current {
            self.undo.push(current.clone());
            while self.undo.len() > 32
                || self
                    .undo
                    .iter()
                    .map(|doc| {
                        doc.encode()
                            .map_or(stagemaster_project::MAX_BYTES, |bytes| bytes.len())
                    })
                    .sum::<usize>()
                    > 16 * 1024 * 1024
            {
                self.undo.remove(0);
            }
            self.redo.clear();
            self.preview.clear_draft();
            self.audio.synchronize(&next);
            self.document = Some(next);
            self.previs_edit_allowed = false;
            self.content_version += 1;
            self.generation += 1;
        }
        Ok(())
    }
}
