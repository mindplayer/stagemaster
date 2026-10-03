//! Lab-only gate: exercise real UART while never enabling the isolated bus.
use crate::dmx::DmxLine;
use stagemaster_output_port::dmx::Line;
pub struct LogicLine(pub DmxLine);
impl Line for LogicLine {
    type Error = esp_hal::uart::TxError;
    fn disable(&mut self) {
        self.0.disable();
    }
    fn enable(&mut self) {
        assert!(self.0.is_disabled());
    }
    async fn break_signal(&mut self, bits: u32) -> Result<(), Self::Error> {
        self.0.break_signal(bits).await
    }
    async fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        self.0.write(bytes).await
    }
    async fn drain(&mut self) -> Result<(), Self::Error> {
        self.0.drain().await
    }
}
