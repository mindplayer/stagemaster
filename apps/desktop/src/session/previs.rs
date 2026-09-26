use super::Session;
use crate::previs::protocol::{InputFrame, PlacementRequest, Revision, Source};
use stagemaster_project::{Document, EditCommand, StageEdit};

impl Session {
    #[cfg(test)]
    pub(crate) fn from_previs_test_document(document: Document) -> Self {
        let mut session = Self::default();
        session.replace(document, None);
        session
    }
    pub(crate) fn previs_revision(&self) -> Revision {
        Revision {
            generation: self.generation,
            content: self.content_version,
        }
    }
    pub(crate) fn previs_document(&self) -> Result<Document, String> {
        self.document
            .clone()
            .ok_or_else(|| "请先新建或打开工程".into())
    }
    pub(crate) fn previs_source(&self) -> Source {
        self.previs_source.clone()
    }
    pub(crate) fn set_previs_source(
        &mut self,
        generation: u32,
        source: Source,
    ) -> Result<(), String> {
        self.guard(generation)?;
        let document = self.document.as_ref().ok_or("请先新建或打开工程")?;
        if let Source::Scene { scene_id } = &source {
            stagemaster_previs::editing_lights(document, Some(scene_id))?;
        }
        self.previs_source = source;
        Ok(())
    }
    pub(crate) fn set_previs_editing(
        &mut self,
        generation: u32,
        allowed: bool,
    ) -> Result<(), String> {
        self.guard(generation)?;
        self.previs_edit_allowed = allowed;
        Ok(())
    }
    pub(crate) fn previs_frame(&mut self) -> Result<InputFrame, String> {
        let playback = if matches!(self.previs_source, Source::Playback) {
            Some(
                self.preview
                    .render_output(self.content_version, self.preview.now())?,
            )
        } else {
            None
        };
        Ok(InputFrame {
            revision: self.previs_revision(),
            source: self.previs_source.clone(),
            can_edit: self.previs_edit_allowed,
            playback,
        })
    }
    pub(crate) fn previs_place(&mut self, request: PlacementRequest) -> Result<Revision, String> {
        self.guard(request.generation)?;
        if request.version != self.content_version.to_string() {
            return Err("场地版本已变化，请刷新三维预演".into());
        }
        if !self.previs_edit_allowed {
            return Err("桌面有待应用修改，请先应用或取消后再移动三维灯位".into());
        }
        let doc = self.document.as_ref().ok_or("请先打开工程")?;
        let view = doc.view();
        match &self.previs_source {
            Source::Scene { scene_id }
                if !view.scenes.iter().any(|scene| scene.id == *scene_id) =>
            {
                return Err("原预演场景已删除，请重新选择".into());
            }
            Source::Playback
                if self
                    .preview
                    .render_output(self.content_version, self.preview.now())?
                    .output
                    .is_none() =>
            {
                return Err("请先重新载入场景列表预览".into());
            }
            _ => {}
        }
        let previous = view
            .stage
            .placements
            .iter()
            .find(|p| p.fixture_id == request.placement.fixture_id)
            .ok_or("此灯具尚未布置，请先在桌面添加灯位")?;
        if previous.space_id != request.placement.space_id {
            return Err("请在桌面属性中修改灯位的空间归属".into());
        }
        self.edit(
            request.generation,
            EditCommand::Stage {
                command: StageEdit::PutPlacement {
                    placement: request.placement,
                },
            },
        )?;
        // The desktop must acknowledge the new generation before another remote edit.
        self.previs_edit_allowed = false;
        Ok(self.previs_revision())
    }
}
