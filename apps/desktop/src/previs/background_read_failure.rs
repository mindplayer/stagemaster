//! Failure provenance stays inside the desktop adapter, not the public Reader API.
#[derive(Debug)]
pub(super) enum ReadFailure {
    Busy(String),
    Refused(String),
}
impl ReadFailure {
    pub(super) fn from_reader(message: String) -> Self {
        if message == "后台请求未成功（503），请核对连接与原回执" {
            Self::Busy(message)
        } else {
            Self::Refused(message)
        }
    }
    pub(super) fn message(&self) -> &str {
        match self {
            Self::Busy(message) | Self::Refused(message) => message,
        }
    }
}
