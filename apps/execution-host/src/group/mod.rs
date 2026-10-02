mod prepare;
mod projection;
pub(crate) mod wire;
use crate::{
    application::{Application, outcome},
    wire::{Failure, Operation},
};
pub(crate) use prepare::prepare;
use serde_json::Value;
use stagemaster_live::{Change, Command, Key};
use stagemaster_live_host::{Action, Live, Patch};
use stagemaster_project::{CompiledOutput, CompiledStep};
use stagemaster_runtime_host::Frame;
use uuid::Uuid;

pub(crate) struct Entry {
    pub id: Uuid,
    pub key: Key,
    pub manual: bool,
    pub steps: Vec<CompiledStep>,
}
pub(crate) struct Catalog {
    pub entries: Vec<Entry>,
    pub output: CompiledOutput,
    pub project: Vec<u8>,
}
impl Application for Live {
    const PROTOCOL: u8 = 2;
    type Context = Catalog;
    fn action(operation: &Operation, catalog: &Catalog) -> Result<Action, Failure> {
        let Operation::Source { source, action } = operation else {
            return Err(Failure::invalid());
        };
        let id = wire::identity(source)?;
        let entry = catalog
            .entries
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(Failure::invalid)?;
        let source = entry.key;
        let command = match action {
            wire::Operation::Start { step } => Command::Execute(
                entry
                    .steps
                    .iter()
                    .position(|s| s.id == *step)
                    .ok_or_else(Failure::invalid)?,
            ),
            wire::Operation::Pause {} => Command::Pause,
            wire::Operation::Resume {} => Command::Resume,
            wire::Operation::Next {} => Command::Next,
            wire::Operation::Stop {} => Command::Stop,
            wire::Operation::Level { value } => {
                return Ok(Action::Level {
                    source,
                    level: *value,
                });
            }
            wire::Operation::Patch { changes } => {
                if !entry.manual {
                    return Err(Failure::invalid());
                }
                let changes = changes
                    .iter()
                    .map(|edit| {
                        let (attribute, value) = catalog
                            .output
                            .manual_value(
                                &edit.fixture_id,
                                &edit.attribute,
                                edit.value.semantic().as_ref(),
                            )
                            .map_err(|_| Failure::invalid())?;
                        Ok(Change { attribute, value })
                    })
                    .collect::<Result<Vec<_>, Failure>>()?;
                return Ok(Action::Patch {
                    source,
                    patch: Patch::new(&changes).map_err(|_| Failure::invalid())?,
                });
            }
        };
        if entry.manual && command != Command::Stop {
            return Err(Failure::invalid());
        }
        Ok(Action::Control { source, command })
    }
    fn state(state: &Self::State, catalog: &Catalog) -> Value {
        projection::state(state, catalog)
    }
    fn receipt(receipt: Self::Receipt, catalog: &Catalog) -> Value {
        outcome(receipt.result, projection::state(&receipt.state, catalog))
    }
    fn frame(frame: &Frame<Self>) -> Value {
        projection::frame(frame)
    }
    fn project(catalog: &Catalog) -> Option<&[u8]> {
        Some(&catalog.project)
    }
}
