use serde::Serialize;
use stagemaster_project::{Document, EditCommand, ProjectView};
use stagemaster_project_store::DiskFile;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

#[derive(Default)]
pub(crate) struct Session {
    document: Option<Document>,
    saved: Option<Document>,
    file: Option<DiskFile>,
    undo: Vec<Document>,
    redo: Vec<Document>,
    generation: u32,
    content_version: u64,
    preview: crate::preview::Preview,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    generation: u32,
    project: Option<ProjectView>,
    file_name: Option<String>,
    dirty: bool,
    can_undo: bool,
    can_redo: bool,
}
impl Session {
    pub(crate) fn preview(
        &mut self,
        request: crate::preview::Request,
    ) -> Result<crate::preview::Snapshot, String> {
        use crate::preview::Request;
        match request {
            Request::Snapshot => {}
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
        self.preview
            .snapshot(self.content_version, self.preview.now())
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            generation: self.generation,
            project: self.document.as_ref().map(Document::view),
            file_name: self
                .file
                .as_ref()
                .map(|f| f.path().to_string_lossy().into_owned()),
            dirty: self.dirty(),
            can_undo: !self.undo.is_empty(),
            can_redo: !self.redo.is_empty(),
        }
    }
    fn dirty(&self) -> bool {
        self.document.as_ref().is_some_and(|doc| {
            self.saved
                .as_ref()
                .is_none_or(|saved| !doc.same_content(saved))
        })
    }
    fn guard(&self, generation: u32) -> Result<(), String> {
        if generation == self.generation {
            Ok(())
        } else {
            Err("工程状态已变化，请重试当前操作".into())
        }
    }
    fn replace(&mut self, doc: Document, file: Option<DiskFile>) {
        self.saved = file.as_ref().map(|_| doc.clone());
        self.document = Some(doc);
        self.file = file;
        self.undo.clear();
        self.redo.clear();
        self.preview.clear();
        self.content_version += 1;
        self.generation += 1;
    }
    pub(crate) fn create(&mut self, app: &tauri::AppHandle, generation: u32) -> Result<(), String> {
        self.guard(generation)?;
        if self.allow_replace(app)? {
            self.replace(Document::new("未命名工程")?, None);
        }
        Ok(())
    }
    pub(crate) fn open(&mut self, app: &tauri::AppHandle, generation: u32) -> Result<(), String> {
        self.guard(generation)?;
        let Some(selected) = app
            .dialog()
            .file()
            .set_title("打开工程")
            .add_filter("舞台大师工程", &["json"])
            .blocking_pick_file()
        else {
            return Ok(());
        };
        let path = selected.into_path().map_err(|_| "请选择本机工程文件")?;
        let (doc, file) = DiskFile::open(&path)?;
        if self.allow_replace(app)? {
            // If the user saved the active document to this same file during the prompt,
            // reread it instead of installing the stale pre-dialog snapshot.
            let (doc, file) = if self
                .file
                .as_ref()
                .is_some_and(|current| current.path() == file.path())
            {
                DiskFile::open(file.path())?
            } else {
                (doc, file)
            };
            self.replace(doc, Some(file));
        }
        Ok(())
    }
    pub(crate) fn edit(&mut self, generation: u32, command: EditCommand) -> Result<(), String> {
        self.guard(generation)?;
        let current = self.document.as_ref().ok_or("请先新建或打开工程")?;
        let mut next = current.clone();
        next.edit(command)?;
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
            self.document = Some(next);
            self.content_version += 1;
            self.generation += 1;
        }
        Ok(())
    }
    pub(crate) fn history(&mut self, generation: u32, redo: bool) -> Result<(), String> {
        self.guard(generation)?;
        let (source, destination) = if redo {
            (&mut self.redo, &mut self.undo)
        } else {
            (&mut self.undo, &mut self.redo)
        };
        if let Some(mut next) = source.pop() {
            if let Some(current) = self.document.take() {
                next.use_revision_from(&current);
                destination.push(current);
            }
            self.document = Some(next);
            self.content_version += 1;
            self.generation += 1;
        }
        Ok(())
    }
    pub(crate) fn save(
        &mut self,
        app: &tauri::AppHandle,
        generation: u32,
        save_as: bool,
    ) -> Result<bool, String> {
        self.guard(generation)?;
        let document = self.document.as_ref().ok_or("请先新建或打开工程")?;
        let mut new_file = None;
        if save_as || self.file.is_none() {
            let Some(selected) = app
                .dialog()
                .file()
                .set_title("保存工程")
                .set_file_name("工程.project.json")
                .add_filter("舞台大师工程", &["json"])
                .blocking_save_file()
            else {
                return Ok(false);
            };
            let path = selected.into_path().map_err(|_| "请选择本机保存位置")?;
            let selected = DiskFile::select(&path)?;
            // Same-path Save As must preserve the original conflict baseline.
            if self
                .file
                .as_ref()
                .is_none_or(|current| current.path() != selected.path())
            {
                new_file = Some(selected);
            }
        }
        let target = new_file
            .as_mut()
            .or(self.file.as_mut())
            .ok_or("没有保存位置")?;
        let receipt = target.save(document)?;
        self.saved = Some(receipt.document.clone());
        self.document = Some(receipt.document);
        if let Some(file) = new_file {
            self.file = Some(file);
        }
        self.generation += 1;
        if let Some(warning) = receipt.warning {
            app.dialog()
                .message(warning)
                .parent(&app.get_webview_window("main").ok_or("主窗口已关闭")?)
                .title("保存状态")
                .buttons(MessageDialogButtons::OkCustom("知道了".into()))
                .blocking_show();
        }
        Ok(true)
    }
    pub(crate) fn allow_replace(&mut self, app: &tauri::AppHandle) -> Result<bool, String> {
        if !self.dirty() {
            return Ok(true);
        }
        let result = app
            .dialog()
            .message("当前工程有未保存的修改。")
            .parent(&app.get_webview_window("main").ok_or("主窗口已关闭")?)
            .title("保存工程？")
            .buttons(MessageDialogButtons::YesNoCancelCustom(
                "保存".into(),
                "不保存".into(),
                "取消".into(),
            ))
            .blocking_show_with_result();
        match result {
            MessageDialogResult::Yes => self.save(app, self.generation, false),
            MessageDialogResult::No => Ok(true),
            MessageDialogResult::Custom(label) if label == "保存" => {
                self.save(app, self.generation, false)
            }
            MessageDialogResult::Custom(label) if label == "不保存" => Ok(true),
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}

#[cfg(test)]
mod preview_integration_tests {
    use super::*;
    fn loaded(session: &mut Session) -> serde_json::Value {
        serde_json::to_value(session.preview(crate::preview::Request::Snapshot).unwrap()).unwrap()["loaded"].clone()
    }
    #[test]
    fn preview_is_not_history_and_only_content_changes_invalidate_it() {
        let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../docs/project-format/examples/lighting-basic.project.json"
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
            "../../../docs/project-format/examples/lighting-basic.project.json"
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
