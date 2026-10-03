use std::{fmt, io};
#[derive(Debug)]
pub enum Error {
    Bounds,
    Incomplete,
    Identity,
    Cancelled,
    Source,
    Network,
    Timeout,
    Allocation,
    Io(io::Error),
    Package(stagemaster_package::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bounds => f.write_str("节目包长度超出导入范围"),
            Self::Incomplete => f.write_str("节目包没有完整收齐"),
            Self::Identity => f.write_str("节目包与所选发布内容不一致"),
            Self::Cancelled => f.write_str("节目包导入已取消"),
            Self::Source => f.write_str("节目包来源或响应不支持"),
            Self::Network => f.write_str("节目包下载失败，请检查连接后重试"),
            Self::Timeout => f.write_str("节目包下载超时"),
            Self::Allocation => f.write_str("没有足够内存接收节目包"),
            Self::Io(error) => write!(f, "无法读取节目包：{error}"),
            Self::Package(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for Error {}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<stagemaster_package::Error> for Error {
    fn from(value: stagemaster_package::Error) -> Self {
        Self::Package(value)
    }
}
