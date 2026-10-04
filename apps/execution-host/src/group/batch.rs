use super::{Catalog, wire::identity};
use crate::wire::Failure;
use serde::Deserialize;
use stagemaster_live::BatchCommand;
use stagemaster_live_host::{Action, Batch};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Operation {
    Pause {},
    Resume {},
    Stop {},
}
impl Operation {
    pub fn command(&self) -> BatchCommand {
        match self {
            Self::Pause {} => BatchCommand::Pause,
            Self::Resume {} => BatchCommand::Resume,
            Self::Stop {} => BatchCommand::Stop,
        }
    }
}
pub(crate) fn validate(sources: &[String]) -> Result<(), Failure> {
    if sources.is_empty() || sources.len() > 64 {
        return Err(Failure::invalid());
    }
    for (i, id) in sources.iter().enumerate() {
        identity(id)?;
        if sources[..i].contains(id) {
            return Err(Failure::invalid());
        }
    }
    Ok(())
}
pub(crate) fn action(
    catalog: &Catalog,
    sources: &[String],
    command: &Operation,
) -> Result<Action, Failure> {
    validate(sources)?;
    let keys = sources
        .iter()
        .map(|id| {
            let id = identity(id)?;
            let entry = catalog
                .entries
                .iter()
                .find(|e| e.id == id)
                .ok_or_else(Failure::invalid)?;
            if entry.manual || entry.steps.is_empty() {
                return Err(Failure::invalid());
            }
            Ok(entry.key)
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    Ok(Action::Batch {
        batch: Batch::new(&keys).map_err(|_| Failure::invalid())?,
        command: command.command(),
    })
}
