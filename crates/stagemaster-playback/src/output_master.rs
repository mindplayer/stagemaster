//! Final intensity attenuation, independent of transport and stored program values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutputMaster {
    percent: u8,
    blackout: bool,
}
impl Default for OutputMaster {
    fn default() -> Self {
        Self {
            percent: 100,
            blackout: false,
        }
    }
}
impl OutputMaster {
    /// # Errors
    /// Rejects percentages outside 0..=100 without changing the prior value.
    pub fn set_percent(&mut self, percent: u8) -> Result<(), &'static str> {
        if percent > 100 {
            return Err("亮度总控须在 0—100% 之间");
        }
        self.percent = percent;
        Ok(())
    }
    pub fn set_blackout(&mut self, blackout: bool) {
        self.blackout = blackout;
    }
    #[must_use]
    pub const fn percent(self) -> u8 {
        self.percent
    }
    #[must_use]
    pub const fn blackout(self) -> bool {
        self.blackout
    }
    #[must_use]
    pub const fn effective_percent(self) -> u8 {
        if self.blackout { 0 } else { self.percent }
    }
    #[must_use]
    pub fn scale(self, value: u16) -> u16 {
        u16::try_from((u32::from(value) * u32::from(self.effective_percent()) + 50) / 100)
            .unwrap_or(u16::MAX)
    }
}
