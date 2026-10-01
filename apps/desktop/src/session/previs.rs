use super::Session;
use crate::previs::protocol::{InputFrame, PlacementRequest, Revision, Source};
use stagemaster_project::{Document, EditCommand, FixturePlacement, StageEdit};

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
            Some(self.render_playback()?)
        } else {
            None
        };
        Ok(InputFrame {
            revision: self.previs_revision(),
            source: self.previs_source.clone(),
            can_edit: self.previs_edit_allowed,
            playback,
            master: self.output_control.master(),
        })
    }
    pub(crate) fn previs_place(&mut self, request: PlacementRequest) -> Result<Revision, String> {
        if !self.previs_edit_allowed {
            return Err("桌面有待应用修改，请先应用或取消后再移动三维灯位".into());
        }
        self.place_from_viewport(request.generation, &request.version, request.placement)
    }
    // Invoked only through the host's ordered project edit queue. No persistent HTTP grant.
    pub(crate) fn place_from_viewport(
        &mut self,
        generation: u32,
        version: &str,
        placement: FixturePlacement,
    ) -> Result<Revision, String> {
        self.guard_viewport_edit(generation, version)?;
        let view = self.document.as_ref().ok_or("请先打开工程")?.view();
        let previous = view
            .stage
            .placements
            .iter()
            .find(|p| p.fixture_id == placement.fixture_id)
            .ok_or("此灯具尚未布置，请先在桌面添加灯位")?;
        if previous.space_id != placement.space_id {
            return Err("请在桌面属性中修改灯位的空间归属".into());
        }
        let before = &previous.rotation_degrees_xyz;
        let after = &placement.rotation_degrees_xyz;
        if [&before.x, &before.y, &before.z] != [&after.x, &after.y, &after.z] {
            return Err("请在灯位属性中调整安装方向".into());
        }
        self.edit(
            generation,
            EditCommand::Stage {
                command: StageEdit::PutPlacement { placement },
            },
        )?;
        // The desktop must acknowledge the new generation before another remote edit.
        self.previs_edit_allowed = false;
        Ok(self.previs_revision())
    }
    pub(super) fn guard_viewport_edit(
        &mut self,
        generation: u32,
        version: &str,
    ) -> Result<(), String> {
        self.guard(generation)?;
        if version != self.content_version.to_string() {
            return Err("场地版本已变化，请刷新三维预演".into());
        }
        let doc = self.document.as_ref().ok_or("请先打开工程")?;
        let view = doc.view();
        match &self.previs_source.clone() {
            Source::Scene { scene_id }
                if !view.scenes.iter().any(|scene| scene.id == *scene_id) =>
            {
                return Err("原预演场景已删除，请重新选择".into());
            }
            Source::Playback if self.render_playback()?.output.is_none() => {
                return Err("请先重新载入场景列表预览".into());
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn session() -> Session {
        let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        root["entryPoints"] = json!([]);
        let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
        let id = doc.view().fixtures[0].id.clone();
        doc.edit(serde_json::from_value(json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":id,"spaceId":null,"positionMeters":{"x":"1","y":"2","z":"3"},"rotationDegreesXYZ":{"x":"25","y":"0","z":"0"}}}})).unwrap()).unwrap();
        Session::from_previs_test_document(doc)
    }
    fn placement(s: &Session) -> FixturePlacement {
        s.previs_document().unwrap().view().stage.placements[0].clone()
    }
    #[test]
    fn viewport_move_is_one_history_step_without_opening_http_editing() {
        let mut s = session();
        let original = s.previs_document().unwrap();
        let rev = s.previs_revision();
        let mut p = placement(&s);
        p.position_meters.x = "-0.125".into();
        s.place_from_viewport(rev.generation, &rev.content.to_string(), p.clone())
            .unwrap();
        let moved = s.previs_document().unwrap();
        assert!(!s.previs_frame().unwrap().can_edit);
        assert_eq!(s.undo.len(), 1);
        assert!(
            s.place_from_viewport(rev.generation, &rev.content.to_string(), p)
                .is_err()
        );
        assert_eq!(s.undo.len(), 1);
        s.history(s.generation, false).unwrap();
        assert_eq!(s.previs_document().unwrap(), original);
        s.history(s.generation, true).unwrap();
        assert_eq!(s.previs_document().unwrap(), moved);
        assert_eq!(Document::decode(&moved.encode().unwrap()).unwrap(), moved);
    }
    #[test]
    fn viewport_cannot_overwrite_newer_edits_or_change_installation_relationships() {
        let mut s = session();
        let rev = s.previs_revision();
        let original = s.previs_document().unwrap();
        for change in 0..4 {
            let mut p = placement(&s);
            match change {
                0 => p.rotation_degrees_xyz.x = "40".into(),
                1 => p.space_id = Some("another-space".into()),
                2 => p.fixture_id = "missing-fixture".into(),
                _ => p.position_meters.x = "NaN".into(),
            }
            assert!(
                s.place_from_viewport(rev.generation, &rev.content.to_string(), p)
                    .is_err()
            );
            assert_eq!(s.previs_document().unwrap(), original);
            assert!(s.undo.is_empty());
        }
        let mut old = placement(&s);
        old.position_meters.x = "9".into();
        s.edit(
            s.generation,
            EditCommand::SetInfo {
                name: "新名称".into(),
                description: String::new(),
            },
        )
        .unwrap();
        assert!(
            s.place_from_viewport(rev.generation, &rev.content.to_string(), old.clone())
                .is_err()
        );
        assert!(
            s.place_from_viewport(s.generation, &rev.content.to_string(), old)
                .is_err()
        );
        assert_eq!(placement(&s).position_meters.x, "1");
    }
    #[test]
    fn locked_fixture_rejects_native_viewport_and_lock_history_restores_permission() {
        let mut s = session();
        let id = placement(&s).fixture_id;
        let command = serde_json::from_value(json!({"op":"stage","command":{"op":"setEditLocks","targets":[{"kind":"placement","targetId":id}],"locked":true}})).unwrap();
        s.edit(s.generation, command).unwrap();
        let locked = s.previs_document().unwrap();
        let revision = s.previs_revision();
        let mut proposal = placement(&s);
        proposal.position_meters.x = "4".into();
        assert!(
            s.place_from_viewport(
                s.generation,
                &revision.content.to_string(),
                proposal.clone()
            )
            .unwrap_err()
            .contains("锁定")
        );
        assert_eq!(s.previs_document().unwrap(), locked);
        assert_eq!(s.undo.len(), 1);
        s.history(s.generation, false).unwrap();
        assert!(
            s.previs_document()
                .unwrap()
                .view()
                .stage
                .edit_locks
                .is_empty()
        );
        s.history(s.generation, true).unwrap();
        assert_eq!(s.previs_document().unwrap(), locked);
        s.history(s.generation, false).unwrap();
        let revision = s.previs_revision();
        s.place_from_viewport(s.generation, &revision.content.to_string(), proposal)
            .unwrap();
        assert_eq!(placement(&s).position_meters.x, "4");
    }
}
