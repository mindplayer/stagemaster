use serde::Serialize;
use stagemaster_device_host::{Phase, Request};
use stagemaster_device_upload::Connection;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Destination {
    revision: u64,
    epoch: u32,
    name: Option<String>,
    device_id: Option<String>,
    allowed: bool,
    reason: Option<String>,
}
pub(super) fn read(connection: &crate::device::Connections) -> Result<Destination, String> {
    let state = connection
        .request(Request::Status)
        .map_err(|e| e.to_string())?;
    let availability = if state.phase == Phase::Connected {
        connection.target(state.epoch).map(|_| ())
    } else {
        Err("请先在设备面板连接播放设备".into())
    };
    Ok(Destination {
        revision: state.revision,
        epoch: state.epoch,
        name: state.selected.map(|d| d.name),
        device_id: state.description.map(|d| d.device_id),
        allowed: availability.is_ok(),
        reason: availability.err(),
    })
}
