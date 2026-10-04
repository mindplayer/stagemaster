use crate::wire::Failure;
use serde::Deserialize;
use stagemaster_live::OutputCommand;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Operation {
    Level { percent: u8 },
    Blackout { enabled: bool },
}
impl Operation {
    pub fn command(&self) -> Result<OutputCommand, Failure> {
        match *self {
            Self::Level { percent } if percent <= 100 => Ok(OutputCommand::Level { percent }),
            Self::Blackout { enabled } => Ok(OutputCommand::Blackout { enabled }),
            Self::Level { .. } => Err(Failure::invalid()),
        }
    }
}
