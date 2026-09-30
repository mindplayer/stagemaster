use super::Session;
use stagemaster_project_store::DiskFile;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

impl Session {
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
        self.open_path(app, &path)
    }
    pub(crate) fn open_recent(
        &mut self,
        app: &tauri::AppHandle,
        generation: u32,
        id: &str,
    ) -> Result<(), String> {
        self.guard(generation)?;
        let path = app.state::<crate::recent::Service>().resolve(id)?;
        self.open_path(app, &path)
    }
    fn open_path(&mut self, app: &tauri::AppHandle, path: &std::path::Path) -> Result<(), String> {
        let (doc, file) = DiskFile::open(path)?;
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
            self.remember_file(app);
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
        let current_path = self.file.as_ref().map(|file| file.path().to_path_buf());
        let target = new_file
            .as_mut()
            .or(self.file.as_mut())
            .ok_or("没有保存位置")?;
        if let Some(track) = document.audio_timeline() {
            app.state::<crate::audio::Service>().resources.archive(
                &track.asset.digest,
                &track.asset.extension,
                current_path.as_deref(),
                target.path(),
            )?;
        }
        let receipt = target.save(document)?;
        self.saved = Some(receipt.document.clone());
        self.document = Some(receipt.document);
        if let Some(file) = new_file {
            self.file = Some(file);
        }
        self.remember_file(app);
        self.previs_edit_allowed = false;
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
    fn remember_file(&mut self, app: &tauri::AppHandle) {
        if let (Some(file), Some(document)) = (&self.file, &self.document) {
            self.recent_problem = app
                .state::<crate::recent::Service>()
                .remember(file.path(), &document.view().name)
                .err();
        }
    }
    pub(crate) fn allow_replace(&mut self, app: &tauri::AppHandle) -> Result<bool, String> {
        if !self.dirty() {
            app.state::<crate::recovery::Service>().retire()?;
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
        let allowed = match result {
            MessageDialogResult::Yes => self.save(app, self.generation, false),
            MessageDialogResult::No => Ok(true),
            MessageDialogResult::Custom(label) if label == "保存" => {
                self.save(app, self.generation, false)
            }
            MessageDialogResult::Custom(label) if label == "不保存" => Ok(true),
            _ => Ok(false),
        }?;
        if allowed {
            app.state::<crate::recovery::Service>().retire()?;
        }
        Ok(allowed)
    }
}
