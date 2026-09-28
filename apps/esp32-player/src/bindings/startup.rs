use super::{Action, Client};
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Instant, Timer};
use stagemaster_device_auth::Vault;

pub async fn recover(client: &mut Client) -> Option<Vault> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match crate::installation::READY.load(Ordering::Acquire) {
            1 => break,
            2 => return None,
            _ if Instant::now() >= deadline => return None,
            _ => Timer::after_millis(50).await,
        }
    }
    let recovered = client.call(Action::Recover).await;
    #[cfg(feature = "binding-local-test")]
    let recovered = match recovered {
        Ok(vault) => Ok(vault),
        Err(_) => {
            // Explicit test image only; initialize itself refuses any nonblank byte.
            use stagemaster_device_auth::{Address, LocalIdentity, Secret};
            let mut address = [0; 6];
            esp_hal::rng::Rng::new().read(&mut address);
            address[5] |= 0xc0;
            let local = LocalIdentity::new(
                Address::new(true, address).ok()?,
                Secret::new(super::random()).ok()?,
            )
            .ok()?;
            esp_println::println!("本地绑定验收镜像：仅尝试初始化全空白绑定区");
            client.call(Action::Initialize(local)).await
        }
    };
    match recovered {
        Ok(vault) => {
            esp_println::println!(
                "绑定档案已核验 generation={} peers={}",
                vault.generation(),
                vault.bindings().count()
            );
            Some(vault)
        }
        Err(error) => {
            esp_println::println!("绑定不可用：{:?}；仅保留诊断连接", error);
            None
        }
    }
}
