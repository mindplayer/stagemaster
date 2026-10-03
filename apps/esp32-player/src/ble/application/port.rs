use super::gate::{Command, Completion, Mode, Port, Result, Validity};
use crate::installation::{self, runtime_io};
use core::sync::atomic::Ordering;

pub(super) struct Queues(pub Mode);
impl Port for Queues {
    fn publish(&mut self, value: Option<Validity>) {
        match value {
            Some(Validity::Installation(epoch)) => {
                runtime_io::publish(None);
                installation::LIVE_EPOCH.store(epoch.get(), Ordering::Release);
            }
            Some(Validity::Runtime(live)) => {
                installation::LIVE_EPOCH.store(0, Ordering::Release);
                runtime_io::publish(Some(live));
            }
            None => {
                installation::LIVE_EPOCH.store(0, Ordering::Release);
                runtime_io::publish(None);
            }
        }
    }
    fn enqueue(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Installation(c) => installation::REQUESTS
                .try_send(c)
                .map_err(|_| "安装队列已满"),
            Command::Runtime(c) => {
                if !runtime_io::live().is_some_and(|live| {
                    live.epoch() == c.epoch()
                        && live
                            .grant(embassy_time::Instant::now().as_millis())
                            .is_some()
                }) {
                    return Err("排队前运行权限已失效");
                }
                runtime_io::REQUESTS.try_send(c).map_err(|_| "运行队列已满")
            }
        }
    }
    fn completion(&mut self) -> Option<Completion> {
        match self.0 {
            Mode::Installation => installation::COMPLETIONS
                .try_receive()
                .ok()
                .map(Completion::Installation),
            Mode::Runtime => runtime_io::COMPLETIONS
                .try_receive()
                .ok()
                .map(Completion::Runtime),
        }
    }
}

pub(super) fn clear() {
    Queues(Mode::Installation).publish(None);
}
