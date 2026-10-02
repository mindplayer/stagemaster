//! Trusted application projection, outside the scheduling worker and network authority checks.
use crate::{
    projection,
    wire::{Failure, Operation},
};
use serde_json::{Value, json};
use stagemaster_runtime_host::{Device, Frame, Profile};

pub(crate) trait Application: Profile {
    const PROTOCOL: u8;
    type Context: Send + Sync + 'static;
    fn action(operation: &Operation, context: &Self::Context) -> Result<Self::Action, Failure>;
    fn state(state: &Self::State, context: &Self::Context) -> Value;
    fn receipt(receipt: Self::Receipt, context: &Self::Context) -> Value;
    fn frame(frame: &Frame<Self>) -> Value;
    fn project(_context: &Self::Context) -> Option<&[u8]> {
        None
    }
}
impl Application for Device {
    const PROTOCOL: u8 = 1;
    type Context = ();
    fn action(operation: &Operation, (): &()) -> Result<Self::Action, Failure> {
        operation.action()
    }
    fn state(state: &Self::State, (): &()) -> Value {
        projection::state(*state)
    }
    fn receipt(receipt: Self::Receipt, (): &()) -> Value {
        outcome(receipt.result, projection::state(receipt.state))
    }
    fn frame(f: &Frame<Self>) -> Value {
        json!({"kind":"softwareSample","universe":f.info.universe,"revision":f.info.revision.to_string(),"sampledMs":f.info.sampled_ms.to_string(),"slots":f.slots.as_slice()})
    }
}
pub(crate) fn outcome(result: Result<(), stagemaster_runtime::Code>, state: Value) -> Value {
    let mut value = match result {
        Ok(()) => json!({"kind":"applied"}),
        Err(code) => {
            json!({"kind":"rejected","code":format!("{code:?}"),"message":code.to_string()})
        }
    };
    value["state"] = state;
    value
}
