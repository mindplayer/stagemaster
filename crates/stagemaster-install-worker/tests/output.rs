#![cfg(feature = "output")]
#[path = "output_cases/failures.rs"]
mod failures;
#[path = "output_cases/lifecycle.rs"]
mod lifecycle;
#[path = "output_cases/maintenance.rs"]
mod maintenance;
#[allow(dead_code)]
#[path = "maintenance_support/mod.rs"]
mod project;
#[path = "output_cases/support.rs"]
mod support;
#[path = "../../stagemaster-output-port/tests/dmx_support/mod.rs"]
mod wire;
