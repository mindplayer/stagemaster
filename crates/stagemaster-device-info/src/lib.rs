//! Public device descriptions, never credentials or grants. No I/O or allocation.
#![no_std]
#![forbid(unsafe_code)]

pub const WIRE_BYTES: usize = 96;
pub const VERSION: u8 = 1;
pub const MODEL_WAVESHARE_ESP32_S3_RS485_CAN: u16 = 1;

pub mod capability {
    pub const DIAGNOSTICS: u32 = 1;
    pub const CATALOG: u32 = 1 << 1;
    pub const INSTALLATION: u32 = 1 << 2;
    pub const PLAYBACK: u32 = 1 << 3;
    pub const DMX_OUTPUT: u32 = 1 << 4;
    pub const KNOWN: u32 = DIAGNOSTICS | CATALOG | INSTALLATION | PLAYBACK | DMX_OUTPUT;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Length,
    Magic,
    Version,
    Reserved,
    Identity,
    Session,
    Capabilities,
    Limits,
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Length => "设备描述长度不正确",
            Self::Magic => "设备描述格式不正确",
            Self::Version => "设备描述版本不受支持",
            Self::Reserved => "设备描述包含不支持的保留字段",
            Self::Identity => "设备或启动标识无效",
            Self::Session => "设备描述不属于当前连接",
            Self::Capabilities => "设备能力声明相互矛盾",
            Self::Limits => "设备能力与容量声明不一致",
        })
    }
}
impl core::error::Error for Error {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Firmware {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

/// Hard limits of implemented capabilities, not current free space or permission.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Limits {
    pub package_version: u16,
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

/// Connection-scoped immutable metadata. Its identifiers are not authenticated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Description {
    pub device: [u8; 16],
    pub boot: [u8; 16],
    pub session: u64,
    /// Unknown model numbers are retained rather than misidentified as a known board.
    pub model: u16,
    pub firmware: Firmware,
    /// Unknown bits are retained, but callers may only use features they implement.
    pub capabilities: u32,
    /// A declared method number, NEVER evidence that a peer has authenticated.
    pub authentication: u16,
    pub limits: Limits,
}

/// Namespaced hardware identifier for the first ESP32 family adapter.
/// Read the factory MAC; do not use the rotating BLE address or burn any eFuse.
/// # Errors
/// Reject absent, erased or multicast values, which cannot identify this board.
pub fn esp32_device_id(factory_mac: [u8; 6]) -> Result<[u8; 16], Error> {
    if factory_mac == [0; 6] || factory_mac == [255; 6] || factory_mac[0] & 1 != 0 {
        return Err(Error::Identity);
    }
    let mut id = [0; 16];
    id[..10].copy_from_slice(b"SMESP32S3\0");
    id[10..].copy_from_slice(&factory_mac);
    Ok(id)
}

impl Description {
    #[must_use]
    pub const fn declares(&self, feature: u32) -> bool {
        feature != 0 && self.capabilities & feature == feature
    }

    #[must_use]
    pub const fn unknown_capabilities(&self) -> u32 {
        self.capabilities & !capability::KNOWN
    }

    /// Match the current successful diagnostic handshake. This is correlation, not trust.
    /// # Errors
    /// Cached metadata from a different connection must never identify the current peer.
    pub fn check_session(&self, expected: u64) -> Result<(), Error> {
        if expected == 0 || self.session != expected {
            Err(Error::Session)
        } else {
            Ok(())
        }
    }

    /// Check internal consistency without claiming support for future protocol versions.
    /// # Errors
    /// Reject zero identifiers and budgets that cannot describe the declared capabilities.
    pub fn validate(&self) -> Result<(), Error> {
        use capability::{CATALOG, DIAGNOSTICS, DMX_OUTPUT, INSTALLATION, PLAYBACK};
        if self.device == [0; 16] || self.boot == [0; 16] || self.model == 0 {
            return Err(Error::Identity);
        }
        self.check_session(self.session)?;
        let catalog = self.declares(CATALOG);
        let install = self.declares(INSTALLATION);
        let playback = self.declares(PLAYBACK);
        if !self.declares(DIAGNOSTICS)
            || ((install || playback) && !catalog)
            || (self.declares(DMX_OUTPUT) && !playback)
            || (install && self.authentication == 0)
        {
            return Err(Error::Capabilities);
        }
        let l = self.limits;
        if if catalog {
            l.package_version == 0 || l.package_bytes == 0 || l.programs == 0
        } else {
            l.package_version != 0 || l.package_bytes != 0 || l.programs != 0
        } {
            return Err(Error::Limits);
        }
        if if install {
            l.transfer_version == 0
                || l.message_bytes <= 8
                || l.chunk_bytes == 0
                || l.chunk_bytes > l.message_bytes - 8
                || l.slot_bytes < l.package_bytes
        } else {
            l.transfer_version != 0
                || l.message_bytes != 0
                || l.chunk_bytes != 0
                || l.slot_bytes != 0
        } {
            return Err(Error::Limits);
        }
        if if playback {
            l.universes == 0 || l.loader_bytes == 0 || l.frame_ms == 0
        } else {
            l.universes != 0 || l.loader_bytes != 0 || l.frame_ms != 0
        } {
            return Err(Error::Limits);
        }
        Ok(())
    }

