use super::{
    Bridge, SharedSession, background,
    protocol::{Request, Source},
    session_access,
};
use tauri::Manager;

pub(super) async fn apply(
    app: &tauri::AppHandle,
    state: &Bridge,
    shared: &SharedSession,
    request: &Request,
) -> Result<(), String> {
    match request {
        Request::Background { generation } => {
            let path = app.state::<crate::execution::Service>().discovery_path()?;
            let binding = background::Background::connect(&path).await?;
            session_access::access(shared, |session| {
                let mut current = state.background.lock().map_err(|_| "后台观察不可用")?;
                session.set_previs_source(
                    *generation,
                    Source::Background {
                        host_id: binding.host_id.clone(),
                    },
                )?;
                current.install(binding)
            })
            .await?;
        }
        Request::Source { generation, source } => {
            if matches!(source, Source::Background { .. }) {
                return Err("请通过后台节目来源重新连接".into());
            }
            session_access::access(shared, |session| {
                session.set_previs_source(*generation, source.clone())
            })
            .await?;
        }
        Request::Editing {
            generation,
            allowed,
        } => {
            session_access::access(shared, |session| {
                session.set_previs_editing(*generation, *allowed)
            })
            .await?;
        }
        Request::Status | Request::Enable | Request::Disable => {}
    }
    Ok(())
}
