mod application;
mod discovery;
mod installation;
mod notifications;
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
    installation: Option<installation::Channel>,
    application: Option<application::Channel>,
    credentials: Option<std::sync::Arc<crate::DevelopmentConfiguration>>,
    bonded_experiment: bool,
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
        self.scan().await
    }
    async fn discover(&mut self) -> Result<Candidate, Problem> {
        self.next_candidate().await
    }
    async fn stop_scan(&mut self) -> Result<(), Problem> {
        self.stop().await
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
        if let Some(characteristic) = &self.description {
            let bytes = peripheral.read(characteristic).await.map_err(error)?;
            let desc =
                stagemaster_device_info::Description::decode(&bytes).map_err(installation::wire)?;
            if desc.authentication == stagemaster_device_session::AUTHENTICATION {
                if let Some(config) = self
                    .credentials
                    .as_ref()
                    .filter(|c| c.device() == desc.device)
                    && desc.declares(stagemaster_device_info::capability::INSTALLATION)
                {
                    self.application = Some(application::prepare(&peripheral, desc, config).await?);
                }
            } else if self.bonded_experiment {
                self.installation = installation::Channel::prepare(&peripheral, &bytes).await?;
            }
        }
        Ok(())
    }
    fn installation_peer(&self) -> Option<crate::InstallationPeer> {
        if let Some(channel) = &self.application {
            return channel.peer().map(|receipt| crate::InstallationPeer {
                device: receipt.device,
                boot: receipt.boot,
                session: receipt.session,
                authentication: stagemaster_device_session::AUTHENTICATION,
                fragment_bytes: stagemaster_device_link::management::MESSAGE_BYTES,
                message_bytes: stagemaster_device_link::management::MESSAGE_BYTES,
            });
        }
        self.installation
            .as_ref()
            .and_then(installation::Channel::peer)
    }
    async fn write_installation(&mut self, bytes: &[u8]) -> Result<(), Problem> {
        let peripheral = self.pending.as_ref().ok_or_else(|| Problem::new(C::Lost))?;
        if let Some(channel) = &mut self.application {
            return channel.write(bytes).await.map_err(installation::wire);
        }
        self.installation
            .as_mut()
            .ok_or_else(|| Problem::new(C::Installation))?
            .write(peripheral, bytes)
            .await
    }
    fn try_installation_notification(&mut self) -> Result<Option<Vec<u8>>, Problem> {
        if let Some(channel) = &mut self.application {
            return channel.receive().map_err(installation::wire);
        }
        match &mut self.installation {
            Some(channel) => channel.receive(),
            None => Ok(None),
        }
    }
    async fn write(&mut self, bytes: &[u8; 20]) -> Result<(), Problem> {
        if let Some(channel) = &mut self.application {
            channel.heartbeat().await.map_err(installation::wire)?;
        }
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
        self.application = None;
        self.installation = None;
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
        let bytes = match &self.description {
            Some(characteristic) => Some(peripheral.read(characteristic).await.map_err(error)?),
            None => None,
        };
        if let Some(channel) = &self.installation {
            channel.correlate(
                bytes
                    .as_deref()
                    .ok_or_else(|| Problem::new(C::Description))?,
            )?;
        }
        if let Some(channel) = &self.application {
            channel
                .correlate(
                    bytes
                        .as_deref()
                        .ok_or_else(|| Problem::new(C::Description))?,
                )
                .map_err(installation::wire)?;
        }
        Ok(bytes)
    }
}
impl Ble {
    /// Historical LESC experiment only; never enable in ordinary product startup.
    #[cfg(feature = "bonded-experiment")]
    #[must_use]
    pub fn experimental_bonded() -> Self {
        Self {
            bonded_experiment: true,
            ..Self::default()
        }
    }
    /// Explicit development credentials; ordinary construction remains diagnostic-only
    /// for an application-authenticated device without a matching trusted identity.
    #[must_use]
    pub fn with_development_configuration(config: crate::DevelopmentConfiguration) -> Self {
        Self {
            credentials: Some(std::sync::Arc::new(config)),
            ..Self::default()
        }
    }
    fn active(&self) -> Result<(&Peripheral, &[Characteristic; 3]), Problem> {
        self.pending
            .as_ref()
            .zip(self.characteristics.as_ref())
            .ok_or_else(|| Problem::new(C::Lost))
    }
}
