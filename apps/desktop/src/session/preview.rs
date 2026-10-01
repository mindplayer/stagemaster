use super::Session;
impl Session {
    pub(crate) fn preview(
        &mut self,
        request: crate::preview::Request,
    ) -> Result<crate::preview::Snapshot, String> {
        use crate::preview::Request;
        match request {
            Request::Snapshot => {}
            request @ (Request::BeginEffectDraft { .. } | Request::UpdateEffectDraft { .. }) => {
                // Synchronous path used by isolated session tests. The desktop command
                // prepares under lock, compiles outside, then finishes under lock.
                let prepared = self.prepare_effect_draft(request)?.compile()?;
                return self.finish_effect_draft(prepared);
            }
            Request::EndEffectDraft { epoch } => self.preview.end_draft(epoch)?,
            Request::LoadScene {
                generation,
                scene_id,
            } => {
                self.guard(generation)?;
                self.preview.load_scene(
                    self.document.as_ref().ok_or("请先打开工程")?,
                    self.content_version,
                    &scene_id,
                )?;
                self.audio.clear();
            }
            Request::Load {
                generation,
                sequence_id,
            } => {
                self.guard(generation)?;
                self.preview.load(
                    self.document.as_ref().ok_or("请先打开工程")?,
                    self.content_version,
                    &sequence_id,
                )?;
                self.audio.clear();
            }
            Request::Control {
                epoch,
                serial,
                command,
            } => {
                self.preview.control(
                    self.content_version,
                    epoch,
                    serial,
                    command,
                    self.preview.now(),
                )?;
            }
        }
        self.preview.snapshot(
            self.content_version,
            self.preview.now(),
            self.output_control.master(),
        )
    }
    pub(crate) fn prepare_effect_draft(
        &self,
        request: crate::preview::Request,
    ) -> Result<crate::preview::Preparation, String> {
        use crate::preview::Request;
        let (generation, epoch, scene_id, effect, illuminate, serial) = match request {
            Request::BeginEffectDraft {
                generation,
                epoch,
                scene_id,
                effect,
                illuminate,
            } => {
                self.guard(generation)?;
                self.preview.guard_epoch(epoch)?;
                (generation, epoch, scene_id, effect, illuminate, None)
            }
            Request::UpdateEffectDraft {
                generation,
                epoch,
                serial,
                effect,
                illuminate,
            } => {
                self.guard(generation)?;
                let scene_id =
                    self.preview
                        .draft_target(self.content_version, epoch, serial, &effect.id)?;
                (
                    generation,
                    epoch,
                    scene_id,
                    effect,
                    illuminate,
                    Some(serial),
                )
            }
            _ => return Err("此请求不是效果草稿编译".into()),
        };
        Ok(crate::preview::Preparation {
            document: self.document.clone().ok_or("请先打开工程")?,
            generation,
            version: self.content_version,
            epoch,
            scene_id,
            effect,
            illuminate,
            serial,
        })
    }
    pub(crate) fn finish_effect_draft(
        &mut self,
        prepared: crate::preview::Prepared,
    ) -> Result<crate::preview::Snapshot, String> {
        self.guard(prepared.generation)?;
        if self.preview.install_draft(prepared, self.content_version)? {
            self.audio.clear();
            self.previs_edit_allowed = false;
            self.previs_source = crate::previs::protocol::Source::Playback;
        }
        self.preview.snapshot(
            self.content_version,
            self.preview.now(),
            self.output_control.master(),
        )
    }
}
