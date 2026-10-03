use crate::{Detail, Error, Failure, Frame, Operation, Program, Reply, Request, Step, codec};
use stagemaster_runtime::State;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observation {
    pub boot: [u8; 16],
    pub revision: u64,
    pub observed_ms: u64,
    pub program_count: u16,
    pub step_count: u16,
}
/// Page replies deliberately omit the full State to fit maximal names in one record.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    State {
        state: State,
        result: Result<(), Failure>,
    },
    Program(Option<Program>),
    Step(Option<Step>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Response {
    pub request: Request,
    pub observed: Observation,
    pub body: Body,
}
impl From<Reply> for Response {
    fn from(value: Reply) -> Self {
        let body = match value.result {
            Ok(Detail::State) => Body::State {
                state: value.state,
                result: Ok(()),
            },
            Err(error) => Body::State {
                state: value.state,
                result: Err(error),
            },
            Ok(Detail::Program(value)) => Body::Program(value),
            Ok(Detail::Step(value)) => Body::Step(value),
        };
        Self {
            request: value.request,
            observed: Observation {
                boot: value.state.boot,
                revision: value.state.revision,
                observed_ms: value.state.observed_ms,
                program_count: value.program_count,
                step_count: value.step_count,
            },
            body,
        }
    }
}
impl Response {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        codec::valid_id(self.observed.boot)?;
        let valid = match self.body {
            Body::State { state, result } => {
                state.boot == self.observed.boot
                    && state.revision == self.observed.revision
                    && state.observed_ms == self.observed.observed_ms
                    && (result.is_err()
                        || !matches!(
                            self.request.operation,
                            Operation::Catalog { .. } | Operation::Step { .. }
                        ))
            }
            Body::Program(entry) => match self.request.operation {
                Operation::Catalog { index } => {
                    self.page(index, self.observed.program_count, entry.is_some())
                }
                _ => false,
            },
            Body::Step(entry) => match self.request.operation {
                Operation::Step { index } => {
                    self.page(index, self.observed.step_count, entry.is_some())
                }
                _ => false,
            },
        };
        if valid {
            Ok(())
        } else {
            Err(Error::Correlation)
        }
    }
    fn page(&self, index: u16, count: u16, present: bool) -> bool {
        self.request.expected_revision == self.observed.revision
            && (if present {
                index < count
            } else {
                index == count
            })
    }
    /// Match the full expected request and boot before applying this historical observation.
    /// # Errors
    /// Refuse wrong sessions, IDs, intent, revision, boot or reply shape.
    pub fn correlate(&self, request: Request, boot: [u8; 16]) -> Result<(), Error> {
        self.validate()?;
        if self.request == request && self.observed.boot == boot {
            Ok(())
        } else {
            Err(Error::Correlation)
        }
    }
    /// # Errors
    /// Invalid response shapes and oversized messages are never truncated.
    pub fn encode(&self) -> Result<Frame, Error> {
        codec::encode(3, |e| codec::response::write(e, self))
    }
    /// # Errors
    /// Reject invalid metadata, unknown versions and incomplete or trailing data.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        codec::decode(bytes, 3, codec::response::read)
    }
}
