use stagemaster_execution_client::{Client, View};
use std::time::{Duration, Instant};

pub const BUSY: &str = "后台请求未成功（503），请核对连接与原回执";
pub async fn wait(client: &mut Client, predicate: impl Fn(&View) -> bool) -> View {
    let end = Instant::now() + Duration::from_secs(6);
    loop {
        let view = match client.refresh().await {
            Ok(view) => view,
            Err(error) if error == BUSY => {
                assert!(Instant::now() < end, "{error}");
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }
            Err(error) => panic!("{error}"),
        };
        if predicate(&view) {
            return view;
        }
        assert!(Instant::now() < end, "{view:?}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
pub async fn settled(client: &mut Client) -> View {
    wait(client, |v| !v.pending).await
}

pub async fn confirmed_control(
    client: &mut Client,
    submitted: Result<View, String>,
    expected: &str,
) -> View {
    if let Err(error) = submitted {
        assert_eq!(error, BUSY);
    }
    // Follow only the already-sent serial; a refused admission has no accepted receipt.
    let serial = client.view().record.unwrap().serial;
    let completed = settled(client).await;
    let record = completed.record.as_ref().unwrap();
    assert_eq!(record.serial, serial);
    assert_eq!(record.outcome.as_ref().unwrap().kind, expected);
    completed
}
