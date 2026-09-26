//! The only module that assigns board pins. The probe cannot enable the transmitter.
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    peripherals::GPIO21,
};

pub struct OutputDisabled {
    _direction: Output<'static>,
}

impl OutputDisabled {
    pub fn new(direction: GPIO21<'static>) -> Self {
        Self {
            _direction: Output::new(direction, Level::Low, OutputConfig::default()),
        }
    }
}
