//! Two-slot package storage on a dedicated synchronous NOR partition, not a filesystem.
//! Writes require an explicit offline maintenance session; leases alone do not ensure real-time I/O.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
mod io;
mod metadata;
mod store;

use alloc::rc::Rc;
use core::{
    cell::{Cell, RefCell},
    fmt,
};
use embedded_storage::nor_flash::NorFlash;
use stagemaster_install::Slot;
use stagemaster_package::MAX_PACKAGE_BYTES;
pub use store::{NorStore, Snapshot};

pub const METADATA_BYTES: usize = 4096;
pub const MAX_WORD_BYTES: usize = 256;
pub const IO_BYTES: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Geometry,
    Capacity,
    Busy,
    ReadOnly,
    Bounds,
    State,
    Integrity,
    Format,
    Layout,
}
impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Geometry => "Flash 擦写粒度不支持",
            Self::Capacity => "Flash 分区容量与双槽布局不符",
            Self::Busy => "Flash 存储或读源正在使用",
            Self::ReadOnly => "当前只允许读取，安装须进入维护状态",
            Self::Bounds => "Flash 访问超出专用分区或包范围",
            Self::State => "Flash 暂存状态不允许此操作",
            Self::Integrity => "Flash 擦写后读回不一致",
            Self::Format => "Flash 存储格式不支持",
            Self::Layout => "Flash 已有布局与当前配置不符，不能覆盖",
        })
    }
}
#[derive(Debug)]
pub enum Error<E> {
    Code(Code),
    Flash(E),
}
impl<E> From<Code> for Error<E> {
    fn from(e: Code) -> Self {
        Self::Code(e)
    }
}
impl<E: fmt::Debug> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(c) => c.fmt(f),
            Self::Flash(e) => write!(f, "Flash 操作失败：{e:?}"),
        }
    }
}
impl<E: fmt::Debug> core::error::Error for Error<E> {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    slot_bytes: usize,
    total: usize,
}
impl Layout {
    /// # Errors
    /// Require two exact-size slots and fixed metadata positions; never infer writable chip offsets.
    pub fn new(slot_bytes: usize) -> Result<Self, Code> {
        if !(METADATA_BYTES..=MAX_PACKAGE_BYTES).contains(&slot_bytes)
            || !slot_bytes.is_multiple_of(METADATA_BYTES)
        {
            return Err(Code::Capacity);
        }
        Ok(Self {
            slot_bytes,
            // Xtensa LLVM 21 mis-selects a shifted value plus this large immediate
            // (esp-rs/rust#275). Keep the constant in a register; no geometry rule changes.
            total: core::hint::black_box(2 * METADATA_BYTES) + 2 * slot_bytes,
        })
    }
    #[must_use]
    pub const fn slot_bytes(self) -> usize {
        self.slot_bytes
    }
    #[must_use]
    pub const fn total_bytes(self) -> usize {
        self.total
    }
    #[must_use]
    pub const fn metadata_offset(self, slot: Slot) -> usize {
        slot.index() * METADATA_BYTES
    }
    #[must_use]
    pub const fn payload_offset(self, slot: Slot) -> usize {
        2 * METADATA_BYTES + slot.index() * self.slot_bytes
    }
}
struct Shared<F> {
    flash: RefCell<F>,
    layout: Layout,
    writer: Cell<bool>,
    active: Cell<Option<Slot>>,
    pins: [Cell<u32>; 2],
}
/// The sole owner of a synchronous, partition-bounded NOR driver. Single executor, not Send/Sync.
pub struct NorDevice<F> {
    shared: Rc<Shared<F>>,
}
impl<F> Clone for NorDevice<F> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}
impl<F: NorFlash> NorDevice<F> {
    /// Constructor reads metadata but never erases, formats or writes.
    /// The driver must complete physical operations before returning; no volatile write cache.
    /// # Errors
    /// Reject incompatible geometry, wrong partition length, valid incompatible layouts and read failures.
    pub fn new(flash: F, layout: Layout) -> Result<Self, Error<F::Error>> {
        if !F::READ_SIZE.is_power_of_two()
            || F::READ_SIZE > 16
            || !F::WRITE_SIZE.is_power_of_two()
            || F::WRITE_SIZE > MAX_WORD_BYTES
            || !F::ERASE_SIZE.is_power_of_two()
            || !(256..=METADATA_BYTES).contains(&F::ERASE_SIZE)
            || !F::ERASE_SIZE.is_multiple_of(F::WRITE_SIZE)
            || !F::ERASE_SIZE.is_multiple_of(F::READ_SIZE)
        {
            return Err(Code::Geometry.into());
        }
        if flash.capacity() != layout.total {
            return Err(Code::Capacity.into());
        }
        let device = Self {
            shared: Rc::new(Shared {
                flash: RefCell::new(flash),
                layout,
                writer: Cell::new(false),
                active: Cell::new(None),
                pins: [Cell::new(0), Cell::new(0)],
            }),
        };
        for slot in [Slot::A, Slot::B] {
            metadata::read(&device.shared, slot)?;
        }
        Ok(device)
    }
    /// # Errors
    /// Refuse a second live storage session. Opening does not mutate flash.
    pub fn open_read_only(&self) -> Result<NorStore<F>, Error<F::Error>> {
        self.open(false)
    }
    /// Trusted runtime entry: physical output must already be stopped or independently proven unaffected.
    /// # Errors
    /// Refuse a second live storage session. A peer cannot grant itself this mode.
    pub fn open_for_installation(&self) -> Result<NorStore<F>, Error<F::Error>> {
        self.open(true)
    }
    fn open(&self, writable: bool) -> Result<NorStore<F>, Error<F::Error>> {
        if self.shared.writer.replace(true) {
            return Err(Code::Busy.into());
        }
        Ok(NorStore::new(self.shared.clone(), writable))
    }
}

#[repr(align(16))]
struct Buffer([u8; IO_BYTES]);
impl Buffer {
    const fn erased() -> Self {
        Self([0xff; IO_BYTES])
    }
}
