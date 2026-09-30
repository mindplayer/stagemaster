use super::*;
#[cfg(test)]
mod history {
    use super::*;
    mod capacity_fixture {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/stagemaster-project/tests/support/capacity.rs"
        ));
    }
    fn session() -> Session {
        let mut session = Session::default();
        session.replace(Document::new("工程").unwrap(), None);
        session
    }
    fn rename(name: &str) -> EditCommand {
        EditCommand::SetInfo {
            name: name.into(),
            description: String::new(),
        }
    }
    #[test]
    fn batch_is_one_history_step_and_failure_keeps_redo_and_generation() {
        let mut s = session();
        let original = s.document.clone();
        s.edit(
            s.generation,
            EditCommand::Batch {
                commands: vec![rename("一"), rename("二")],
            },
        )
        .unwrap();
        assert_eq!(s.undo.len(), 1);
        s.history(s.generation, false).unwrap();
        assert_eq!(s.document, original);
        let generation = s.generation;
        assert!(
            s.edit(
                generation,
                EditCommand::Batch {
                    commands: vec![rename("三"), rename(" ")]
                }
            )
            .is_err()
        );
        assert_eq!(s.generation, generation);
        assert_eq!(s.redo.len(), 1);
        s.history(s.generation, true).unwrap();
        assert_eq!(s.document.as_ref().unwrap().view().name, "二");
    }
    #[test]
    fn stale_edits_and_failed_edits_never_replace_active_document() {
        let mut s = session();
        let before = s.document.clone();
        assert!(s.edit(0, rename("陈旧命令")).is_err());
        assert_eq!(s.document, before);
        assert!(s.edit(s.generation, rename(" ")).is_err());
        assert_eq!(s.document, before);
        assert!(s.undo.is_empty());
    }
    #[test]
    fn undo_redo_tracks_saved_content_and_saved_revision() {
        let mut s = session();
        s.saved = s.document.clone();
        assert!(!s.dirty());
        s.edit(s.generation, rename("改名")).unwrap();
        assert!(s.dirty());
        s.history(s.generation, false).unwrap();
        assert!(!s.dirty());
        s.history(s.generation, true).unwrap();
        assert!(s.dirty());
        let saved = s.document.as_ref().unwrap().next_revision();
        s.document = Some(saved.clone());
        s.saved = Some(saved.clone());
        s.history(s.generation, false).unwrap();
        assert!(s.dirty());
        s.history(s.generation, true).unwrap();
        assert!(!s.dirty());
        assert_eq!(s.document, Some(saved));
    }
    #[test]
    fn no_op_does_not_make_history_and_new_edit_discards_redo() {
        let mut s = session();
        let generation = s.generation;
        s.edit(generation, rename("工程")).unwrap();
        assert_eq!(s.generation, generation);
        assert!(s.undo.is_empty());
        s.edit(generation, rename("一")).unwrap();
        s.history(s.generation, false).unwrap();
        assert!(!s.redo.is_empty());
        s.edit(s.generation, rename("二")).unwrap();
        assert!(s.redo.is_empty());
    }
    #[test]
    fn capacity_failure_preserves_history_generation_and_recovery_snapshot() {
        let root = capacity_fixture::root_of_size(stagemaster_project::MAX_BYTES - 38 - 6);
        let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
        let mut s = Session::default();
        s.replace(doc, None);
        let description = |value: &str| EditCommand::SetInfo {
            name: "容量验收".into(),
            description: value.into(),
        };
        s.edit(s.generation, description("中文")).unwrap();
        assert_eq!(s.undo.len(), 1);
        s.history(s.generation, false).unwrap();
        let before = s.document.clone();
        let generation = s.generation;
        let version = s.content_version;
        let checkpoint = s.checkpoint().document;
        assert!(
            s.edit(generation, description("中文x"))
                .unwrap_err()
                .contains("8 MiB")
        );
        assert_eq!(s.document, before);
        assert_eq!(s.generation, generation);
        assert_eq!(s.content_version, version);
        assert_eq!(s.checkpoint().document, checkpoint);
        assert!(s.undo.is_empty());
        assert_eq!(s.redo.len(), 1);
        s.history(generation, true).unwrap();
        let doc = s.document.as_ref().unwrap();
        assert_eq!(doc.view().description, "中文");
        assert_eq!(
            doc.next_revision().encode().unwrap().len(),
            stagemaster_project::MAX_BYTES
        );
    }
}

