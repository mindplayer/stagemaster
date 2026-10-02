use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct Freshness {
    last: Option<(u64, u64, u64, Instant)>,
}
impl Freshness {
    pub fn accept(
        &mut self,
        cycles: u64,
        sampled: u64,
        sequence: u64,
        now: Instant,
    ) -> Result<(), String> {
        if let Some((old_cycles, old_sampled, old_sequence, at)) = self.last {
            if cycles < old_cycles || sampled < old_sampled || sequence < old_sequence {
                return Err("后台采样顺序异常，请重新连接".into());
            }
            if sequence == old_sequence {
                if sampled != old_sampled {
                    return Err("后台采样身份不一致".into());
                }
                if now.saturating_duration_since(at) >= Duration::from_secs(2) {
                    return Err("后台采样已过期，三维暂停显示灯光".into());
                }
                return Ok(());
            }
        }
        self.last = Some((cycles, sampled, sequence, now));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_samples_never_extend_freshness_and_new_samples_recover() {
        let mut clock = Freshness::default();
        let now = Instant::now();
        clock.accept(1, 10, 1, now).unwrap();
        clock
            .accept(2, 10, 1, now + Duration::from_millis(1900))
            .unwrap();
        assert!(
            clock
                .accept(3, 10, 1, now + Duration::from_secs(2))
                .is_err()
        );
        clock
            .accept(4, 20, 2, now + Duration::from_secs(3))
            .unwrap();
        assert!(clock.accept(3, 20, 2, now).is_err());
        assert!(clock.accept(5, 21, 2, now).is_err());
        assert!(clock.accept(5, 19, 3, now).is_err());
        clock
            .accept(5, 20, 3, now + Duration::from_secs(3))
            .unwrap();
    }
}
