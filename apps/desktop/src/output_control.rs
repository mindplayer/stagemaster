//! Session-only output controls; independent of edits, playback loading and audio transport.
use serde::{Deserialize, Serialize};
use stagemaster_playback::OutputMaster;
use tauri::Manager;

#[derive(Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    Snapshot,
    Set {
        epoch: u32,
        serial: u32,
        percent: u8,
        blackout: bool,
    },
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    pub epoch: u32,
    pub serial: u32,
    pub percent: u8,
    pub blackout: bool,
    pub uncontrolled_fixtures: usize,
}
#[derive(Default)]
pub(crate) struct Control {
    epoch: u32,
    serial: u32,
    master: OutputMaster,
}
impl Control {
    pub fn reset(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.serial = 0;
        self.master = OutputMaster::default();
    }
    pub fn master(&self) -> OutputMaster {
        self.master
    }
    pub fn request(
        &mut self,
        request: Request,
        uncontrolled_fixtures: usize,
    ) -> Result<Snapshot, String> {
        if let Request::Set {
            epoch,
            serial,
            percent,
            blackout,
        } = request
        {
            if epoch != self.epoch {
                return Err("预演工程已更换，请确认总控状态后重试".into());
            }
            if serial <= self.serial {
                return Err("重复或乱序的总控操作已忽略".into());
            }
            self.master.set_percent(percent)?;
            self.master.set_blackout(blackout);
            self.serial = serial;
        }
        Ok(Snapshot {
            epoch: self.epoch,
            serial: self.serial,
            percent: self.master.percent(),
            blackout: self.master.blackout(),
            uncontrolled_fixtures,
        })
    }
}

#[tauri::command]
pub(crate) async fn output_request(
    app: tauri::AppHandle,
    request: Request,
) -> Result<Snapshot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<crate::previs::SharedSession>();
        let mut session = state.lock().map_err(|_| "工程会话发生错误，请重启应用")?;
        session.output_request(request)
    })
    .await
    .map_err(|_| "预演总控操作未完成".to_string())?
}
