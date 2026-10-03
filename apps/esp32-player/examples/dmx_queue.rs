//! Logic-only queue/UART lab image. GPIO21 remains low; no physical Sent claims.
#![no_std]
#![no_main]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use stagemaster_output_port::{
    Config, Port, Sample, Source, SourceKind,
    dmx::{
        Transmitter,
        queued::{Queue, Receiver},
    },
};
use static_cell::StaticCell;
#[path = "../src/board/dmx.rs"]
mod dmx;
#[path = "support/logic_line.rs"]
mod logic_line;
use logic_line::LogicLine;
esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn transmit(rx: Receiver<'static, CriticalSectionRawMutex>, line: LogicLine) {
    rx.run(Transmitter::new(line, dmx::Monotonic, 40_000).unwrap())
        .await;
}

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    esp_alloc::heap_allocator!(size: 128 * 1024);
    let p = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let line = LogicLine(dmx::DmxLine::new(p.UART1, p.GPIO17, p.GPIO21).unwrap());
    let timer = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timer.timer0, p.FROM_CPU_INTR0);
    static QUEUE: StaticCell<Queue<CriticalSectionRawMutex>> = StaticCell::new();
    let (driver, rx) = QUEUE.init(Queue::new()).split().unwrap();
    spawner.spawn(transmit(rx, line).unwrap());
    let now = || embassy_time::Instant::now().as_millis();
    let mut port = Port::new(
        Config {
            boot: [0x55; 16],
            port: 1,
            universe: 1,
            max_age_ms: 100,
            ack_timeout_ms: 100,
        },
        driver,
        now(),
    )
    .unwrap();
    port.select(
        Source {
            id: [0x33; 16],
            kind: SourceKind::Local,
        },
        false,
        now(),
    )
    .unwrap();
    while port.state().permit.is_none() {
        port.poll(now()).unwrap();
        embassy_time::Timer::after_millis(1).await;
    }
    let permit = port.state().permit.unwrap();
    esp_println::println!("队列逻辑侧探针：RS485始终关闭，无蓝牙／节目安装；不是物理输出验收");
    for serial in 1..=40 {
        let start = now();
        let slots = [u8::try_from(serial).unwrap(); 512];
        port.submit(
            permit,
            Sample {
                serial,
                sampled_ms: start,
                universe: 1,
                slots: &slots,
            },
            start,
        )
        .unwrap();
        loop {
            port.poll(now()).unwrap();
            if now() >= start + 25 {
                break;
            }
            embassy_time::Timer::after_millis(1).await;
        }
    }
    port.stop(permit, now()).unwrap();
    while !port.state().quiet {
        port.poll(now()).unwrap();
        embassy_time::Timer::after_millis(1).await;
    }
    esp_println::println!("逻辑侧队列停止已排空；未启用RS485");
    core::future::pending::<()>().await;
}
