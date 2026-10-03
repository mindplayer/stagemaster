//! Sole UART1/GPIO17/GPIO21 owner. Never coexist with the diagnostic TX LED owner.
#[cfg(feature = "runtime-dmx-probe")]
#[path = "dmx/metrics.rs"]
pub mod metrics;
use embassy_time::{Delay, Instant, Timer};
use esp_hal::{
    Async, Blocking,
    gpio::{Level, Output, OutputConfig},
    peripherals::{GPIO17, GPIO21, UART1},
    uart::{Config, ConfigError, DataBits, Parity, StopBits, TxError, UartTx},
};
use stagemaster_output_port::dmx::{BAUD, Clock, Line};

pub struct DmxLine {
    tx: UartTx<'static, Async>,
    direction: Output<'static>,
}
/// Configure disabled pins on boot; bind the UART interrupt on its execution core.
pub struct PreparedLine {
    tx: UartTx<'static, Blocking>,
    direction: Output<'static>,
}
impl PreparedLine {
    pub fn new(
        uart: UART1<'static>,
        tx: GPIO17<'static>,
        enable: GPIO21<'static>,
    ) -> Result<Self, ConfigError> {
        // Disable before UART/pin setup, even if configuration fails.
        let direction = Output::new(enable, Level::Low, OutputConfig::default());
        let config = Config::default()
            .with_baudrate(BAUD)
            .with_data_bits(DataBits::_8)
            .with_parity(Parity::None)
            .with_stop_bits(StopBits::_2);
        let tx = UartTx::new(uart, config)?.with_tx(tx);
        Ok(Self { tx, direction })
    }
    pub fn into_async(self) -> DmxLine {
        DmxLine {
            tx: self.tx.into_async(),
            direction: self.direction,
        }
    }
}
impl DmxLine {
    #[cfg(not(feature = "runtime-dmx-probe"))]
    pub fn new(
        uart: UART1<'static>,
        tx: GPIO17<'static>,
        enable: GPIO21<'static>,
    ) -> Result<Self, ConfigError> {
        Ok(PreparedLine::new(uart, tx, enable)?.into_async())
    }
    pub fn is_disabled(&self) -> bool {
        self.direction.is_set_low()
    }
}
impl Line for DmxLine {
    type Error = TxError;
    fn disable(&mut self) {
        self.direction.set_low();
    }
    fn enable(&mut self) {
        self.direction.set_high();
    }
    async fn break_signal(&mut self, bits: u32) -> Result<(), TxError> {
        #[cfg(feature = "runtime-dmx-probe")]
        let operation = metrics::Operation::begin(0);
        self.tx.send_break_async(&mut Delay, bits).await;
        #[cfg(feature = "runtime-dmx-probe")]
        operation.complete();
        Ok(())
    }
    async fn write(&mut self, bytes: &[u8]) -> Result<usize, TxError> {
        #[cfg(feature = "runtime-dmx-probe")]
        let operation = metrics::Operation::begin(1);
        // Sole writer: once ready, FIFO capacity cannot decrease before write.
        // The transmitter's unchanged deadline bounds this cooperative wait.
        while !bytes.is_empty() && !self.tx.write_ready() {
            Timer::after_millis(1).await;
        }
        let result = self.tx.write(bytes);
        #[cfg(feature = "runtime-dmx-probe")]
        operation.complete();
        result
    }
    async fn drain(&mut self) -> Result<(), TxError> {
        // HAL also checks the final shifter after FIFO drain. Its last-byte poll
        // is synchronous; an independent hardware watchdog is still required
        // for a wedged peripheral/CPU before enabling production RS485 output.
        #[cfg(feature = "runtime-dmx-probe")]
        let operation = metrics::Operation::begin(2);
        let result = self.tx.flush_async().await;
        #[cfg(feature = "runtime-dmx-probe")]
        operation.complete();
        result
    }
}
impl Drop for DmxLine {
    fn drop(&mut self) {
        self.direction.set_low();
    }
}

pub struct Monotonic;
impl Clock for Monotonic {
    fn now_us(&self) -> u64 {
        Instant::now().as_micros()
    }
    async fn wait_until_us(&self, deadline: u64) {
        Timer::at(Instant::from_micros(deadline)).await;
        #[cfg(feature = "runtime-dmx-probe")]
        metrics::timer(deadline);
    }
}
