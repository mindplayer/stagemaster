//! Host presentation of validated declarations. No field grants a business session.
use crate::{Problem, ProblemCode};
use serde::Serialize;
use stagemaster_device_info::{Description, MODEL_WAVESHARE_ESP32_S3_RS485_CAN, capability as cap};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDescription {
    #[serde(skip)]
    pub(crate) installation_declared: bool,
    pub device_id: String,
    pub boot_id: String,
    pub model: u16,
    pub model_name: String,
    pub firmware: String,
    pub declared_functions: Vec<&'static str>,
    pub unknown_capabilities: u32,
    pub authentication_method: u16,
    pub limits: DeviceLimits,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceLimits {
    pub package_version: u16,
    pub execution_semantics: u16,
    pub transfer_version: u16,
    pub package_bytes: u32,
    pub programs: u16,
    pub universes: u16,
    pub message_bytes: u16,
    pub chunk_bytes: u16,
    pub slot_bytes: u32,
    pub loader_bytes: u32,
    pub frame_ms: u32,
}
impl DeviceDescription {
    pub(crate) fn decode(bytes: &[u8], session: u64) -> Result<Self, Problem> {
        let problem = |e: stagemaster_device_info::Error| {
            Problem::new(ProblemCode::Description).detail(e.to_string())
        };
        let value = Description::decode(bytes).map_err(problem)?;
        value.check_session(session).map_err(problem)?;
        let l = value.limits;
        Ok(Self {
            installation_declared: value.declares(cap::INSTALLATION),
            device_id: hex(&value.device),
            boot_id: hex(&value.boot),
            model: value.model,
            model_name: if value.model == MODEL_WAVESHARE_ESP32_S3_RS485_CAN {
                "微雪 ESP32-S3-RS485-CAN".into()
            } else {
                format!("未识别型号（{}）", value.model)
            },
            firmware: format!(
                "{}.{}.{}",
                value.firmware.major, value.firmware.minor, value.firmware.patch
            ),
            declared_functions: [
                (cap::DIAGNOSTICS, "连接诊断"),
                (cap::CATALOG, "节目目录"),
                (cap::PACKAGE_SEMANTICS_2, "离散功能节目"),
                (cap::INSTALLATION, "节目安装"),
                (cap::PLAYBACK, "节目播放"),
                (cap::RUNTIME_APPLICATION, "运行控制接口"),
                (cap::DMX_OUTPUT, "DMX 输出"),
            ]
            .into_iter()
            .filter_map(|(bit, label)| value.declares(bit).then_some(label))
            .collect(),
            unknown_capabilities: value.unknown_capabilities(),
            authentication_method: value.authentication,
            limits: DeviceLimits {
                package_version: l.package_version,
                execution_semantics: if value.declares(cap::PACKAGE_SEMANTICS_2) {
                    2
                } else {
                    u16::from(value.declares(cap::CATALOG))
                },
                transfer_version: l.transfer_version,
                package_bytes: l.package_bytes,
                programs: l.programs,
                universes: l.universes,
                message_bytes: l.message_bytes,
                chunk_bytes: l.chunk_bytes,
                slot_bytes: l.slot_bytes,
                loader_bytes: l.loader_bytes,
                frame_ms: l.frame_ms,
            },
        })
    }
}
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(result, "{byte:02x}");
    }
    result
}