#[cfg(test)]
mod preview_integration_tests {
    use super::*;
    #[test]
    fn restored_document_is_dirty_has_no_save_target_history_or_loaded_player() {
        let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        root["entryPoints"] = serde_json::json!([]);
        let document = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
        let mut session = Session::default();
        session.replace(document.clone(), None);
        session
            .preview(crate::preview::Request::LoadScene {
                generation: session.generation,
                scene_id: document.view().scenes[0].id.clone(),
            })
            .unwrap();
        session
            .edit(
                session.generation,
                EditCommand::SetInfo {
                    name: "编辑中".into(),
                    description: String::new(),
                },
            )
            .unwrap();
        session.history(session.generation, false).unwrap();
        assert!(!session.redo.is_empty());
        // This is the same replacement path used after the recovery claim succeeds.
        session.replace(document.clone(), None);
        assert!(session.dirty());
        assert!(session.file.is_none());
        assert!(session.saved.is_none());
        assert!(session.undo.is_empty() && session.redo.is_empty());
        assert!(loaded(&mut session).is_null());
        assert_eq!(session.checkpoint().document, Some(document));
        session.recovery_source = Some("仅作来源说明.json".into());
        assert_eq!(
            session.checkpoint().source_file.as_deref(),
            Some("仅作来源说明.json")
        );
        assert!(session.file.is_none());
    }
    #[test]
    fn check_captures_a_read_only_generation_without_replacing_preview_or_history() {
        let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        root["entryPoints"] = serde_json::json!([]);
        let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
        let mut s = Session::default();
        let id = doc.view().sequences[0].id.clone();
        s.replace(doc, None);
        s.preview(crate::preview::Request::Load {
            generation: s.generation,
            sequence_id: id,
        })
        .unwrap();
        let before = serde_json::to_value(s.snapshot()).unwrap();
        let preview = loaded(&mut s);
        assert!(s.check_snapshot(s.generation - 1).is_err());
        let captured = s.check_snapshot(s.generation).unwrap();
        assert!(captured.check().desktop_ready);
        assert_eq!(before, serde_json::to_value(s.snapshot()).unwrap());
        assert_eq!(preview, loaded(&mut s));
        s.edit(
            s.generation,
            EditCommand::SetInfo {
                name: "后续编辑".into(),
                description: String::new(),
            },
        )
        .unwrap();
        assert_ne!(
            s.document.as_ref().unwrap().view().name,
            captured.view().name
        );
        assert!(captured.check().desktop_ready);
        assert_eq!(s.undo.len(), 1);
    }
    #[test]
    fn effect_and_color_base_edit_undo_as_one_atomic_scene_change() {
        use serde_json::json;
        let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        root["entryPoints"] = json!([]);
        let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
        let view = doc.view();
        let scene = &view.scenes[0].id;
        let fixture = &view.fixtures[0].id;
        let mut s = Session::default();
        s.replace(doc.clone(), None);
        s.preview(crate::preview::Request::LoadScene {
            generation: s.generation,
            scene_id: scene.clone(),
        })
        .unwrap();
        let command = json!({"op":"batch","commands":[
            {"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,"attribute":"dimmer","mode":"literal","value":65535},
            {"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{
                "id":"29999999-0000-4000-8000-000000000001","name":"呼吸","enabled":true,"fixtureIds":[fixture],
                "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"smooth","dutyPercent":25,
                "channels":[{"attribute":"dimmer","low":0,"high":65535}]}}}]});
        s.edit(s.generation, serde_json::from_value(command).unwrap())
            .unwrap();
        assert_eq!(s.undo.len(), 1);
        assert_eq!(loaded(&mut s)["stale"], true);
        let changed = s.document.clone().unwrap();
        s.history(s.generation, false).unwrap();
        assert_eq!(s.document.as_ref(), Some(&doc));
        s.history(s.generation, true).unwrap();
        assert_eq!(s.document.as_ref(), Some(&changed));
    }
    fn loaded(session: &mut Session) -> serde_json::Value {
        serde_json::to_value(session.preview(crate::preview::Request::Snapshot).unwrap()).unwrap()["loaded"].clone()
    }
    #[test]
    fn preview_is_not_history_and_only_content_changes_invalidate_it() {
        let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        root["entryPoints"] = serde_json::json!([]);
        let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
        let id = doc.view().sequences[0].id.clone();
        let mut session = Session::default();
        session.replace(doc, None);
        let generation = session.generation;
        session
            .preview(crate::preview::Request::Load {
                generation,
                sequence_id: id,
            })
            .unwrap();
        assert!(session.undo.is_empty());
        assert_eq!(session.generation, generation);
        assert_eq!(loaded(&mut session)["stale"], false);
        let name = session.document.as_ref().unwrap().view().name;
        let description = session.document.as_ref().unwrap().view().description;
        session
            .edit(generation, EditCommand::SetInfo { name, description })
            .unwrap();
        assert_eq!(loaded(&mut session)["stale"], false);
        session
            .edit(
                generation,
                EditCommand::SetInfo {
                    name: "新名".into(),
                    description: String::new(),
                },
            )
            .unwrap();
        assert_eq!(loaded(&mut session)["stale"], true);
        session.history(session.generation, false).unwrap();
        assert_eq!(loaded(&mut session)["stale"], true);
        session.replace(Document::new("新工程").unwrap(), None);
        assert!(loaded(&mut session).is_null());
    }
}

