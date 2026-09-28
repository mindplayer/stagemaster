use super::Result;
use btleplug::{
    api::{
        Central, CentralEvent, CentralState, Manager as _, Peripheral as _,
        RetrievePeripheralsOptions, ScanFilter,
    },
    platform::{Adapter, Manager, Peripheral},
};
use futures_util::StreamExt;
use std::time::Duration;
use uuid::Uuid;
pub const DIAGNOSTIC: Uuid = Uuid::from_u128(0xf889ed60_0100_4e83_968e_799ab99558fa);
pub const SERVICE: Uuid = Uuid::from_u128(0xf889ed90_0100_4e83_968e_799ab99558fa);
pub const WRITE: Uuid = Uuid::from_u128(0xf889ed92_0100_4e83_968e_799ab99558fa);
pub const NOTIFY: Uuid = Uuid::from_u128(0xf889ed93_0100_4e83_968e_799ab99558fa);
pub fn diagnostic(last: u128) -> Uuid {
    Uuid::from_u128(0xf889ed60_0100_4e83_968e_799ab99558fa + (last << 96))
}

pub async fn adapter() -> Result<Adapter> {
    let adapter = Manager::new()
        .await?
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or("没有蓝牙适配器")?;
    if adapter.adapter_state().await? != CentralState::PoweredOn {
        return Err("系统蓝牙未就绪".into());
    }
    Ok(adapter)
}
pub async fn discover(adapter: &Adapter) -> Result<Peripheral> {
    let mut events = adapter.events().await?;
    adapter
        .start_scan(ScanFilter {
            services: vec![DIAGNOSTIC],
        })
        .await?;
    let found = tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let event = events.next().await.ok_or("蓝牙事件流结束")?;
            let (CentralEvent::DeviceDiscovered(id)
            | CentralEvent::DeviceUpdated(id)
            | CentralEvent::ServicesAdvertisement { id, .. }
            | CentralEvent::RssiUpdate { id, .. }) = event
            else {
                continue;
            };
            let peripheral = adapter.peripheral(&id).await?;
            if peripheral
                .properties()
                .await?
                .is_some_and(|p| p.services.contains(&DIAGNOSTIC))
            {
                return Ok::<_, Box<dyn std::error::Error>>(peripheral);
            }
        }
    })
    .await;
    adapter.stop_scan().await?;
    let peripheral = found??;
    // Refresh the CoreBluetooth handle after each disconnect instead of reusing its cache.
    let id = peripheral.id();
    match adapter
        .retrieve_peripherals(RetrievePeripheralsOptions {
            identifiers: Some(vec![id.clone()]),
            services: None,
        })
        .await
    {
        Ok(found) => found
            .into_iter()
            .find(|p| p.id() == id)
            .ok_or_else(|| "重新发现的设备不可用".into()),
        Err(btleplug::Error::NotSupported(_)) => Ok(peripheral),
        Err(error) => Err(error.into()),
    }
}
