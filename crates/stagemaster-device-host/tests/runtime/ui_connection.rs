use crate::adapter::*;
use stagemaster_device_auth::application::{Configuration, Role};
use stagemaster_device_host::{
    Phase, Request as DeviceRequest,
    runtime_ui::{ExpectedAccess, Request},
};
use stagemaster_device_session::SecretKey;
use std::time::Duration;

// Fixed test keys only. No credentials are read from the user's environment.
fn configuration(version: u8, scopes: u8) -> Configuration {
    let mut bytes = [0; 160];
    bytes[..8].copy_from_slice(b"SMDV\x01\x02\xa0\0");
    bytes[4] = version;
    bytes[8..24].fill(1);
    bytes[24..40].fill(9);
    bytes[40..48].copy_from_slice(&3_u64.to_le_bytes());
    bytes[48..52].copy_from_slice(&60_000_u32.to_le_bytes());
    bytes[52] = scopes;
    bytes[56..88].fill(4);
    bytes[88..120].copy_from_slice(&SecretKey::import([5; 32]).unwrap().public());
    bytes[120..152].copy_from_slice(&SecretKey::import([4; 32]).unwrap().public());
    Configuration::import(&bytes, Role::Controller).unwrap()
}

#[tokio::test(start_paused = true)]
async fn json_connect_uses_explicit_native_scopes_without_legacy_upgrade_or_auto_acquire() {
    for (version, scopes) in [(1, 0), (2, 2), (2, 6)] {
        let (host, data) = setup();
        host.request(DeviceRequest::Scan {
            epoch: status(&host).epoch,
        })
        .unwrap();
        settle().await;
        tokio::time::advance(Duration::from_secs(8)).await;
        settle().await;
        let expected = ExpectedAccess::from_configuration(&configuration(version, scopes));
        let result = host
            .runtime_request(
                Request::Connect {
                    epoch: status(&host).epoch,
                    id: "runtime".into(),
                },
                &expected,
            )
            .await;
        if version == 1 {
            assert!(result.is_err());
            assert_eq!(data.lock().unwrap().connects, 0);
        } else {
            let connecting = result.unwrap();
            assert_eq!(connecting.epoch, status(&host).epoch);
            wait_for_settled(&host).await;
            assert_eq!(status(&host).phase, Phase::Connected);
            let reply = host
                .runtime_request(
                    Request::Refresh {
                        epoch: connecting.epoch,
                    },
                    &expected,
                )
                .await
                .unwrap();
            let json = serde_json::to_value(reply).unwrap();
            assert_eq!(json["peer"]["control"], scopes == 6);
            assert_eq!(json["peer"]["installation"], false);
            assert!(json["reply"]["body"]["state"]["owner"].is_null());
            assert!(json["reply"]["body"]["state"]["instance"].is_null());
        }
        host.shutdown().await.unwrap();
    }
}
