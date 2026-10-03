//! Physical connection and explicit application-mode selection.
use super::*;
use stagemaster_runtime_protocol::Access;

impl Ble {
    pub(super) async fn connect_mode(
        &mut self,
        id: &str,
        runtime: Option<Access>,
    ) -> Result<(), Problem> {
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
            self.prepare_application(&peripheral, &bytes, runtime)
                .await?;
        } else if runtime.is_some() {
            return Err(Problem::new(C::Runtime));
        }
        Ok(())
    }
    async fn prepare_application(
        &mut self,
        peripheral: &Peripheral,
        bytes: &[u8],
        runtime: Option<Access>,
    ) -> Result<(), Problem> {
        let desc =
            stagemaster_device_info::Description::decode(bytes).map_err(installation::wire)?;
        if let Some(expected) = runtime {
            if !expected.observe
                || !desc.declares(stagemaster_device_info::capability::RUNTIME_APPLICATION)
            {
                return Err(Problem::new(C::Runtime));
            }
            let config = self
                .credentials
                .as_ref()
                .filter(|c| c.device() == desc.device)
                .ok_or_else(|| Problem::new(C::Runtime))?;
            self.runtime =
                Some(application::prepare_runtime(peripheral, desc, config, expected).await?);
        } else if desc.authentication == stagemaster_device_session::AUTHENTICATION {
            if let Some(config) = self
                .credentials
                .as_ref()
                .filter(|c| c.device() == desc.device)
                && desc.declares(stagemaster_device_info::capability::INSTALLATION)
            {
                self.application = Some(application::prepare(peripheral, desc, config).await?);
            }
        } else if self.bonded_experiment {
            self.installation = installation::Channel::prepare(peripheral, bytes).await?;
        }
        Ok(())
    }
}
