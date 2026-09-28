//! Read-only, build-only readiness. No link is authorized, no erase/write is invoked.
use crate::package_layout;
use esp_bootloader_esp_idf::partitions::{
    PARTITION_TABLE_MAX_LEN, PartitionEntry, read_partition_table,
};
use esp_storage::FlashStorage;
#[cfg(not(feature = "worker-readiness"))]
use stagemaster_install::{Installer, Storage};
#[cfg(not(feature = "worker-readiness"))]
use stagemaster_nor_store::{Layout, NorDevice};
#[cfg(not(feature = "worker-readiness"))]
use stagemaster_transfer::{Frame, Service};

#[cfg(not(feature = "worker-readiness"))]
type Driver = esp_bootloader_esp_idf::partitions::NorFlashRegion<'static, 'static, 'static>;
#[cfg(not(feature = "worker-readiness"))]
type Store = stagemaster_nor_store::NorStore<Driver>;
// Kept in the ELF and printed by the read-only check, so budgets use actual Xtensa
// type layouts rather than macOS pointer sizes. These are sizes, not peak heap usage.
#[cfg(not(feature = "worker-readiness"))]
pub static RESOURCE_BYTES: [usize; 6] = [
    core::mem::size_of::<Service<Store>>(),
    core::mem::size_of::<Store>(),
    core::mem::size_of::<stagemaster_nor_store::Snapshot<Driver>>(),
    core::mem::size_of::<Frame>(),
    core::mem::size_of::<stagemaster_transfer::Assembler>(),
    core::mem::size_of::<Driver>(),
];

pub(crate) fn partition(flash: &mut FlashStorage<'_>) -> Option<PartitionEntry> {
    let mut bytes = [0; PARTITION_TABLE_MAX_LEN];
    let capacity = flash.capacity();
    let table = match read_partition_table(flash, &mut bytes) {
        Ok(table) => table,
        Err(error) => {
            esp_println::println!("节目分区表读取失败：{:?}", error);
            return None;
        }
    };
    let index = match package_layout::validate(capacity, table.len(), |i| {
        table.get_partition(i).ok().map(|e| package_layout::Entry {
            package_label: e.label_as_str() == package_layout::PARTITION_LABEL,
            kind: e.raw_type(),
            subtype: e.raw_subtype(),
            offset: e.offset(),
            length: e.len(),
            flags: e.flags(),
        })
    }) {
        Ok(index) => index,
        Err(error) => {
            esp_println::println!("节目分区未就绪：{:?}；保持只读", error);
            return None;
        }
    };
    table.get_partition(index).ok()
}

#[cfg(not(feature = "worker-readiness"))]
pub fn inspect(peripheral: esp_hal::peripherals::FLASH<'static>) {
    if esp_storage::flash_encryption() {
        esp_println::println!("当前存储检查仅支持未加密的开发板分区");
        return;
    }
    let mut flash = FlashStorage::new(peripheral);
    let Some(entry) = partition(&mut flash) else {
        return;
    };
    let mut region = entry.as_flash_region(&mut flash);
    let nor = match region.as_nor_flash() {
        Ok(nor) => nor,
        Err(error) => {
            esp_println::println!("节目分区 NOR 接口未就绪：{:?}", error);
            return;
        }
    };
    let device = match NorDevice::new(nor, Layout::new(package_layout::SLOT_BYTES).unwrap()) {
        Ok(device) => device,
        Err(error) => {
            esp_println::println!("节目存储打开失败：{}", error);
            return;
        }
    };
    let mut boot = [0; 16];
    esp_hal::rng::Rng::new().read(&mut boot);
    let installer = match Installer::open(device.open_read_only().unwrap(), boot) {
        Ok((installer, report)) => {
            esp_println::println!("节目存储只读检查：{:?}", report);
            installer
        }
        Err(error) => {
            esp_println::println!("节目恢复失败：{}", error);
            return;
        }
    };
    let mut service = Service::new(installer).unwrap();
    retain_handler(&mut service);
    #[cfg(feature = "runtime-readiness")]
    crate::runtime_readiness::inspect(&service, boot);
    esp_println::println!(
        "Xtensa 类型大小（服务含存储／存储／读源／帧／重组／驱动）：{:?}",
        RESOURCE_BYTES
    );
    esp_println::println!(
        "节目存储构建检查：服务 {} B；当前堆 {} B；未接收安装请求",
        core::mem::size_of_val(&service),
        esp_alloc::HEAP.used()
    );
}

// Compile and link the actual ESP32 driver path for every service action, but do not
// invoke it or attach a link. A generic-core-only check would miss driver incompatibilities.
#[cfg(not(feature = "worker-readiness"))]
fn retain_handler<S: Storage>(service: &mut Service<S>) {
    let handler: fn(&mut Service<S>, &[u8]) -> Result<Frame, stagemaster_transfer::Error> =
        Service::<S>::process;
    core::hint::black_box((handler, service));
}
