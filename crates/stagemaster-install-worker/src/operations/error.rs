use stagemaster_runtime::Code;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Closed,
    Obsolete,
    Identity,
    Denied,
    Sequence,
    Clock,
    Runtime(Code),
    Protocol(stagemaster_runtime_protocol::Error),
    Negotiation,
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Closed => f.write_str("设备运行连接已关闭"),
            Self::Obsolete => f.write_str("设备操作权限已失效，请重新连接并读取状态"),
            Self::Identity => f.write_str("设备运行请求不属于当前连接或启动"),
            Self::Denied => f.write_str("当前连接没有此设备操作权限"),
            Self::Sequence => f.write_str("设备运行请求序号冲突，请重新读取状态"),
            Self::Clock => f.write_str("设备运行入口计时异常"),
            Self::Runtime(error) => error.fmt(f),
            Self::Protocol(error) => error.fmt(f),
            Self::Negotiation => f.write_str("设备运行协议尚未协商或协商顺序无效"),
        }
    }
}
impl core::error::Error for Error {}