#[cfg(test)]
mod library_history_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn library_transactions_restore_references_and_invalidate_preview_as_one_history_step() {
        let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/project-format/examples/lighting-basic.project.json"
        ))
        .unwrap();
        root["entryPoints"] = json!([]);
        let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
        let sequence = doc.view().sequences[0].id.clone();
        let preset = doc.view().presets[0].id.clone();
        let before = serde_json::to_value(doc.view()).unwrap();
        let mut s = Session::default();
        s.replace(doc, None);
        s.preview(crate::preview::Request::Load {
            generation: s.generation,
            sequence_id: sequence,
        })
        .unwrap();
        let remove = |keep| {
            serde_json::from_value(json!({"op":"library","command":{"kind":"remove","resource":"preset","id":preset,"keepValues":keep}})).unwrap()
        };
        assert!(s.edit(s.generation, remove(false)).is_err());
        assert!(s.undo.is_empty());
        s.edit(s.generation, remove(true)).unwrap();
        assert_eq!(s.undo.len(), 1);
        let preview =
            serde_json::to_value(s.preview(crate::preview::Request::Snapshot).unwrap()).unwrap();
        assert_eq!(preview["loaded"]["stale"], true);
        assert!(s.document.as_ref().unwrap().view().presets.is_empty());
        s.history(s.generation, false).unwrap();
        assert_eq!(
            serde_json::to_value(s.document.as_ref().unwrap().view()).unwrap(),
            before
        );
        s.history(s.generation, true).unwrap();
        assert!(s.document.as_ref().unwrap().view().presets.is_empty());
    }
}

#[cfg(test)]
mod stage_history_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn space_edit_undo_redo_and_invalid_geometry_keep_one_authoritative_document() {
        let mut s = Session::default();
        s.replace(Document::new("布置").unwrap(), None);
        let empty = s.document.clone();
        let room=serde_json::from_value(json!({"op":"stage","command":{"op":"putSpace","id":null,"name":"厅","outlineMeters":[["0","0"],["8","0"],["8","6"],["0","6"]],"floorElevationMeters":"0","clearHeightMeters":"5"}})).unwrap();
        s.edit(s.generation, room).unwrap();
        let created = s.document.clone();
        assert_eq!(s.undo.len(), 1);
        let id = s.document.as_ref().unwrap().view().stage.spaces[0]
            .id
            .clone();
        let invalid=serde_json::from_value(json!({"op":"stage","command":{"op":"putSpace","id":id,"name":"坏轮廓","outlineMeters":[["0","0"],["4","4"],["0","4"],["4","0"]],"floorElevationMeters":"0","clearHeightMeters":"5"}})).unwrap();
        assert!(s.edit(s.generation, invalid).is_err());
        assert_eq!(s.document, created);
        assert_eq!(s.undo.len(), 1);
        s.history(s.generation, false).unwrap();
        assert_eq!(s.document, empty);
        s.history(s.generation, true).unwrap();
        assert_eq!(s.document, created);
        let document = s.document.as_ref().unwrap();
        assert_eq!(
            document,
            &Document::decode(&document.encode().unwrap()).unwrap()
        );
    }
}

#[cfg(test)]
mod fixture_history_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn exchange_keeps_programming_and_repatches_as_one_history_event() {
        let mut doc = Document::new("换灯历史").unwrap();
        let base = doc.view();
        doc.edit(serde_json::from_value(json!({"op":"addFixture","name":"灯","profileId":base.profiles[0].id,"domainId":base.domains[0].id,"universe":1,"address":1})).unwrap()).unwrap();
        doc.edit(serde_json::from_value(json!({"op":"addScene","name":"场景"})).unwrap())
            .unwrap();
        doc.edit(serde_json::from_value(json!({"op":"fixture","command":{"op":"saveProfile","definition":{"name":"16 位","manufacturer":"测试","model":"P","mode":"双通道","footprint":2,"channels":[{"attribute":"dimmer","coarse":2,"fine":1,"defaultValue":0}]}}})).unwrap()).unwrap();
        let view = doc.view();
        let before = doc.clone();
        let mut s = Session::default();
        s.replace(doc, None);
        s.preview(crate::preview::Request::LoadScene {
            generation: s.generation,
            scene_id: view.scenes[0].id.clone(),
        })
        .unwrap();
        let exchange=serde_json::from_value(json!({"op":"fixture","command":{"op":"exchange","fixtureIds":[view.fixtures[0].id],"profileId":view.profiles.last().unwrap().id,"layout":{"universe":1,"address":511,"gap":0}}})).unwrap();
        s.edit(s.generation, exchange).unwrap();
        assert_eq!(s.undo.len(), 1);
        let after = s.document.clone();
        assert_eq!(
            after.as_ref().unwrap().view().fixtures[0].address,
            Some(511)
        );
        let preview =
            serde_json::to_value(s.preview(crate::preview::Request::Snapshot).unwrap()).unwrap();
        assert_eq!(preview["loaded"]["stale"], true);
        s.history(s.generation, false).unwrap();
        assert_eq!(s.document, Some(before));
        s.history(s.generation, true).unwrap();
        assert_eq!(s.document, after);
    }
}
