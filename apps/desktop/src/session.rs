use serde::Serialize;
use stagemaster_project::{Document, EditCommand, ProjectView};
use stagemaster_project_store::DiskFile;
use tauri::Manager;
mod audio;
#[cfg(test)]
mod audio_tests;
mod files;
mod output;
mod previs;
#[cfg(test)]
mod sequence_group_tests;

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
    output_control: crate::output_control::Control,
    audio: crate::audio::AudioPreview,
    previs_source: crate::previs::protocol::Source,
    previs_edit_allowed: bool,
    pub(crate) recovery: crate::recovery::Status,
    recovery_source: Option<String>,
    recent_problem: Option<String>,
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
    recovery: crate::recovery::Status,
    recent_problem: Option<String>,
}
impl Session {
    pub(crate) fn checkpoint(&self) -> crate::recovery::Checkpoint {
        crate::recovery::Checkpoint {
            generation: self.generation,
            document: self.document.clone().filter(|_| self.dirty()),
            source_file: self
                .file
                .as_ref()
                .map(|file| file.path().to_string_lossy().into_owned())
                .or_else(|| self.recovery_source.clone()),
        }
    }

    pub(crate) fn recover(
        &mut self,
        app: &tauri::AppHandle,
        generation: u32,
        id: &str,
        token: &str,
    ) -> Result<(), String> {
        self.guard(generation)?;
        let service = app.state::<crate::recovery::Service>();
        let candidate = service.claim(id, token)?;
        if self.allow_replace(app)? {
            let document = candidate.document.clone();
            let source = candidate.source_file.clone();
            service.take_claim(candidate)?;
            self.replace(document, None);
            self.recovery_source = source;
        }
        Ok(())
    }
    pub(crate) fn check_snapshot(&self, generation: u32) -> Result<Document, String> {
        self.guard(generation)?;
        self.document
            .clone()
            .ok_or_else(|| "请先新建或打开工程".into())
    }
    pub(crate) fn export_source(
        &self,
        generation: u32,
    ) -> Result<Option<std::path::PathBuf>, String> {
        self.guard(generation)?;
        if self.document.is_none() {
            return Err("请先打开工程".into());
        }
        Ok(self.file.as_ref().map(|f| f.path().to_path_buf()))
    }

    pub(crate) fn preview(
        &mut self,
        request: crate::preview::Request,
    ) -> Result<crate::preview::Snapshot, String> {
        use crate::preview::Request;
        match request {
            Request::Snapshot => {}
            Request::LoadScene {
                generation,
                scene_id,
            } => {
                self.guard(generation)?;
                self.audio.clear();
                self.preview.load_scene(
                    self.document.as_ref().ok_or("请先打开工程")?,
                    self.content_version,
                    &scene_id,
                )?;
            }
            Request::Load {
                generation,
                sequence_id,
            } => {
                self.guard(generation)?;
                self.audio.clear();
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
        self.preview.snapshot(
            self.content_version,
            self.preview.now(),
            self.output_control.master(),
        )
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
            recovery: self.recovery.clone(),
            recent_problem: self.recent_problem.clone(),
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
        self.recovery_source = None;
        self.recent_problem = None;
        self.undo.clear();
        self.redo.clear();
        self.preview.clear();
        self.audio.clear();
        self.output_control.reset();
        self.previs_source = crate::previs::protocol::Source::default();
        self.previs_edit_allowed = false;
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
            self.audio.synchronize(&next);
            self.document = Some(next);
            self.previs_edit_allowed = false;
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
            self.audio.transport.pause();
            if let Some(current) = self.document.take() {
                next.use_revision_from(&current);
                destination.push(current);
            }
            self.audio.synchronize(&next);
            self.document = Some(next);
            self.previs_edit_allowed = false;
            self.content_version += 1;
            self.generation += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "session/tests.rs"]
mod tests;
