//! Board-local owner of the install service; only the live authenticated epoch may dispatch.
mod runtime;
use core::sync::atomic::{AtomicU8, AtomicU32, AtomicUsize, Ordering};
#[cfg(not(feature = "binding-readiness"))]
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
#[cfg(not(feature = "binding-readiness"))]
use embassy_time::Timer;
use esp_hal::{peripherals, system::Stack};
use esp_rtos::embassy::Executor;
use stagemaster_install::Installer;
use stagemaster_install_worker::{Command, Completion, Epoch, ManagedWorker};
use stagemaster_nor_store::{Layout, NorDevice};
use static_cell::{ConstStaticCell, StaticCell};

pub static REQUESTS: Channel<CriticalSectionRawMutex, Command, 1> = Channel::new();
pub static COMPLETIONS: Channel<CriticalSectionRawMutex, Completion, 1> = Channel::new();
// Readiness: 0 starting, 1 ready, 2 failed. This is not a business capability.
pub static READY: AtomicU8 = AtomicU8::new(0);
pub static LIVE_EPOCH: AtomicU32 = AtomicU32::new(0);
static STACK_TOP: AtomicUsize = AtomicUsize::new(0);
static SAMPLED_DEPTH: AtomicUsize = AtomicUsize::new(0);
static OPERATIONS: AtomicU32 = AtomicU32::new(0);
static MAX_OPERATION_US: AtomicU32 = AtomicU32::new(0);
const STACK_BYTES: usize = 32 * 1024;

pub fn live_epoch() -> Option<Epoch> {
    Epoch::new(LIVE_EPOCH.load(Ordering::Acquire))
}

pub fn start(
    cpu: peripherals::CPU_CTRL,
    interrupt: peripherals::FROM_CPU_INTR1<'static>,
    flash: peripherals::FLASH<'static>,
    boot: [u8; 16],
    _output_disabled: &crate::board::OutputDisabled,
) {
    static STACK: ConstStaticCell<Stack<STACK_BYTES>> = ConstStaticCell::new(Stack::new());
    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    let stack = STACK.take();
    STACK_TOP.store(stack.top() as usize, Ordering::Release);
    esp_rtos::start_second_core(cpu, interrupt, stack, move || {
        EXECUTOR.init(Executor::new()).run(|spawner| {
            spawner.spawn(run(flash, boot).unwrap());
        });
    });
}

// Address-only sampling on the owning core. Includes callers, not the deeper ROM
// driver's peak. Never reads a live stack through an alias or claims a high-water mark.
#[inline(never)]
pub fn sample_stack() {
    let marker = 0_u8;
    let address = core::ptr::from_ref(&marker) as usize;
    let top = STACK_TOP.load(Ordering::Acquire);
    assert!(address <= top && top - address < STACK_BYTES);
    SAMPLED_DEPTH.fetch_max(top - address, Ordering::Relaxed);
}

pub fn report() {
    esp_println::println!(
        "INSTALL WORKER operations={} max_operation_us={} sampled_stack={} heap={}/{} heap_peak={}",
        OPERATIONS.load(Ordering::Relaxed),
        MAX_OPERATION_US.load(Ordering::Relaxed),
        SAMPLED_DEPTH.load(Ordering::Relaxed),
        esp_alloc::HEAP.used(),
        esp_alloc::HEAP.free(),
        esp_alloc::HEAP.stats().max_usage
    );
    crate::measured_nor::report();
}

#[embassy_executor::task]
async fn run(peripheral: peripherals::FLASH<'static>, boot: [u8; 16]) {
    if !serve(peripheral, boot).await {
        READY.store(2, Ordering::Release);
        esp_println::println!("INSTALL WORKER unavailable; wireless installation remains disabled");
    }
}

