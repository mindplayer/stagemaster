use crate::{
    group::{Catalog, wire::identity},
    wire::{Decimal, Failure},
};
use serde::Deserialize;
use serde_json::{Value, json};
use stagemaster_live_host::{
    Action, State,
    media::{ControlFailure, MediaCommand, Termination},
};
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Operation {
    Play {},
    Pause {},
    Stop {},
    Recover {
        position_ms: u64,
    },
    Seek {
        position_ms: u64,
        playing: bool,
    },
    ExitLoop {
        instance: Decimal,
        region: usize,
        pass: Decimal,
        requested: bool,
    },
}
pub(crate) fn action(
    catalog: &Catalog,
    group: &str,
    generation: u64,
    operation: &Operation,
) -> Result<Action, Failure> {
    let owner = catalog.media.as_ref().ok_or_else(Failure::invalid)?;
    let id = identity(group)?;
    let state = owner
        .observer
        .read()
        .map_err(|_| Failure::busy())?
        .snapshot
        .ok_or_else(Failure::closed)?
        .state;
    let group = state
        .media
        .iter()
        .flatten()
        .find(|m| m.group.id == *id.as_bytes() && m.group.key.generation() == generation)
        .ok_or_else(Failure::invalid)?
        .group;
    let command = match operation {
        Operation::Play {} => MediaCommand::Play,
        Operation::Pause {} => MediaCommand::Pause,
        Operation::Stop {} => MediaCommand::Stop,
        Operation::Recover { position_ms } => MediaCommand::Recover {
            position_ms: *position_ms,
        },
        Operation::ExitLoop {
            instance,
            region,
            pass,
            requested,
        } => {
            let view = owner.view();
            if instance.0 == 0
                || pass.0 == 0
                || *region >= stagemaster_project::MAX_AUDIO_LOOP_REGIONS
                || view["instance"]
                    .as_str()
                    .and_then(|v| v.parse::<u64>().ok())
                    != Some(instance.0)
                || view["loopState"]["region"].as_u64() != Some(*region as u64)
                || view["loopState"]["pass"]
                    .as_str()
                    .and_then(|v| v.parse::<u64>().ok())
                    != Some(pass.0)
            {
                return Err(Failure::invalid());
            }
            MediaCommand::ExitLoop {
                instance: instance.0,
                region: *region,
                pass: pass.0,
                requested: *requested,
            }
        }
        Operation::Seek {
            position_ms,
            playing,
        } => {
            if owner.view()["durationMs"]
                .as_u64()
                .is_none_or(|duration| *position_ms > duration)
            {
                return Err(Failure::invalid());
            }
            MediaCommand::Seek {
                position_ms: *position_ms,
                playing: *playing,
            }
        }
    };
    Ok(Action::RequestMedia {
        group: group.key,
        command,
    })
}
pub(crate) fn state(state: &State) -> Value {
    json!(state.media.iter().flatten().map(|media| {
        let control = media.control.map(|c| {
            let (status, problem) = match c.result {
                None => ("pending", None), Some(Ok(())) => ("applied", None),
                Some(Err(ControlFailure::TimedOut)) => ("timedOut", Some("音乐操作超时".to_owned())),
                Some(Err(ControlFailure::Provider(code))) => ("failed", Some(code.to_string())),
            };
            json!({"request":c.request.ticket.serial().to_string(),"status":status,"problem":problem})
        });
        json!({"id":Uuid::from_bytes(media.group.id).to_string(),"generation":media.group.key.generation().to_string(),
            "status":format!("{:?}",media.group.status),"positionMs":media.group.position_ms,"control":control,
            "termination":media.termination.map(|r|json!({"generation":r.generation.to_string(),"reason":match r.reason {Termination::Ended=>"ended",Termination::Failed(_)=>"failed"},"applied":r.result.is_ok()}))})
    }).collect::<Vec<_>>())
}
