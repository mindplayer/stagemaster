//! Do not freeze a connection's capabilities while persistent recovery is pending.
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Timer, with_timeout};

pub(super) async fn wait() {
    let state = with_timeout(Duration::from_secs(10), async {
        loop {
            let state = crate::installation::READY.load(Ordering::Acquire);
            if state != 0 {
                return state;
            }
            Timer::after_millis(10).await;
        }
    })
    .await
    .expect("设备恢复超时，未开放蓝牙连接");
    match state {
        1 => esp_println::println!("GATT worker ready; opening discovery"),
        2 => esp_println::println!("GATT worker failed; diagnostic discovery only"),
        _ => panic!("设备恢复状态无效"),
    }
}
