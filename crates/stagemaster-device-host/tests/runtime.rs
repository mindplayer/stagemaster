// Reuse the existing real secure-session/package fixtures, not a second protocol implementation.
#[path = "runtime/adapter.rs"]
mod adapter;
#[path = "runtime/failures.rs"]
mod failures;
#[path = "runtime/flows.rs"]
mod flows;
#[path = "../../stagemaster-device-channel/tests/runtime_support/mod.rs"]
mod runtime_support;
#[path = "../../stagemaster-device-channel/tests/support/mod.rs"]
mod support;
