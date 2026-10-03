use stagemaster_live::{Change, Command, Key, SourceInfo};
use stagemaster_runtime::{Code, Owner, Receipt};
use stagemaster_runtime_host::Profile;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patch(Arc<[Change]>);
impl Patch {
    /// Allocate a bounded immutable command before queue admission; worker clones share its storage.
    /// # Errors
    /// Reject more than 512 entries or duplicate/out-of-range indices; backend checks actual layout too.
    pub fn new(values: &[Change]) -> Result<Self, Code> {
        if values.len() > 512 {
            return Err(Code::Budget);
        }
        let mut seen = [false; 512];
        for v in values {
            if v.attribute >= 512 || seen[v.attribute] {
                return Err(Code::State);
            }
            seen[v.attribute] = true;
        }
        Ok(Self(values.into()))
    }
    pub(crate) fn values(&self) -> &[Change] {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Control {
        source: Key,
        command: Command,
    },
    Patch {
        source: Key,
        patch: Patch,
    },
    Level {
        source: Key,
        level: u16,
    },
    ActivateMedia {
        ticket: crate::media::Activation,
    },
    StopMedia {
        group: stagemaster_live::media::GroupKey,
    },
    RequestMedia {
        group: stagemaster_live::media::GroupKey,
        command: crate::media::MediaCommand,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub boot: [u8; 16],
    pub revision: u64,
    pub observed_ms: u64,
    pub layout: [u8; 32],
    pub owner: Option<Owner>,
    pub sources: [Option<SourceInfo>; 64],
    pub media: [Option<crate::media::MediaState>; 64],
    pub fault: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameInfo {
    pub boot: [u8; 16],
    pub revision: u64,
    pub layout: [u8; 32],
    pub sampled_ms: u64,
    pub sequence: u64,
    pub universe: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Live;
impl Profile for Live {
    type Action = Action;
    type State = State;
    type Receipt = Receipt<Action, State>;
    type FrameInfo = FrameInfo;
}
