use crate::wire::{Failure, Input, RecordView};
use serde_json::Value;
use stagemaster_runtime_host::Client;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

pub(crate) const MAX_SESSIONS: usize = 8;
const IDLE: Duration = Duration::from_mins(1);
#[derive(Clone)]
pub(crate) struct Binding {
    pub client: Arc<Client>,
    pub next: Option<u64>,
}
struct Record {
    input: Input,
    view: RecordView,
}
struct Session {
    touched: Instant,
    binding: Option<Binding>,
    record: Option<Record>,
}
impl Session {
    fn pending(&self) -> bool {
        self.record
            .as_ref()
            .is_some_and(|r| r.view.outcome.is_none())
    }
}
pub(crate) struct Registry {
    pub accepting: bool,
    entries: HashMap<Uuid, Session>,
}
pub(crate) struct Job {
    pub session: Uuid,
    pub input: Input,
    pub binding: Option<Binding>,
    pub deadline: Instant,
}
pub(crate) enum Admission {
    Existing(RecordView),
    New(Job),
}
pub(crate) enum Change {
    Keep,
    Set(Binding),
    Clear,
}
pub(crate) struct Completion {
    pub outcome: Value,
    pub change: Change,
}
impl Registry {
    pub fn new() -> Self {
        Self {
            accepting: true,
            entries: HashMap::new(),
        }
    }
    fn check(&mut self, now: Instant) -> Result<(), Failure> {
        if !self.accepting {
            return Err(Failure::closed());
        }
        self.entries
            .retain(|_, s| s.pending() || now.saturating_duration_since(s.touched) < IDLE);
        Ok(())
    }
    pub fn create(&mut self, now: Instant) -> Result<Uuid, Failure> {
        self.check(now)?;
        if self.entries.len() >= MAX_SESSIONS {
            return Err(Failure::conflict(
                "sessionLimit",
                "操作会话已满，请等待空闲会话释放",
            ));
        }
        let id = Uuid::new_v4();
        self.entries.insert(
            id,
            Session {
                touched: now,
                binding: None,
                record: None,
            },
        );
        Ok(id)
    }
    fn session(&mut self, id: Uuid, now: Instant) -> Result<&mut Session, Failure> {
        self.check(now)?;
        let s = self
            .entries
            .get_mut(&id)
            .ok_or_else(|| Failure::conflict("sessionMissing", "操作会话不存在或已失效"))?;
        s.touched = now;
        Ok(s)
    }
    pub fn begin(
        &mut self,
        id: Uuid,
        input: Input,
        now: Instant,
        worker_available: bool,
    ) -> Result<Admission, Failure> {
        input.validate()?;
        let s = self.session(id, now)?;
        let mut next = Some(1);
        if let Some(r) = &s.record {
            if input.serial == r.input.serial {
                return if input == r.input {
                    Ok(Admission::Existing(r.view.clone()))
                } else {
                    Err(Failure::conflict(
                        "duplicateConflict",
                        "相同序号的操作内容不同",
                    ))
                };
            }
            if input.serial.0 < r.input.serial.0 {
                return Err(Failure::conflict(
                    "notRetained",
                    "此操作已超出回执保留窗口，不会重新执行",
                ));
            }
            next = r.input.serial.0.checked_add(1);
        }
        if Some(input.serial.0) != next {
            return Err(Failure::conflict("sequence", "操作序号不连续或已耗尽"));
        }
        if s.pending() || !worker_available {
            return Err(Failure::busy());
        }
        let view = RecordView {
            serial: input.serial,
            status: "pending",
            outcome: None,
        };
        let job = Job {
            session: id,
            binding: s.binding.clone(),
            deadline: now + Duration::from_millis(input.ttl_ms),
            input: input.clone(),
        };
        s.record = Some(Record { input, view });
        Ok(Admission::New(job))
    }
    pub fn receipt(&mut self, id: Uuid, serial: u64, now: Instant) -> Result<RecordView, Failure> {
        let s = self.session(id, now)?;
        let r = s
            .record
            .as_ref()
            .filter(|r| r.input.serial.0 == serial)
            .ok_or_else(|| {
                Failure::conflict("notRetained", "该序号没有可查询回执，请核对当前执行状态")
            })?;
        Ok(r.view.clone())
    }
    pub fn finish(&mut self, id: Uuid, serial: u64, result: Completion, now: Instant) {
        if let Some(s) = self.entries.get_mut(&id)
            && let Some(r) = s
                .record
                .as_mut()
                .filter(|r| r.input.serial.0 == serial && r.view.outcome.is_none())
        {
            r.view.status = "complete";
            r.view.outcome = Some(result.outcome);
            s.touched = now;
            match result.change {
                Change::Keep => {}
                Change::Set(binding) => s.binding = Some(binding),
                Change::Clear => s.binding = None,
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/sessions.rs"]
mod tests;
