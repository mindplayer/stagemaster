//! Device admission from bindings or application keys. No radio, UI or playback license.
#![no_std]
#![forbid(unsafe_code)]
#[cfg(feature = "application")]
pub mod application;
pub mod authority;
mod binding;
mod codec;
#[cfg(feature = "persistence")]
pub mod persistence;
mod vault;

pub use binding::{Address, Binding, LocalIdentity, Secret};
pub use codec::{RECORD_BYTES, SecretRecord};
pub use vault::{MAX_BINDINGS, Vault};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Invalid,
    Format,
    Duplicate,
    Full,
    Missing,
    Exhausted,
    Unavailable,
    Geometry,
    NotBlank,
    Conflict,
    Integrity,
}
impl core::fmt::Display for Code {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "绑定凭据无效",
            Self::Format => "绑定存储格式不支持或已损坏",
            Self::Duplicate => "绑定主体或身份重复",
            Self::Full => "绑定数量已达上限",
            Self::Missing => "绑定不存在",
            Self::Exhausted => "绑定修订号已耗尽",
            Self::Unavailable => "绑定状态尚未核验，请先恢复",
            Self::Geometry => "绑定存储区域或擦写粒度不支持",
            Self::NotBlank => "绑定存储不是空白区域，不能自动初始化",
            Self::Conflict => "绑定状态已变化，请重新核对",
            Self::Integrity => "绑定提交回读不一致",
        })
    }
}
impl core::error::Error for Code {}
