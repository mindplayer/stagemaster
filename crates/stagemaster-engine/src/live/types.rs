use stagemaster_domain::MixMode;
use std::fmt;

pub type LayoutId = [u8; 32];
pub const MAX_ATTRIBUTES: usize = 512;
pub const MAX_SOURCES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub default: u16,
    pub mix: MixMode,
    pub intensity: bool,
    pub discrete: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    pub(super) id: LayoutId,
    pub(super) attributes: Vec<Attribute>,
}
impl Layout {
    /// # Errors
    /// Reject missing identity, oversized layouts and unsafe discrete mixing rules.
    pub fn new(id: LayoutId, attributes: Vec<Attribute>) -> Result<Self, Error> {
        if id == [0; 32] || attributes.is_empty() || attributes.len() > MAX_ATTRIBUTES {
            return Err(Error::Layout);
        }
        if attributes
            .iter()
            .any(|a| a.discrete && (a.intensity || a.mix != MixMode::LatestTakesPrecedence))
        {
            return Err(Error::Discrete);
        }
        Ok(Self { id, attributes })
    }
    #[must_use]
    pub const fn id(&self) -> LayoutId {
        self.id
    }
    #[must_use]
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Playback,
    Programmer,
    External,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub id: [u8; 16],
    pub kind: Kind,
}
/// Process-local capability. Never a peer-supplied identity or an output port permit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle {
    pub(super) boot: [u8; 16],
    pub(super) generation: u64,
    pub(super) slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceState {
    pub handle: Handle,
    pub source: Source,
    pub priority: i16,
    pub level: u16,
    pub serial: u64,
}
/// Full sparse-ownership snapshot: None withdraws this source's attribute.
/// An assertion advances LTP order; changing a sampled value alone does not.
#[derive(Clone, Copy)]
pub struct Frame<'a> {
    pub layout: LayoutId,
    pub serial: u64,
    pub values: &'a [Option<u16>],
    pub assert: &'a [bool],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Identity,
    Layout,
    Discrete,
    Budget,
    Allocation,
    Handle,
    Sequence,
    Shape,
    Assertion,
    Exhausted,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Identity => "混合器启动身份或来源身份无效／重复",
            Self::Layout => "贡献与当前工程属性布局不一致",
            Self::Discrete => "离散功能不支持亮度缩放或取高混合",
            Self::Budget => "混合器来源或属性数量超出当前预算",
            Self::Allocation => "无法准备属性混合缓冲",
            Self::Handle => "贡献来源已释放或被替换",
            Self::Sequence => "来源命令序号必须非零且严格递增",
            Self::Shape => "贡献或输出缓冲与属性数量不一致",
            Self::Assertion => "未控制的属性不能声明接管",
            Self::Exhausted => "来源或接管顺序已耗尽，不能回绕",
        })
    }
}
impl std::error::Error for Error {}
