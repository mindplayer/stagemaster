//! Logic-side lab image only. It never calls Line::enable and keeps RS485 off.
//! No Bluetooth, package writes or physical output capability is advertised.
#![no_std]
#![no_main]
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use stagemaster_output_port::dmx::{BREAK_BITS, Clock, FRAME_BYTES, Line, MARK_US};
#[path = "../src/board/dmx.rs"]
mod dmx;
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) {
    esp_alloc::heap_allocator!(size: 128 * 1024);
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let mut line =
        dmx::DmxLine::new(peripherals.UART1, peripherals.GPIO17, peripherals.GPIO21).unwrap();
    let timer = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timer.timer0, peripherals.FROM_CPU_INTR0);
    let clock = dmx::Monotonic;
    let mut packet = [0; FRAME_BYTES];
    for (index, byte) in packet[1..].iter_mut().enumerate() {
        *byte = (index % 256) as u8;
    }
    esp_println::println!("UART逻辑侧探针；RS485方向始终关闭，不连接灯具；不报告物理发送完成");
    loop {
        assert!(line.is_disabled());
        let start = clock.now_us();
        line.drain().await.unwrap();
        line.break_signal(BREAK_BITS).await.unwrap();
        clock.wait_until_us(clock.now_us() + MARK_US).await;
        let mut offset = 0;
        while offset < packet.len() {
            let count = line.write(&packet[offset..]).await.unwrap();
            assert!(count > 0 && count <= packet.len() - offset);
            offset += count;
        }
        line.drain().await.unwrap();
        assert!(line.is_disabled());
        clock.wait_until_us(start + 25_000).await;
    }
}
