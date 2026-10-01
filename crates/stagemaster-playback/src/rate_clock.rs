//! Fixed-point transport time. Sampling frequency cannot change the result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RateClock {
    real_ms: u64,
    logical_ms: u64,
    hundredths: u8,
    percent: u16,
}
impl RateClock {
    #[must_use]
    pub const fn new(now_ms: u64) -> Self {
        Self {
            real_ms: now_ms,
            logical_ms: now_ms,
            hundredths: 0,
            percent: 100,
        }
    }
    #[must_use]
    pub const fn percent(self) -> u16 {
        self.percent
    }

    /// # Errors
    /// Backwards input or logical overflow leaves the complete clock unchanged.
    pub fn advance(&mut self, now_ms: u64) -> Result<u64, &'static str> {
        let delta = now_ms.checked_sub(self.real_ms).ok_or("预演时钟不能倒退")?;
        let scaled = u128::from(delta) * u128::from(self.percent) + u128::from(self.hundredths);
        let whole = u64::try_from(scaled / 100).map_err(|_| "预演时间超出范围")?;
        let next = self
            .logical_ms
            .checked_add(whole)
            .ok_or("预演时间超出范围")?;
        let remainder = u8::try_from(scaled % 100).map_err(|_| "预演时间超出范围")?;
        self.real_ms = now_ms;
        self.logical_ms = next;
        self.hundredths = remainder;
        Ok(next)
    }

    /// Integrate the old rate first; a new rate only applies after this instant.
    /// # Errors
    /// Invalid percentages, backwards input or overflow leave all state unchanged.
    pub fn set_rate(&mut self, now_ms: u64, percent: u16) -> Result<u64, &'static str> {
        if !(25..=400).contains(&percent) {
            return Err("预演速率须为 25—400% 的整数");
        }
        let time = self.advance(now_ms)?;
        self.percent = percent;
        Ok(time)
    }
}
