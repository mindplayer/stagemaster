use super::*;
use serde_json::Value;
use stagemaster_runtime_host::{Frame, Profile};

// Fault-injected trusted adapter, not a real media/HTTP or hardware acceptance fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Refusing;
impl Profile for Refusing {
    type Action = ();
    type State = ();
    type Receipt = ();
    type FrameInfo = ();
}
impl Application for Refusing {
    const PROTOCOL: u8 = 2;
    type Context = Failure;
    fn action(_: &Operation, failure: &Failure) -> Result<(), Failure> {
        Err(*failure)
    }
    fn state((): &(), _: &Failure) -> Value {
        json!(null)
    }
    fn receipt((): (), _: &Failure) -> Value {
        json!(null)
    }
    fn frame(_: &Frame<Self>) -> Value {
        json!(null)
    }
}
fn refusal(failure: Failure) -> Completion<Refusing> {
    let error = translated::<Refusing>(&Operation::Stop {}, &failure).unwrap_err();
    rejected(error)
}
fn check(failure: Failure) {
    let result = refusal(failure);
    assert_eq!(result.outcome["kind"], "rejected");
    assert_eq!(result.outcome["code"], failure.1);
    assert_eq!(result.outcome["message"], failure.2);
    assert!(result.outcome.get("state").is_none());
    assert!(matches!(result.change, Change::Keep));
}
#[test]
fn a_busy_adapter_is_not_an_invalid_user_operation() {
    check(Failure::busy());
}
#[test]
fn a_closed_adapter_is_not_an_invalid_user_operation() {
    check(Failure::closed());
}
#[test]
fn invalid_parameters_remain_explicitly_rejected() {
    check(Failure::invalid());
}
#[test]
fn an_unknown_runtime_result_still_clears_the_binding_and_has_no_invented_state() {
    let result = rejected::<Refusing>(Problem::Unknown);
    assert_eq!(result.outcome["kind"], "unknown");
    assert!(result.outcome.get("state").is_none());
    assert!(matches!(result.change, Change::Clear));
}
