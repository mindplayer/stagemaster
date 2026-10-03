use crate::{audio, execution, recent, recovery};
use tauri::Manager;

pub(crate) fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let data = recovery::directory(app)?
        .parent()
        .ok_or("缺少数据目录")?
        .to_path_buf();
    std::fs::create_dir_all(&data)?;
    app.state::<crate::previs::SharedSession>()
        .lock()
        .map_err(|_| "工程会话不可用")?
        .configure_audio_output(stagemaster_audio::OutputScope::new(data.join("execution"))?)?;
    app.manage(execution::Service::new(
        app.handle(),
        data.join("execution"),
    ));
    app.manage(audio::Service::new(data.join("audio")));
    app.manage(recent::Service::new(data.join("navigation")));
    app.manage(recovery::Service::new(recovery::directory(app)?));
    Ok(())
}
