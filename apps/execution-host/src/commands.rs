use crate::{
    projection,
    sessions::{Binding, Change, Completion, Job},
    wire::Command,
};
use serde_json::json;
use stagemaster_runtime::{Grant, Origin};
use stagemaster_runtime_host::{Error, Host, WaitError};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

enum Problem {
    Host(Error),
    Unknown,
    NoControl,
    Exhausted,
    Invalid,
}
impl From<Error> for Problem {
    fn from(value: Error) -> Self {
        Self::Host(value)
    }
}

// A timeout keeps this same ticket/connection alive. It is never a retry or cancellation.
fn resolve<T>(
    mut wait: impl FnMut(Duration) -> Result<Result<T, Error>, WaitError>,
) -> Result<T, Problem> {
    loop {
        match wait(Duration::from_millis(250)) {
            Ok(value) => return value.map_err(Into::into),
            Err(WaitError::Timeout) => {}
            Err(WaitError::Unavailable) => return Err(Problem::Unknown),
        }
    }
}
pub(crate) fn run(host: &Mutex<Host>, job: &Job) -> Completion {
    execute(host,job).unwrap_or_else(|error| {
        let (code,message,change)=match error {
            Problem::Host(e)=>(format!("{e:?}"),e.to_string(),Change::Keep),
            Problem::Unknown=>("unknown".into(),"操作结果无法确认，请重新核对状态和控制权".into(),Change::Clear),
            Problem::NoControl=>("noControl".into(),"请先取得运行控制权".into(),Change::Keep),
            Problem::Exhausted=>("exhausted".into(),"控制序号已耗尽，请重新取得控制权".into(),Change::Keep),
            Problem::Invalid=>("invalid".into(),"操作参数无效".into(),Change::Keep),
        };
        Completion { outcome:json!({"kind":if code=="unknown" {"unknown"} else {"rejected"},"code":code,"message":message}),change }
    })
}
fn execute(host: &Mutex<Host>, job: &Job) -> Result<Completion, Problem> {
    let ttl = job.deadline.saturating_duration_since(Instant::now());
    if ttl.is_zero() {
        return Err(Error::Deadline.into());
    }
    if let Command::Acquire {
        duration_ms,
        takeover,
    } = &job.input.command
    {
        let pending = host.lock().map_err(|_| Problem::Unknown)?.connect(
            Grant {
                principal: *job.session.as_bytes(),
                origin: Origin::Remote,
                duration_ms: *duration_ms,
            },
            *takeover,
            ttl,
        )?;
        let client = resolve(|timeout| pending.wait(timeout))?;
        let outcome = json!({"kind":"acquired","state":projection::state(client.acquired_state())});
        return Ok(Completion {
            outcome,
            change: Change::Set(Binding {
                client: Arc::new(client),
                next: Some(1),
            }),
        });
    }
    let binding = job.binding.as_ref().ok_or(Problem::NoControl)?;
    match &job.input.command {
        Command::Submit {
            expected_revision,
            action,
        } => {
            let serial = binding.next.ok_or(Problem::Exhausted)?;
            let pending = binding.client.submit(
                serial,
                expected_revision.0,
                action.action().map_err(|_| Problem::Invalid)?,
                ttl,
            )?;
            let receipt = resolve(|timeout| pending.wait(timeout))?;
            // A business refusal still consumed a runtime serial. Admission/lease errors did not.
            let outcome = match receipt.result {
                Ok(()) => json!({"kind":"applied","state":projection::state(receipt.state)}),
                Err(code) => {
                    json!({"kind":"rejected","code":format!("{code:?}"),"message":code.to_string(),"state":projection::state(receipt.state)})
                }
            };
            Ok(Completion {
                outcome,
                change: Change::Set(Binding {
                    client: binding.client.clone(),
                    next: serial.checked_add(1),
                }),
            })
        }
        Command::Renew { duration_ms } => {
            let pending = binding.client.renew(*duration_ms, ttl)?;
            resolve(|timeout| pending.wait(timeout))?;
            Ok(Completion {
                outcome: json!({"kind":"renewed"}),
                change: Change::Keep,
            })
        }
        Command::Release {} => {
            let pending = binding.client.release(ttl)?;
            resolve(|timeout| pending.wait(timeout))?;
            Ok(Completion {
                outcome: json!({"kind":"released"}),
                change: Change::Clear,
            })
        }
        Command::Acquire { .. } => Err(Problem::Invalid),
    }
}
