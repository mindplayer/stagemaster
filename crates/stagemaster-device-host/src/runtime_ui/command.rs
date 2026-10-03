use super::invalid;
use crate::Problem;
use serde::{Deserialize, Serialize};
use stagemaster_runtime::{Action as Core, ProgramKey};
use stagemaster_runtime_protocol::Operation;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Key {
    pub kind: Kind,
    pub id: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Scene,
    Sequence,
}
impl From<ProgramKey> for Key {
    fn from(key: ProgramKey) -> Self {
        Self {
            kind: match key.kind {
                stagemaster_package::Kind::Scene => Kind::Scene,
                stagemaster_package::Kind::Sequence => Kind::Sequence,
            },
            id: crate::description::hex(&key.id),
        }
    }
}
impl Key {
    fn decode(&self) -> Result<ProgramKey, Problem> {
        Ok(ProgramKey {
            kind: match self.kind {
                Kind::Scene => stagemaster_package::Kind::Scene,
                Kind::Sequence => stagemaster_package::Kind::Sequence,
            },
            id: identity(&self.id)?,
        })
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Connect {
        epoch: u32,
        id: String,
    },
    Snapshot {
        epoch: u32,
    },
    Refresh {
        epoch: u32,
    },
    Catalog {
        epoch: u32,
        revision: String,
        index: u16,
    },
    Step {
        epoch: u32,
        revision: String,
        index: u16,
    },
    Apply {
        epoch: u32,
        revision: String,
        action: Action,
    },
}
impl Request {
    pub(super) const fn epoch(&self) -> u32 {
        match self {
            Self::Connect { epoch, .. }
            | Self::Snapshot { epoch }
            | Self::Refresh { epoch }
            | Self::Catalog { epoch, .. }
            | Self::Step { epoch, .. }
            | Self::Apply { epoch, .. } => *epoch,
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Action {
    Acquire { takeover: bool },
    Renew {},
    Release {},
    Select { program: Key },
    Load {},
    Start { step: String },
    Pause {},
    Resume {},
    Next {},
    Stop {},
    BeginMaintenance {},
    CancelMaintenance {},
    FinishMaintenance {},
}
impl Action {
    pub(super) fn operation(&self, duration_ms: u64) -> Result<Operation, Problem> {
        Ok(match self {
            Self::Acquire { takeover } => Operation::Acquire {
                duration_ms,
                takeover: *takeover,
            },
            Self::Renew {} => Operation::Renew { duration_ms },
            Self::Release {} => Operation::Release,
            Self::FinishMaintenance {} => Operation::FinishMaintenance,
            Self::Select { program } => Operation::Apply(Core::Select(program.decode()?)),
            Self::Load {} => Operation::Apply(Core::Load),
            Self::Start { step } => Operation::Apply(Core::Start {
                step: identity(step)?,
            }),
            Self::Pause {} => Operation::Apply(Core::Pause),
            Self::Resume {} => Operation::Apply(Core::Resume),
            Self::Next {} => Operation::Apply(Core::Next),
            Self::Stop {} => Operation::Apply(Core::Stop),
            Self::BeginMaintenance {} => Operation::Apply(Core::BeginMaintenance),
            Self::CancelMaintenance {} => Operation::Apply(Core::CancelMaintenance),
        })
    }
}
pub(super) fn number(value: &str) -> Result<u64, Problem> {
    if value.is_empty()
        || value.len() > 20
        || !value.bytes().all(|b| b.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(invalid());
    }
    value.parse().map_err(|_| invalid())
}
fn identity(value: &str) -> Result<[u8; 16], Problem> {
    if value.len() != 32 || !value.is_ascii() {
        return Err(invalid());
    }
    let mut id = [0; 16];
    for (index, byte) in id.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(|_| invalid())?;
    }
    if id == [0; 16] {
        return Err(invalid());
    }
    Ok(id)
}
