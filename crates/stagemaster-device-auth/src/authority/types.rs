use crate::{Address, Binding, Secret};
use core::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Connection(pub(super) NonZeroU32);
impl Connection {
    #[must_use]
    pub const fn epoch(self) -> NonZeroU32 {
        self.0
    }
}

/// Trusted stack observation, not a peer-declared field. Authenticated means LESC,
/// with legacy pairing disabled in the platform adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Security {
    Unencrypted,
    Encrypted,
    Authenticated,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Pairing,
    Resumed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Peer {
    address: Address,
    ltk: Secret,
    irk: Option<Secret>,
}
impl Peer {
    #[must_use]
    pub const fn new(address: Address, ltk: Secret, irk: Option<Secret>) -> Self {
        Self { address, ltk, irk }
    }
    #[must_use]
    pub fn from_binding(binding: &Binding) -> Self {
        Self::new(
            binding.address(),
            binding.ltk().clone(),
            binding.irk().cloned(),
        )
    }
    pub(super) fn matches(&self, binding: &Binding) -> bool {
        self.address == binding.address()
            && &self.ltk == binding.ltk()
            && self.irk.as_ref() == binding.irk()
    }
    pub(super) fn same_identity(&self, binding: &Binding) -> bool {
        self.address == binding.address()
            || self
                .irk
                .as_ref()
                .zip(binding.irk())
                .is_some_and(|(a, b)| a == b)
    }
    pub(super) fn binding(&self, principal: [u8; 16]) -> Result<Binding, crate::Code> {
        Binding::new(principal, self.address, self.ltk.clone(), self.irk.clone())
    }
}

/// Build only from the current stack event plus its actual connection security.
/// No wire decoder: possession must already have been verified by the BLE stack.
#[derive(Clone, Debug)]
pub struct Evidence {
    pub security: Security,
    pub bonded: bool,
    pub origin: Origin,
    pub peer: Peer,
}
impl Evidence {
    pub(super) fn valid(&self, origin: Origin) -> bool {
        self.security == Security::Authenticated && self.bonded && self.origin == origin
    }
}

/// Instantaneous authority. Check the owning Authority again before each dispatch;
/// downstream workers also need the live epoch revocation channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grant {
    pub(super) connection: Connection,
    pub(super) principal: [u8; 16],
    pub(super) session: [u8; 16],
}
impl Grant {
    #[must_use]
    pub const fn connection(self) -> Connection {
        self.connection
    }
    #[must_use]
    pub const fn principal(self) -> [u8; 16] {
        self.principal
    }
    #[must_use]
    pub const fn session(self) -> [u8; 16] {
        self.session
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unavailable,
    Invalid,
    Busy,
    Stale,
    Closed,
    Expired,
    Clock,
    Denied,
    PairingClosed,
    Attempts,
    Exhausted,
    Binding(crate::Code),
}
impl From<crate::Code> for Error {
    fn from(code: crate::Code) -> Self {
        Self::Binding(code)
    }
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Self::Binding(code) = self {
            return code.fmt(f);
        }
        f.write_str(match self {
            Self::Unavailable => "绑定档案尚未核验",
            Self::Invalid => "连接随机标识无效或重复",
            Self::Busy => "已有连接或绑定操作正在进行",
            Self::Stale => "结果不属于当前连接",
            Self::Closed => "当前连接权限已关闭",
            Self::Expired => "当前连接验证或保活已超时",
            Self::Clock => "设备计时异常，连接权限已关闭",
            Self::Denied => "当前连接未通过绑定认证",
            Self::PairingClosed => "请先在设备上开启绑定",
            Self::Attempts => "本次绑定尝试次数已用完",
            Self::Exhausted => "设备连接计数或计时范围已耗尽",
            Self::Binding(_) => unreachable!(),
        })
    }
}
impl core::error::Error for Error {}
