//! The only module that assigns board pins. The probe cannot enable the transmitter.
#[cfg(feature = "runtime-dmx-probe")]
pub mod dmx;
#[cfg(feature = "runtime-dmx-probe")]
mod logic_dmx;
#[cfg(feature = "runtime-dmx-probe")]
pub mod output_probe;
#[cfg(not(feature = "runtime-dmx-probe"))]
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    peripherals::{GPIO17, GPIO21},
};

#[cfg(not(feature = "runtime-dmx-probe"))]
pub struct OutputDisabled {
    direction: Output<'static>,
    diagnostic_tx: Output<'static>,
}

#[cfg(not(feature = "runtime-dmx-probe"))]
impl OutputDisabled {
    pub fn new(direction: GPIO21<'static>, tx: GPIO17<'static>) -> Self {
        // Disable the physical transmitter BEFORE taking ownership of TXD1.
        let direction = Output::new(direction, Level::Low, OutputConfig::default());
        Self {
            direction,
            diagnostic_tx: Output::new(tx, Level::High, OutputConfig::default()),
        }
    }

    /// LED2 green is active-low on TXD1, not an independent status LED.
    /// This diagnostic-only owner never enables RS485. A future UART owner must
    /// take exclusive ownership of the pin; do not call this during DMX output.
    pub fn connected(&mut self, connected: bool) {
        assert!(self.direction.is_set_low());
        if self.diagnostic_tx.is_set_low() == connected {
            return;
        }
        self.diagnostic_tx
            .set_level(if connected { Level::Low } else { Level::High });
        esp_println::println!(
            "连接指示灯：{}；GPIO17={}；GPIO21=低，RS485发送禁用",
            if connected { "绿灯亮" } else { "绿灯灭" },
            if connected { "低" } else { "高" },
        );
    }
}
