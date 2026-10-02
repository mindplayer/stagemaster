use super::Service;
use std::sync::atomic::Ordering;

pub(super) struct PreparationGuard<'a>(&'a Service);
impl Drop for PreparationGuard<'_> {
    fn drop(&mut self) {
        self.0.preparing.store(false, Ordering::Release);
    }
}

impl Service {
    pub(super) fn cancellation_version(&self) -> u64 {
        self.cancellation.load(Ordering::Acquire)
    }

    pub(super) fn cancel(&self) {
        self.cancellation.fetch_add(1, Ordering::AcqRel);
        self.cancelled.store(true, Ordering::Release);
    }

    pub(super) fn check_cancellation_version(&self, expected: u64) -> Result<(), String> {
        if expected != self.cancellation_version() {
            return Err("音乐操作已取消".into());
        }
        Ok(())
    }

    pub(super) fn begin_preparation(&self, expected: u64) -> Result<PreparationGuard<'_>, String> {
        self.preparing
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "已有音乐正在准备，请稍候或取消")?;
        let guard = PreparationGuard(self);
        self.cancelled.store(false, Ordering::Release);
        self.check_cancellation_version(expected)?;
        Ok(guard)
    }

    pub(super) fn check_cancelled(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err("音乐准备已取消".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_preparation_gate_excludes_overlaps_and_cancel_does_not_leak_to_the_next_job() {
        let dir = tempfile::tempdir().unwrap();
        let service = Service::new(dir.path().to_owned());
        let version = service.cancellation_version();
        let guard = service.begin_preparation(version).unwrap();
        assert!(service.begin_preparation(version).is_err());
        service.cancel();
        assert!(service.check_cancelled().is_err());
        drop(guard);
        assert!(service.begin_preparation(version).is_err());
        let _next = service
            .begin_preparation(service.cancellation_version())
            .unwrap();
        service.check_cancelled().unwrap();
    }

    #[test]
    fn cancel_before_worker_starts_rejects_the_old_request_and_releases_the_gate() {
        let dir = tempfile::tempdir().unwrap();
        let service = Service::new(dir.path().to_owned());
        let waiting = service.cancellation_version();
        service.cancel();
        assert!(service.check_cancellation_version(waiting).is_err());
        assert!(service.begin_preparation(waiting).is_err());
        let latest = service.cancellation_version();
        let _next = service.begin_preparation(latest).unwrap();
        service.check_cancelled().unwrap();
        service.check_cancellation_version(latest).unwrap();
    }

    #[test]
    fn rejected_pause_and_stop_do_not_cancel_another_project_preparation() {
        let dir = tempfile::tempdir().unwrap();
        let service = Service::new(dir.path().to_owned());
        let mut session = crate::session::Session::default();
        let version = service.cancellation_version();
        let _guard = service.begin_preparation(version).unwrap();
        for command in [crate::audio::Command::Pause, crate::audio::Command::Stop] {
            assert!(service.apply_immediate(&mut session, 1, command).is_err());
            service.check_cancelled().unwrap();
            service.check_cancellation_version(version).unwrap();
        }
        service
            .apply_immediate(&mut session, 0, crate::audio::Command::Stop)
            .unwrap();
        assert!(service.check_cancelled().is_err());
        assert!(service.check_cancellation_version(version).is_err());
    }
}
