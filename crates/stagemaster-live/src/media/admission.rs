use super::{Group, Sample, Status, nanos};
use stagemaster_playback::MAX_TIME_MS;
use stagemaster_time::{Clock, Mapping};

impl Group {
    pub(super) fn expired(&self, now_ms: u64) -> Result<bool, String> {
        Ok(self.last.is_some()
            && nanos(now_ms)?.saturating_sub(self.observed_host_ns)
                >= nanos(self.spec.limits.max_gap_ms)?)
    }
    pub(crate) fn expire(&mut self, now_ms: u64) -> Result<(), String> {
        if matches!(self.status, Status::Following | Status::Paused) && self.expired(now_ms)? {
            self.status = Status::Lost;
        }
        Ok(())
    }
    pub(super) fn validate(
        &self,
        sample: Sample,
        mapping: &Mapping,
        host: Clock,
        now_ms: u64,
        starting: bool,
    ) -> Result<u64, String> {
        if sample.at.clock != self.spec.clock
            || mapping.source() != self.spec.clock
            || mapping.target() != host
        {
            return Err("媒体观测与时钟映射身份不一致".into());
        }
        if sample.sequence == 0 || sample.position_ms > MAX_TIME_MS {
            return Err("媒体观测序号或位置超出范围".into());
        }
        let window = mapping.convert(sample.at).map_err(|e| e.to_string())?;
        let now = nanos(now_ms)?;
        let age = now
            .checked_sub(window.earliest().nanos)
            .ok_or("媒体观测位于宿主未来")?;
        if window.latest().nanos > now
            || age >= self.spec.limits.max_age_ns
            || age >= nanos(self.spec.limits.max_gap_ms)?
            || window.uncertainty_ns() > self.spec.limits.max_uncertainty_ns
        {
            return Err("媒体观测过期、位于未来或误差超限".into());
        }
        if !starting {
            if !matches!(self.status, Status::Following | Status::Paused) || self.expired(now_ms)? {
                return Err("同步组未运行或已失联，须重新准备播放代次".into());
            }
            let previous = self.last.ok_or("同步组缺少上次观测")?;
            if sample.sequence <= previous.sequence || sample.at.nanos <= previous.at.nanos {
                return Err("媒体观测重复或倒序".into());
            }
            let progress = sample
                .position_ms
                .checked_sub(previous.position_ms)
                .ok_or("媒体定位须重新准备播放代次")?;
            let elapsed = sample.at.nanos - previous.at.nanos;
            let bound = (u128::from(elapsed) * u128::from(self.spec.limits.max_rate_percent))
                .div_ceil(100_000_000)
                + u128::from(self.spec.limits.position_tolerance_ms);
            if u128::from(progress) > bound
                || (!previous.playing && !sample.playing && progress != 0)
            {
                return Err("媒体位置不连续，须重新准备播放代次".into());
            }
        }
        Ok(window.earliest().nanos)
    }
    pub(super) fn accept(&mut self, sample: Sample, observed_host_ns: u64) {
        self.last = Some(sample);
        self.observed_host_ns = observed_host_ns;
        self.status = if sample.playing {
            Status::Following
        } else {
            Status::Paused
        };
    }
}
