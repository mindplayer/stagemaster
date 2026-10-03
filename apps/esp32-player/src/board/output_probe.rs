//! Original application assembly's UART owner. The isolated bus remains disabled.
use super::{
    dmx::{DmxLine, Monotonic},
    logic_dmx::LogicLine,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use stagemaster_output_port::dmx::{
    Transmitter,
    queued::{Queue, QueuedDriver, Receiver},
};
use static_cell::StaticCell;

pub type Driver = QueuedDriver<'static, CriticalSectionRawMutex>;

pub fn start(spawner: embassy_executor::Spawner, line: DmxLine) -> Driver {
    static QUEUE: StaticCell<Queue<CriticalSectionRawMutex>> = StaticCell::new();
    let (driver, rx) = QUEUE.init(Queue::new()).split().unwrap();
    spawner.spawn(transmit(rx, LogicLine(line)).unwrap());
    driver
}

#[embassy_executor::task]
async fn transmit(rx: Receiver<'static, CriticalSectionRawMutex>, line: LogicLine) {
    rx.run(Transmitter::new(line, Monotonic, 40_000).unwrap())
        .await;
}