async fn serve(peripheral: peripherals::FLASH<'static>, boot: [u8; 16]) -> bool {
    if esp_storage::flash_encryption() {
        esp_println::println!("INSTALL WORKER refuses encrypted development partition");
        return false;
    }
    // Driver and all Rc-backed handles are created and remain on this core.
    let flash = esp_storage::FlashStorage::new(peripheral).multicore_auto_park();
    #[cfg(feature = "binding-readiness")]
    let shared = crate::shared_flash::SharedFlash::new(flash);
    #[cfg(feature = "binding-readiness")]
    let (nor, mut bindings) = {
        let Some(nor) = shared.partition(crate::package_layout::PARTITION_LABEL) else {
            return false;
        };
        let Some(bindings) = shared
            .partition("stmbonds")
            .and_then(crate::bindings::Store::new)
        else {
            return false;
        };
        (nor, bindings)
    };
    #[cfg(not(feature = "binding-readiness"))]
    let mut flash = flash;
    #[cfg(not(feature = "binding-readiness"))]
    let Some(entry) = crate::package_storage::partition(&mut flash) else {
        return false;
    };
    #[cfg(not(feature = "binding-readiness"))]
    let mut region = entry.as_flash_region(&mut flash);
    #[cfg(not(feature = "binding-readiness"))]
    let nor = region.as_nor_flash().unwrap();
    let nor = crate::measured_nor::MeasuredNor::new(nor);
    let device = match NorDevice::new(nor, Layout::new(crate::package_layout::SLOT_BYTES).unwrap())
    {
        Ok(device) => device,
        Err(error) => {
            esp_println::println!("INSTALL WORKER open failed: {}", error);
            return false;
        }
    };
    #[cfg(any(
        feature = "worker-write-test",
        feature = "installation-gatt",
        feature = "application-gatt"
    ))]
    // All writes run through ManagedWorker maintenance and the independent live epoch.
    let store = device.open_for_installation().unwrap();
    #[cfg(not(any(
        feature = "worker-write-test",
        feature = "installation-gatt",
        feature = "application-gatt"
    )))]
    let store = device.open_read_only().unwrap();
    let (installer, recovery) = match Installer::open(store, boot) {
        Ok(opened) => opened,
        Err(error) => {
            esp_println::println!("INSTALL WORKER recover failed: {}", error);
            return false;
        }
    };
    let mut worker = ManagedWorker::new(
        installer,
        runtime::now(),
        64 * 1024,
        runtime::DisabledPlayback,
    )
    .unwrap();
    // main retains OutputDisabled for this entire boot. There is no physical output
    // task/queue in this firmware. Future output adapters must acknowledge real quiescence.
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), runtime::now())
        .unwrap();
    esp_println::println!(
        "INSTALL WORKER ready {:?}; worker={} command={} completion={} stack={} heap={}/{}; local-write-test={}",
        recovery,
        core::mem::size_of_val(&worker),
        core::mem::size_of::<Command>(),
        core::mem::size_of::<Completion>(),
        STACK_BYTES,
        esp_alloc::HEAP.used(),
        esp_alloc::HEAP.free(),
        cfg!(feature = "worker-write-test")
    );
    READY.store(1, Ordering::Release);
    #[cfg(feature = "worker-write-test")]
    let mut replayed = None;
    loop {
        worker.observe(live_epoch());
        #[cfg(feature = "binding-readiness")]
        let Some(command) = bindings.next(&mut worker).await else {
            continue;
        };
        #[cfg(not(feature = "binding-readiness"))]
        let command = match select(REQUESTS.receive(), Timer::after_millis(100)).await {
            Either::First(command) => command,
            Either::Second(()) => continue,
        };
        sample_stack();
        let start = esp_hal::time::Instant::now();
        let completion = worker.process(command, runtime::now(), live_epoch);
        MAX_OPERATION_US.fetch_max(
            start.elapsed().as_micros().min(u64::from(u32::MAX)) as u32,
            Ordering::Relaxed,
        );
        let count = OPERATIONS.fetch_add(1, Ordering::Relaxed) + 1;
        // esp-println formats and flushes under its critical-section lock.
        // Aggregate hot-path samples instead of blocking radio on every chunk.
        if count <= 4 || count.is_multiple_of(32) {
            report();
        }
        #[cfg(feature = "worker-write-test")]
        crate::worker_probe::verify_reply(&mut worker, &completion, &mut replayed);
        // The consumer must also check its epoch before notifying its peer.
        // A full queue yields this executor; it never blocks radio callbacks.
        COMPLETIONS.send(completion).await;
    }
}
