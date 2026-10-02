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
pub(super) type Channel = stagemaster_device_channel::Channel<GattRecords>;
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
    let chars = peripheral.characteristics();
    let find = |uuid, property| {
        chars
            .iter()
            .find(|c| {
                c.service_uuid == SERVICE && c.uuid == uuid && c.properties.contains(property)
            })
            .cloned()
            .ok_or_else(|| Problem::new(C::Installation))
    };
    let request = find(REQUEST, CharPropFlags::WRITE_WITHOUT_RESPONSE)?;
    let response = find(RESPONSE, CharPropFlags::NOTIFY)?;
    let origin = Instant::now();
    let budget = usize::from(peripheral.mtu().saturating_sub(3).min(244));
    let sender = Sender::new(budget, 0).map_err(super::installation::wire)?;
    let incoming = Incoming::spawn(
        peripheral.notifications().await.map_err(super::error)?,
        origin,
        budget,
    );
    peripheral
        .subscribe(&response)
        .await
        .map_err(super::error)?;
    Channel::prepare(
        GattRecords {
            peripheral: peripheral.clone(),
            request,
            sender,
            incoming: Some(incoming),
            origin,
            usable: true,
        },
        desc,
        config,
    )
    .await
    .map_err(super::installation::wire)
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
