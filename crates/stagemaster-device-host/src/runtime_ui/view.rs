use super::Key;
use crate::{RuntimeSnapshot, description::hex};
use serde::Serialize;
use stagemaster_runtime::{Mode, State, Status};
use stagemaster_runtime_protocol::{Body, Response};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub epoch: u32,
    pub connection_epoch: Option<u32>,
    pub peer: Option<Peer>,
    pub pending: bool,
    pub last_response: Option<Reply>,
    pub reply: Option<Reply>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    pub device: String,
    pub boot: String,
    pub session: String,
    pub control: bool,
    pub installation: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub id: String,
    pub boot: String,
    pub revision: String,
    pub observed_ms: String,
    pub program_count: u16,
    pub step_count: u16,
    pub body: Content,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Content {
    State {
        state: StateView,
        error: Option<String>,
    },
    Program {
        index: u16,
        program: Option<Program>,
    },
    Step {
        index: u16,
        step: Option<Step>,
    },
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    pub key: Key,
    pub name: String,
    pub loader_bytes: u32,
}
#[derive(Clone, Debug, Serialize)]
pub struct Step {
    pub id: String,
    pub name: String,
    pub number: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateView {
    pub mode: &'static str,
    pub package: Option<String>,
    pub selected: Option<Key>,
    pub loaded: Option<Key>,
    pub status: Option<&'static str>,
    pub instance: Option<String>,
    pub step: Option<String>,
    pub elapsed_ms: String,
    pub owner: Option<Owner>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub lease: String,
    pub expires_ms: String,
}
impl From<State> for StateView {
    fn from(s: State) -> Self {
        Self {
            mode: match s.mode {
                Mode::Operation => "operation",
                Mode::Quiescing => "quiescing",
                Mode::Maintenance => "maintenance",
            },
            package: s.bound_package.map(|c| hex(&c.identity.digest)),
            selected: s.selected.map(Into::into),
            loaded: s.loaded.map(Into::into),
            status: s.status.map(|s| match s {
                Status::Running => "running",
                Status::Paused => "paused",
                Status::Idle => "idle",
                Status::Finished => "finished",
            }),
            instance: s.instance.map(|i| i.number.to_string()),
            step: s.step.map(|s| hex(&s)),
            elapsed_ms: s.elapsed_ms.to_string(),
            owner: s.owner.map(|o| Owner {
                lease: o.lease.epoch.to_string(),
                expires_ms: o.expires_ms.to_string(),
            }),
        }
    }
}
impl From<Response> for Reply {
    fn from(r: Response) -> Self {
        let body = match r.body {
            Body::State { state, result } => Content::State {
                state: state.into(),
                error: result.err().map(|e| e.to_string()),
            },
            Body::Program(p) => Content::Program {
                index: match r.request.operation {
                    stagemaster_runtime_protocol::Operation::Catalog { index } => index,
                    _ => unreachable!(),
                },
                program: p.map(|p| Program {
                    key: p.key.into(),
                    name: p.name.as_str().into(),
                    loader_bytes: p.loader_bytes,
                }),
            },
            Body::Step(s) => Content::Step {
                index: match r.request.operation {
                    stagemaster_runtime_protocol::Operation::Step { index } => index,
                    _ => unreachable!(),
                },
                step: s.map(|s| Step {
                    id: hex(&s.id),
                    name: s.name.as_str().into(),
                    number: s.number.as_str().into(),
                }),
            },
        };
        Self {
            id: r.request.id.to_string(),
            boot: hex(&r.observed.boot),
            revision: r.observed.revision.to_string(),
            observed_ms: r.observed.observed_ms.to_string(),
            program_count: r.observed.program_count,
            step_count: r.observed.step_count,
            body,
        }
    }
}
impl View {
    pub(super) fn new(epoch: u32, snapshot: &RuntimeSnapshot, reply: Option<&Response>) -> Self {
        Self {
            epoch,
            connection_epoch: snapshot.connection_epoch,
            peer: snapshot.peer.map(|p| Peer {
                device: hex(&p.peer.device),
                boot: hex(&p.peer.boot),
                session: hex(&p.peer.session),
                control: p.access.control,
                installation: p.access.installation,
            }),
            pending: snapshot.pending.is_some(),
            last_response: snapshot.last_response.map(Into::into),
            reply: reply.copied().map(Into::into),
        }
    }
}
