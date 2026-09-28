use crate::{Candidate, MAX_CANDIDATES, Problem, ProblemCode as C, Transport};
use btleplug::{
    api::{
        Central, CentralEvent, CentralState, CharPropFlags, Characteristic, Manager as _,
        Peripheral as _, RetrievePeripheralsOptions, ScanFilter, WriteType,
    },
    platform::{Adapter, Manager, Peripheral},
};
use futures_util::{StreamExt, future::BoxFuture, stream::BoxStream};
use std::collections::BTreeMap;
use uuid::Uuid;

const SERVICE: Uuid = Uuid::from_u128(0xf889ed60_0100_4e83_968e_799ab99558fa);
const RX: Uuid = Uuid::from_u128(0xf889ed61_0100_4e83_968e_799ab99558fa);
const TX: Uuid = Uuid::from_u128(0xf889ed62_0100_4e83_968e_799ab99558fa);
const INFO: Uuid = Uuid::from_u128(0xf889ed63_0100_4e83_968e_799ab99558fa);
const DESCRIPTION: Uuid = Uuid::from_u128(0xf889ed64_0100_4e83_968e_799ab99558fa);

/// Lazy native adapter. Construction never asks for permissions or scans.
#[derive(Default)]
pub struct Ble {
    adapter: Option<Adapter>,
    // Retained across cancelled waits. Native initialization can be waiting on
    // the OS permission prompt; never create another central manager on retry.
    initializing: Option<BoxFuture<'static, Result<Adapter, Problem>>>,
    events: Option<BoxStream<'static, CentralEvent>>,
    found: BTreeMap<String, Peripheral>,
    pending: Option<Peripheral>,
    characteristics: Option<[Characteristic; 3]>,
    description: Option<Characteristic>,
    scanning: bool,
}
#[allow(clippy::needless_pass_by_value)] // Used directly as a Result::map_err adapter.
fn error(value: btleplug::Error) -> Problem {
    let code = match &value {
        btleplug::Error::PermissionDenied => C::Permission,
        btleplug::Error::NoAdapterAvailable => C::Adapter,
        btleplug::Error::TimedOut(_) => C::Timeout,
        _ => C::Lost,
    };
    Problem::new(code).detail(value.to_string())
}
impl Transport for Ble {
    async fn start_scan(&mut self) -> Result<(), Problem> {
        if self.adapter.is_none() {
            let initialization = self.initializing.get_or_insert_with(|| {
                Box::pin(async {
                    let manager = Manager::new().await.map_err(error)?;
                    manager
                        .adapters()
                        .await
                        .map_err(error)?
                        .into_iter()
                        .next()
                        .ok_or_else(|| Problem::new(C::Adapter))
                })
            });
            let result = initialization.await;
            self.initializing = None;
            self.adapter = Some(result?);
        }
        let adapter = self
            .adapter
            .as_ref()
            .ok_or_else(|| Problem::new(C::Adapter))?;
        match adapter.adapter_state().await.map_err(error)? {
            CentralState::PoweredOn => (),
            CentralState::PoweredOff => return Err(Problem::new(C::PoweredOff)),
            CentralState::Unknown => return Err(Problem::new(C::Adapter)),
        }
        self.found.clear();
        self.events = Some(adapter.events().await.map_err(error)?);
        self.scanning = true; // cleanup also runs after a cancelled start_scan future
        adapter
            .start_scan(ScanFilter {
                services: vec![SERVICE],
            })
            .await
            .map_err(error)
    }
    async fn discover(&mut self) -> Result<Candidate, Problem> {
        loop {
            let event = self
                .events
                .as_mut()
                .ok_or_else(|| Problem::new(C::Adapter))?
                .next()
                .await
                .ok_or_else(|| Problem::new(C::Adapter))?;
            let id = match event {
                CentralEvent::DeviceDiscovered(id)
                | CentralEvent::DeviceUpdated(id)
                | CentralEvent::ServicesAdvertisement { id, .. }
                | CentralEvent::RssiUpdate { id, .. } => id,
                CentralEvent::StateUpdate(CentralState::PoweredOff) => {
                    return Err(Problem::new(C::PoweredOff));
                }
                _ => continue,
            };
            let peripheral = self
                .adapter
                .as_ref()
                .ok_or_else(|| Problem::new(C::Adapter))?
                .peripheral(&id)
                .await
                .map_err(error)?;
            let Some(properties) = peripheral.properties().await.map_err(error)? else {
                continue;
            };
            if !properties.services.contains(&SERVICE) {
                continue;
            }
            let id = id.to_string();
            if self.found.len() < MAX_CANDIDATES || self.found.contains_key(&id) {
                self.found.insert(id.clone(), peripheral);
            }
            // Still report the extra candidate so the service can report truncation.
            return Ok(Candidate {
                id,
                name: properties
                    .local_name
                    .unwrap_or_else(|| "未命名设备".into())
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(80)
                    .collect(),
                rssi: properties.rssi,
            });
        }
    }
    async fn stop_scan(&mut self) -> Result<(), Problem> {
        if self.scanning {
            self.adapter
                .as_ref()
                .ok_or_else(|| Problem::new(C::Adapter))?
                .stop_scan()
                .await
                .map_err(error)?;
            self.scanning = false;
        }
        self.events = None;
        Ok(())
    }
    async fn connect(&mut self, id: &str) -> Result<(), Problem> {
        let discovered = self
            .found
            .get(id)
            .cloned()
            .ok_or_else(|| Problem::new(C::Unavailable))?;
        // CoreBluetooth drops its internal peripheral after disconnect. Resolve
        // the native identifier again so reconnect doesn't use a dead handle.
        let adapter = self
            .adapter
            .as_ref()
            .ok_or_else(|| Problem::new(C::Adapter))?;
        let peripheral = match adapter
            .retrieve_peripherals(RetrievePeripheralsOptions {
                identifiers: Some(vec![discovered.id()]),
                services: None,
            })
            .await
        {
            Ok(values) => values
                .into_iter()
                .find(|p| p.id() == discovered.id())
                .ok_or_else(|| Problem::new(C::Unavailable))?,
            Err(btleplug::Error::NotSupported(_)) => discovered,
            Err(value) => return Err(error(value)),
        };
        self.found.insert(id.to_owned(), peripheral.clone());
        self.pending = Some(peripheral.clone()); // retained even when connect is cancelled
        peripheral.connect().await.map_err(error)?;
        peripheral.discover_services().await.map_err(error)?;
        let characteristics = peripheral.characteristics();
        self.description = characteristics
            .iter()
            .find(|c| c.service_uuid == SERVICE && c.uuid == DESCRIPTION)
            .cloned();
        if self
            .description
            .as_ref()
            .is_some_and(|c| !c.properties.contains(CharPropFlags::READ))
        {
            return Err(Problem::new(C::Description));
        }
        let find = |uuid, flag| {
            characteristics
                .iter()
                .find(|c| {
                    c.service_uuid == SERVICE && c.uuid == uuid && c.properties.contains(flag)
                })
                .cloned()
                .ok_or_else(|| Problem::new(C::Protocol))
        };
        self.characteristics = Some([
            find(RX, CharPropFlags::WRITE)?,
            find(TX, CharPropFlags::READ)?,
            find(INFO, CharPropFlags::READ)?,
        ]);
        Ok(())
    }
    async fn write(&mut self, bytes: &[u8; 20]) -> Result<(), Problem> {
        let (peripheral, characteristics) = self.active()?;
        peripheral
            .write(&characteristics[0], bytes, WriteType::WithResponse)
            .await
            .map_err(error)
    }
    async fn reply(&mut self) -> Result<Vec<u8>, Problem> {
        let (peripheral, characteristics) = self.active()?;
        peripheral.read(&characteristics[1]).await.map_err(error)
    }
    async fn diagnostics(&mut self) -> Result<Vec<u8>, Problem> {
        let (peripheral, characteristics) = self.active()?;
        peripheral.read(&characteristics[2]).await.map_err(error)
    }
    async fn disconnect(&mut self) -> Result<(), Problem> {
        self.characteristics = None;
        self.description = None;
        if let Some(peripheral) = &self.pending {
            // Do not gate on is_connected: cancelling a pending connect is essential.
            match peripheral.disconnect().await {
                Ok(()) | Err(btleplug::Error::NotConnected) => (),
                Err(value) => return Err(error(value)),
            }
        }
        self.pending = None;
        Ok(())
    }
    async fn description(&mut self) -> Result<Option<Vec<u8>>, Problem> {
        let (peripheral, _) = self.active()?;
        match &self.description {
            Some(characteristic) => peripheral
                .read(characteristic)
                .await
                .map(Some)
                .map_err(error),
            None => Ok(None),
        }
    }
}
impl Ble {
    fn active(&self) -> Result<(&Peripheral, &[Characteristic; 3]), Problem> {
        self.pending
            .as_ref()
            .zip(self.characteristics.as_ref())
            .ok_or_else(|| Problem::new(C::Lost))
    }
}
