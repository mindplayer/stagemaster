//! Board metadata adapter. Factory reads only; no eFuse writes or authorization.
use stagemaster_device_info::{
    Description, Firmware, Limits, MODEL_WAVESHARE_ESP32_S3_RS485_CAN, capability, esp32_device_id,
};

pub struct Identity {
    device: [u8; 16],
    boot: [u8; 16],
}
impl Identity {
    #[cfg(feature = "worker-readiness")]
    pub const fn boot(&self) -> [u8; 16] {
        self.boot
    }

    /// Capture exactly once after main has enabled RF. The boot nonce is public
    /// correlation metadata, not a credential or proof of a secure boot chain.
    pub fn capture() -> Self {
        let mac = esp_hal::efuse::base_mac_address();
        let device = esp32_device_id(mac.as_bytes().try_into().unwrap()).unwrap();
        let mut boot = [0; 16];
        while boot == [0; 16] {
            esp_hal::rng::Rng::new().read(&mut boot);
        }
        Self { device, boot }
    }

    pub fn describe(&self, session: u64) -> [u8; 96] {
        Description {
            device: self.device,
            boot: self.boot,
            session,
            model: MODEL_WAVESHARE_ESP32_S3_RS485_CAN,
            firmware: Firmware {
                major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
                minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
                patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
            },
            // Readiness build features are NOT operational business capabilities.
            capabilities: capability::DIAGNOSTICS,
            authentication: 0,
            limits: Limits::default(),
        }
        .encode()
        .unwrap()
    }
}
