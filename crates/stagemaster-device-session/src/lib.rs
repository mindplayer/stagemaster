//! Transport-independent identity proof and confidential records, not business permission.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

mod channel;
mod context;
mod crypto;
mod handshake;
mod key;

pub use channel::{Channel, Kind, PeerProof, Record};
pub use context::Context;
pub use crypto::Entropy;
pub use handshake::{HANDSHAKE_BYTES, Handshake};
pub use key::SecretKey;

pub const AUTHENTICATION: u16 = 2;
pub const MAX_PAYLOAD: usize = 1280;
pub const PLAINTEXT_BYTES: usize = MAX_PAYLOAD + 1;
pub const CIPHERTEXT_BYTES: usize = PLAINTEXT_BYTES + 16;
pub const HANDSHAKE_MS: u64 = 10_000;
pub const LEASE_MS: u64 = 6_000;
pub const SUITE: &str = "Noise_IK_25519_ChaChaPoly_SHA256";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    State,
    Crypto,
    Entropy,
    Expired,
    Clock,
    Closed,
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "安全消息格式或长度无效",
            Self::State => "安全会话步骤不匹配",
            Self::Crypto => "设备安全验证失败",
            Self::Entropy => "安全随机源不可用",
            Self::Expired => "设备安全会话已超时",
            Self::Clock => "设备安全时钟不连续",
            Self::Closed => "设备安全会话已关闭",
        })
    }
}
impl core::error::Error for Error {}
