//! Group movement uses the same revision and source gate as single-fixture placement.
use super::Session;
use crate::previs::protocol::Revision;
use stagemaster_project::{EditCommand, SpatialVector3, StageEdit};

impl Session {
    pub(crate) fn transform_from_viewport(
        &mut self,
        generation: u32,
        version: &str,
        fixture_ids: Vec<String>,
        yaw_degrees: String,
        spacing_scale: String,
    ) -> Result<Revision, String> {
        self.guard_viewport_edit(generation, version)?;
        self.edit(
            generation,
            EditCommand::Stage {
                command: StageEdit::TransformPlacements {
                    fixture_ids,
                    yaw_degrees,
                    spacing_scale,
                },
            },
        )?;
        self.previs_edit_allowed = false;
        Ok(self.previs_revision())
    }

    pub(crate) fn translate_from_viewport(
        &mut self,
        generation: u32,
        version: &str,
        fixture_ids: Vec<String>,
        delta_meters: SpatialVector3,
    ) -> Result<Revision, String> {
        self.guard_viewport_edit(generation, version)?;
        self.edit(
            generation,
            EditCommand::Stage {
                command: StageEdit::TranslatePlacements {
                    fixture_ids,
                    delta_meters,
                },
            },
        )?;
        self.previs_edit_allowed = false;
        Ok(self.previs_revision())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::previs::protocol::Source;
    use serde_json::json;
    use stagemaster_project::Document;

    fn setup() -> (Session, Vec<String>) {
        let mut document = Document::new("三维整组历史").unwrap();
        let view = document.view();
        for index in 0..2 {
            document
                .edit(EditCommand::AddFixture {
                    name: format!("灯 {index}"),
                    profile_id: view.profiles[0].id.clone(),
                    domain_id: view.domains[0].id.clone(),
                    universe: 1,
                    address: 1 + index * 4,
                })
                .unwrap();
        }
        let ids = document
            .view()
            .fixtures
            .iter()
            .map(|f| f.id.clone())
            .collect::<Vec<_>>();
        for (index, id) in ids.iter().enumerate() {
            document.edit(serde_json::from_value(json!({"op":"stage","command":{"op":"putPlacement","placement":{
                "fixtureId":id,"spaceId":null,"positionMeters":{"x":index.to_string(),"y":"2","z":(3+index).to_string()},
                "rotationDegreesXYZ":{"x":"25","y":"0","z":"0"}}}})).unwrap()).unwrap();
        }
        (Session::from_previs_test_document(document), ids)
    }
    fn delta() -> SpatialVector3 {
        SpatialVector3 {
            x: "-0.125".into(),
            y: "0".into(),
            z: "1.25".into(),
        }
    }
    #[test]
    fn group_translation_has_one_history_step_and_survives_round_trip() {
        let (mut session, ids) = setup();
        let original = session.previs_document().unwrap();
        let revision = session.previs_revision();
        session
            .translate_from_viewport(
                revision.generation,
                &revision.content.to_string(),
                ids.clone(),
                delta(),
            )
            .unwrap();
        let moved = session.previs_document().unwrap();
        assert_eq!(session.undo.len(), 1);
        assert_eq!(moved.view().stage.placements[0].position_meters.x, "-0.125");
        assert_eq!(moved.view().stage.placements[1].position_meters.z, "5.25");
        assert!(!session.previs_frame().unwrap().can_edit);
        assert!(
            session
                .translate_from_viewport(
                    revision.generation,
                    &revision.content.to_string(),
                    ids.clone(),
                    delta()
                )
                .is_err()
        );
        assert_eq!(session.undo.len(), 1);
        session.history(session.generation, false).unwrap();
        assert_eq!(session.previs_document().unwrap(), original);
        session.history(session.generation, true).unwrap();
        assert_eq!(session.previs_document().unwrap(), moved);
        assert_eq!(Document::decode(&moved.encode().unwrap()).unwrap(), moved);
        let revision = session.previs_revision();
        session
            .translate_from_viewport(
                revision.generation,
                &revision.content.to_string(),
                ids,
                SpatialVector3 {
                    x: "0".into(),
                    y: "0".into(),
                    z: "0".into(),
                },
            )
            .unwrap();
        assert_eq!(session.previs_document().unwrap(), moved);
        assert_eq!(session.undo.len(), 1);
    }
    #[test]
    fn stale_generation_version_and_source_reject_without_history() {
        let (mut session, ids) = setup();
        let original = session.previs_document().unwrap();
        let revision = session.previs_revision();
        assert!(
            session
                .translate_from_viewport(
                    revision.generation + 1,
                    &revision.content.to_string(),
                    ids.clone(),
                    delta()
                )
                .is_err()
        );
        assert!(
            session
                .translate_from_viewport(
                    revision.generation,
                    &(revision.content + 1).to_string(),
                    ids.clone(),
                    delta()
                )
                .is_err()
        );
        for source in [
            Source::Scene {
                scene_id: "missing-scene".into(),
            },
            Source::Playback,
        ] {
            session.previs_source = source;
            assert!(
                session
                    .translate_from_viewport(
                        revision.generation,
                        &revision.content.to_string(),
                        ids.clone(),
                        delta()
                    )
                    .is_err()
            );
            assert_eq!(session.previs_document().unwrap(), original);
            assert!(session.undo.is_empty());
        }
    }
    #[test]
    fn one_locked_member_prevents_whole_translation_and_undo_restores_permission() {
        let (mut session, ids) = setup();
        session
            .edit(
                session.generation,
                serde_json::from_value(json!({"op":"stage","command":{
            "op":"setEditLocks","targets":[{"kind":"placement","targetId":ids[1]}],"locked":true}}))
                .unwrap(),
            )
            .unwrap();
        let locked = session.previs_document().unwrap();
        let revision = session.previs_revision();
        assert!(
            session
                .translate_from_viewport(
                    revision.generation,
                    &revision.content.to_string(),
                    ids.clone(),
                    delta()
                )
                .unwrap_err()
                .contains("锁定")
        );
        assert_eq!(session.previs_document().unwrap(), locked);
        assert_eq!(session.undo.len(), 1);
        session.history(session.generation, false).unwrap();
        let revision = session.previs_revision();
        session
            .translate_from_viewport(
                revision.generation,
                &revision.content.to_string(),
                ids,
                delta(),
            )
            .unwrap();
        assert_eq!(session.undo.len(), 1);
    }
    #[test]
    fn rotation_and_spacing_share_atomic_history_and_revision_gate() {
        let (mut session, ids) = setup();
        let before = session.previs_document().unwrap();
        let rev = session.previs_revision();
        session
            .transform_from_viewport(
                rev.generation,
                &rev.content.to_string(),
                ids.clone(),
                "90".into(),
                "2".into(),
            )
            .unwrap();
        let after = session.previs_document().unwrap();
        assert_eq!(after.view().stage.placements[0].position_meters.x, "0.5");
        assert_eq!(after.view().stage.placements[0].position_meters.y, "1");
        assert_eq!(
            after.view().stage.placements[0].rotation_degrees_xyz.z,
            "90"
        );
        assert_eq!(session.undo.len(), 1);
        assert!(
            session
                .transform_from_viewport(
                    rev.generation,
                    &rev.content.to_string(),
                    ids.clone(),
                    "90".into(),
                    "1".into()
                )
                .is_err()
        );
        session.history(session.generation, false).unwrap();
        assert_eq!(session.previs_document().unwrap(), before);
        session.history(session.generation, true).unwrap();
        assert_eq!(session.previs_document().unwrap(), after);
        let rev = session.previs_revision();
        session
            .transform_from_viewport(
                rev.generation,
                &rev.content.to_string(),
                ids.clone(),
                "360".into(),
                "1".into(),
            )
            .unwrap();
        assert_eq!(session.undo.len(), 1);
        session.previs_source = Source::Playback;
        assert!(
            session
                .transform_from_viewport(
                    rev.generation,
                    &rev.content.to_string(),
                    ids,
                    "10".into(),
                    "1".into()
                )
                .is_err()
        );
    }
}