    /// Produce the exact v1 wire representation; no uninitialized padding is transmitted.
    /// # Errors
    /// Invalid declarations cannot be advertised as a usable device.
    pub fn encode(&self) -> Result<[u8; WIRE_BYTES], Error> {
        self.validate()?;
        let mut bytes = [0; WIRE_BYTES];
        bytes[..4].copy_from_slice(b"SMDC");
        bytes[4] = VERSION;
        put16(&mut bytes, 6, 96);
        bytes[8..24].copy_from_slice(&self.device);
        bytes[24..40].copy_from_slice(&self.boot);
        bytes[40..48].copy_from_slice(&self.session.to_le_bytes());
        put16(&mut bytes, 48, self.model);
        put16(&mut bytes, 50, self.firmware.major);
        put16(&mut bytes, 52, self.firmware.minor);
        put16(&mut bytes, 54, self.firmware.patch);
        put32(&mut bytes, 56, self.capabilities);
        put16(&mut bytes, 60, self.limits.package_version);
        put16(&mut bytes, 62, self.limits.transfer_version);
        put32(&mut bytes, 64, self.limits.package_bytes);
        put16(&mut bytes, 68, self.limits.programs);
        put16(&mut bytes, 70, self.limits.universes);
        put16(&mut bytes, 72, self.limits.message_bytes);
        put16(&mut bytes, 74, self.limits.chunk_bytes);
        put32(&mut bytes, 76, self.limits.slot_bytes);
        put32(&mut bytes, 80, self.limits.loader_bytes);
        put32(&mut bytes, 84, self.limits.frame_ms);
        put16(&mut bytes, 88, self.authentication);
        Ok(bytes)
    }

    /// Parse exactly one immutable GATT value. Unknown bits/model/auth numbers remain public claims.
    /// # Errors
    /// Refuse truncation, extension, unsupported format and contradictory metadata.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != WIRE_BYTES {
            return Err(Error::Length);
        }
        if &bytes[..4] != b"SMDC" {
            return Err(Error::Magic);
        }
        if bytes[4] != VERSION {
            return Err(Error::Version);
        }
        if bytes[5] != 0 || bytes[90..].iter().any(|b| *b != 0) {
            return Err(Error::Reserved);
        }
        if get16(bytes, 6) != 96 {
            return Err(Error::Length);
        }
        let description = Self {
            device: core::array::from_fn(|i| bytes[8 + i]),
            boot: core::array::from_fn(|i| bytes[24 + i]),
            session: u64::from_le_bytes(core::array::from_fn(|i| bytes[40 + i])),
            model: get16(bytes, 48),
            firmware: Firmware {
                major: get16(bytes, 50),
                minor: get16(bytes, 52),
                patch: get16(bytes, 54),
            },
            capabilities: get32(bytes, 56),
            authentication: get16(bytes, 88),
            limits: Limits {
                package_version: get16(bytes, 60),
                transfer_version: get16(bytes, 62),
                package_bytes: get32(bytes, 64),
                programs: get16(bytes, 68),
                universes: get16(bytes, 70),
                message_bytes: get16(bytes, 72),
                chunk_bytes: get16(bytes, 74),
                slot_bytes: get32(bytes, 76),
                loader_bytes: get32(bytes, 80),
                frame_ms: get32(bytes, 84),
            },
        };
        description.validate()?;
        Ok(description)
    }
}

fn get16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn get32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(core::array::from_fn(|i| bytes[at + i]))
}
fn put16(bytes: &mut [u8; WIRE_BYTES], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(bytes: &mut [u8; WIRE_BYTES], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
