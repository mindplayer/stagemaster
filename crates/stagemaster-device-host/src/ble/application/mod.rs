//! GATT carrier only. The shared application channel owns authentication and message semantics.
mod incoming;
use super::{C, Characteristic, Peripheral, Problem};
use btleplug::api::{CharPropFlags, Peripheral as _, WriteType};
use incoming::Incoming;
use stagemaster_device_channel::{Error, RecordIo};
use stagemaster_device_link::secure::Sender;
use tokio::time::Instant;
use uuid::Uuid;

pub(super) const SERVICE: Uuid = Uuid::from_u128(0xf889eda0_0100_4e83_968e_799ab99558fa);
const REQUEST: Uuid = Uuid::from_u128(0xf889eda2_0100_4e83_968e_799ab99558fa);
const RESPONSE: Uuid = Uuid::from_u128(0xf889eda3_0100_4e83_968e_799ab99558fa);
const RUNTIME_SERVICE: Uuid = Uuid::from_u128(0xf889edb0_0100_4e83_968e_799ab99558fa);
const RUNTIME_REQUEST: Uuid = Uuid::from_u128(0xf889edb2_0100_4e83_968e_799ab99558fa);
const RUNTIME_RESPONSE: Uuid = Uuid::from_u128(0xf889edb3_0100_4e83_968e_799ab99558fa);
pub(super) type Channel = stagemaster_device_channel::Channel<GattRecords>;
pub(super) type Runtime = stagemaster_device_channel::runtime::RuntimeClient<GattRecords>;
fn now(origin: Instant) -> u64 {
    u64::try_from(origin.elapsed().as_millis()).unwrap_or(u64::MAX)
}
fn wire(error: impl std::fmt::Display) -> Error {
    Error::Transport(error.to_string())
}

pub(super) async fn prepare(
    peripheral: &Peripheral,
    desc: stagemaster_device_info::Description,
    config: &crate::DevelopmentConfiguration,
) -> Result<Channel, Problem> {
    Channel::prepare(
        records(peripheral, [SERVICE, REQUEST, RESPONSE], C::Installation).await?,
        desc,
        config,
    )
    .await
    .map_err(super::installation::wire)
}
pub(super) async fn prepare_runtime(
    peripheral: &Peripheral,
    desc: stagemaster_device_info::Description,
    config: &crate::DevelopmentConfiguration,
    expected: stagemaster_runtime_protocol::Access,
) -> Result<Runtime, Problem> {
    let channel = Channel::prepare_runtime(
        records(
            peripheral,
            [RUNTIME_SERVICE, RUNTIME_REQUEST, RUNTIME_RESPONSE],
            C::Runtime,
        )
        .await?,
        desc,
        config,
        expected,
    )
    .await
    .map_err(crate::runtime::wire)?;
    Runtime::new(channel).map_err(crate::runtime::wire)
}
async fn records(
    peripheral: &Peripheral,
    endpoints: [Uuid; 3],
    failure: C,
) -> Result<GattRecords, Problem> {
    let [service_id, request_id, response_id] = endpoints;
    let chars = peripheral.characteristics();
    let find = |uuid, property| {
        chars
            .iter()
            .find(|c| {
                c.service_uuid == service_id && c.uuid == uuid && c.properties.contains(property)
            })
            .cloned()
            .ok_or_else(|| Problem::new(failure))
    };
    let request = find(request_id, CharPropFlags::WRITE_WITHOUT_RESPONSE)?;
    let response = find(response_id, CharPropFlags::NOTIFY)?;
    let origin = Instant::now();
    let budget = usize::from(peripheral.mtu().saturating_sub(3).min(244));
    let sender = Sender::new(budget, 0).map_err(|e| Problem::new(failure).detail(e.to_string()))?;
    let incoming = Incoming::spawn(
        peripheral.notifications().await.map_err(super::error)?,
        origin,
        budget,
        [service_id, response_id],
    );
    peripheral
        .subscribe(&response)
        .await
        .map_err(super::error)?;
    Ok(GattRecords {
        peripheral: peripheral.clone(),
        request,
        sender,
        incoming: Some(incoming),
        origin,
        usable: true,
    })
}

pub(super) struct GattRecords {
    peripheral: Peripheral,
    request: Characteristic,
    sender: Sender,
    incoming: Option<Incoming>,
    origin: Instant,
    usable: bool,
}
impl RecordIo for GattRecords {
    async fn send(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if !self.healthy() {
            return Err(Error::Closed);
        }
        self.usable = false;
        self.sender.queue(bytes, now(self.origin)).map_err(wire)?;
        while let Some(packet) = self.sender.fragment(now(self.origin)).map_err(wire)? {
            self.peripheral
                .write(&self.request, packet.bytes(), WriteType::WithoutResponse)
                .await
                .map_err(wire)?;
            self.sender.sent(now(self.origin)).map_err(wire)?;
        }
        self.usable = true;
        if !self.healthy() {
            return Err(Error::Closed);
        }
        Ok(())
    }
    fn try_receive(&mut self) -> Result<Option<Vec<u8>>, Error> {
        if !self.healthy() {
            return Err(Error::Closed);
        }
        self.incoming
            .as_mut()
            .ok_or(Error::Closed)?
            .next()
            .map(|r| r.map(|r| r.bytes().to_vec()))
            .map_err(wire)
    }
    fn healthy(&self) -> bool {
        self.usable && self.incoming.as_ref().is_some_and(Incoming::healthy)
    }
    fn close(&mut self) {
        self.usable = false;
        self.incoming = None;
    }
}
impl Drop for GattRecords {
    fn drop(&mut self) {
        self.close();
    }
}
