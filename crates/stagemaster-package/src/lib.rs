//! Versioned playback archives, independent of projects, storage, transport and authorization.
//! `Archive::open` checks every block without allocating a plan. `load` loads only one program.
//! Readers must supply a stable, committed snapshot for the lifetime of an archive.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
mod archive;
mod archive_builder;
mod codec;
mod effect_codec;
mod program;

use alloc::{string::String, vec::Vec};
pub use archive::{Archive, Entry, ReadAt, Source};
pub use archive_builder::Builder;
use core::fmt;
pub use program::{decode_program, encode_program};
use stagemaster_playback::Plan;

pub const MAX_PACKAGE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_CATALOG_BYTES: usize = 16 * 1024;
pub const MAX_PROGRAM_BYTES: usize = 32 * 1024;
pub const MAX_LOADER_BYTES: usize = 64 * 1024;
pub const MAX_PROGRAMS: usize = 64;
pub const MAX_STEPS: usize = 128;
pub const MAX_TEXT_BYTES: usize = 512;
pub const PROFILE: &str = "reference-single-line-v1";
pub const COMPILER: &str = "stagemaster-lighting-1";
pub const SNAP_COMPILER: &str = "stagemaster-lighting-2";
pub type Id = [u8; 16];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    Limit(&'static str),
    Integrity,
    Version,
    Read,
    Allocation,
    Plan(String),
    AtProgram { index: usize, message: String },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(s) => write!(f, "播放包内容无效：{s}"),
            Self::Limit(s) => write!(f, "超出播放包参考限额：{s}"),
            Self::Integrity => f.write_str("播放包摘要不符，文件可能已损坏或改变"),
            Self::Version => f.write_str("不支持此播放包版本、编译语义或目标档位"),
            Self::Read => f.write_str("无法完整读取播放包"),
            Self::Allocation => f.write_str("没有足够内存装载播放包"),
            Self::Plan(s) => write!(f, "播放计划无效：{s}"),
            Self::AtProgram { index, message } => write!(f, "第 {} 个节目：{message}", index + 1),
        }
    }
}
impl core::error::Error for Error {}
impl From<minicbor::decode::Error> for Error {
    fn from(_: minicbor::decode::Error) -> Self {
        Self::Invalid("字段类型、长度或内容不符")
    }
}
impl From<minicbor::encode::Error<Error>> for Error {
    fn from(e: minicbor::encode::Error<Error>) -> Self {
        e.into_write().unwrap_or(Self::Invalid("无法编码字段"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Scene,
    Sequence,
}
impl Kind {
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::Scene => 0,
            Self::Sequence => 1,
        }
    }
    pub(crate) fn read(code: u8) -> Result<Self, Error> {
        match code {
            0 => Ok(Self::Scene),
            1 => Ok(Self::Sequence),
            _ => Err(Error::Invalid("节目类型")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mapping {
    pub coarse: u16,
    pub fine: Option<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepLabel {
    pub id: Id,
    pub name: String,
    pub number: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub output: Output,
    pub labels: Vec<StepLabel>,
    pub plan: Plan,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub universe: u16,
    pub mappings: Vec<Mapping>,
}
impl Output {
    /// Encode one complete output frame without allocation. Slots are one-based in the mapping.
    /// # Errors
    /// Rejects an invalid mapping or mismatched value shape, leaving the output unchanged.
    pub fn render(&self, values: &[u16], slots: &mut [u8; 512]) -> Result<(), Error> {
        if values.len() != self.mappings.len() {
            return Err(Error::Invalid("属性与输出映射数量不符"));
        }
        validate_mappings(&self.mappings)?;
        slots.fill(0);
        for (&value, m) in values.iter().zip(&self.mappings) {
            let [coarse, fine] = value.to_be_bytes();
            slots[usize::from(m.coarse) - 1] = coarse;
            if let Some(slot) = m.fine {
                slots[usize::from(slot) - 1] = fine;
            }
        }
        Ok(())
    }
}
pub(crate) fn occupy(used: &mut [bool; 512], slot: u16) -> Result<(), Error> {
    if !(1..=512).contains(&slot) {
        return Err(Error::Invalid("DMX 槽位须在 1–512 内"));
    }
    let index = usize::from(slot) - 1;
    if used[index] {
        return Err(Error::Invalid("DMX 槽位重叠"));
    }
    used[index] = true;
    Ok(())
}
fn validate_mappings(mappings: &[Mapping]) -> Result<(), Error> {
    if mappings.is_empty() || mappings.len() > 512 {
        return Err(Error::Limit("属性数须在 1–512 内"));
    }
    let mut used = [false; 512];
    for m in mappings {
        occupy(&mut used, m.coarse)?;
        if let Some(fine) = m.fine {
            occupy(&mut used, fine)?;
        }
    }
    Ok(())
}
/// Conservative 64-bit reference accounting, including input, catalogue and allocator allowances.
/// This is a software admission budget, not a measurement of an MCU allocator or wireless stack.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Usage {
    pub encoded_bytes: usize,
    pub attributes: usize,
    pub steps: usize,
    pub effect_channels: usize,
    pub snap_attributes: usize,
    pub keyframes: usize,
    pub value_bytes: usize,
    pub resident_bytes: usize,
    pub loader_peak_bytes: usize,
}
pub(crate) fn reserve<T>(count: usize) -> Result<Vec<T>, Error> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| Error::Allocation)?;
    Ok(values)
}
pub(crate) fn own(value: &str) -> Result<String, Error> {
    let mut text = String::new();
    text.try_reserve_exact(value.len())
        .map_err(|_| Error::Allocation)?;
    text.push_str(value);
    Ok(text)
}
