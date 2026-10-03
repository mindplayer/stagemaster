use crate::{adapter::*, runtime_support::rights};
use serde_json::to_value;
use stagemaster_device_host::runtime_ui::Reply;
use stagemaster_runtime::{Instance, Lease, Origin, Owner};
use stagemaster_runtime_protocol::{Body, Operation};

#[tokio::test(start_paused = true)]
async fn json_projection_preserves_full_width_counters_and_times() {
    let (host, _) = setup();
    connect(&host, rights()).await;
    let mut response = command(&host, Operation::Status).await;
    response.request.id = u64::MAX;
    response.observed.revision = u64::MAX;
    response.observed.observed_ms = u64::MAX;
    let Body::State { ref mut state, .. } = response.body else {
        panic!("status reply expected");
    };
    state.revision = u64::MAX;
    state.observed_ms = u64::MAX;
    state.elapsed_ms = u64::MAX;
    state.instance = Some(Instance {
        boot: state.boot,
        number: u64::MAX,
    });
    state.owner = Some(Owner {
        lease: Lease {
            boot: state.boot,
            epoch: u64::MAX,
        },
        principal: [1; 16],
        origin: Origin::Remote,
        expires_ms: u64::MAX,
        serial: u64::MAX,
    });
    let json = to_value(Reply::from(response)).unwrap();
    for path in [
        "/id",
        "/revision",
        "/observedMs",
        "/body/state/elapsedMs",
        "/body/state/instance",
        "/body/state/owner/lease",
        "/body/state/owner/expiresMs",
    ] {
        assert_eq!(
            json.pointer(path).unwrap(),
            "18446744073709551615",
            "{path}"
        );
    }
    assert_eq!(json["boot"].as_str().unwrap().len(), 32);
    host.shutdown().await.unwrap();
}
