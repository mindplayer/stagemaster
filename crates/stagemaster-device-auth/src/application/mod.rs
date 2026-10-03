//! Application-key admission. Independent from OS bonds and file playback licenses.
mod configuration;
mod permit;
mod scopes;
mod session;
pub use configuration::{CONFIGURATION_BYTES, Configuration, Role};
pub use permit::{DevelopmentPermit, Grant, MAX_PERMISSION_MS};
pub use scopes::{Permissions, Scope};
pub use session::Session;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Denied,
    Expired,
    Closed,
    Clock,
    Secure(stagemaster_device_session::Error),
}
impl From<stagemaster_device_session::Error> for Error {
    fn from(value: stagemaster_device_session::Error) -> Self {
        Self::Secure(value)
    }
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Invalid => f.write_str("开发设备权限配置无效"),
            Self::Denied => f.write_str("当前控制端没有该设备的相应操作权限"),
            Self::Expired => f.write_str("本次设备操作权限已到期，请重新验证"),
            Self::Closed => f.write_str("本次设备操作权限已关闭"),
            Self::Clock => f.write_str("设备操作权限计时异常"),
            Self::Secure(value) => value.fmt(f),
        }
    }
}
impl core::error::Error for Error {}
