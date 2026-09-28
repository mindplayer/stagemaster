//! Native bonded GATT adapter. The device checks authority; metadata alone never
//! enables its writes. See ADR-045 for `CoreBluetooth`'s security evidence boundary.
use super::{C, Characteristic, Peripheral, Problem, error, notifications::Receiver};
use btleplug::api::{CharPropFlags, Peripheral as _, WriteType};
use stagemaster_device_info::{Description, capability};
use stagemaster_device_link::management::{AUTHENTICATED_LESC, MESSAGE_BYTES, Receipt, Serial};
use std::time::Duration;
use uuid::Uuid;

pub(super) const SERVICE: Uuid = Uuid::from_u128(0xf889ed80_0100_4e83_968e_799ab99558fa);
const RECEIPT: Uuid = Uuid::from_u128(0xf889ed81_0100_4e83_968e_799ab99558fa);
const REQUEST: Uuid = Uuid::from_u128(0xf889ed82_0100_4e83_968e_799ab99558fa);
const RESPONSE: Uuid = Uuid::from_u128(0xf889ed83_0100_4e83_968e_799ab99558fa);

pub(super) struct Channel {
    receipt: Receipt,
    request: Characteristic,
    incoming: Receiver,
    tx: Serial,
    usable: bool,
}

pub(super) fn wire(error: impl std::fmt::Display) -> Problem {
    Problem::new(C::Installation).detail(error.to_string())
}

impl Channel {
    pub async fn prepare(peripheral: &Peripheral, bytes: &[u8]) -> Result<Option<Self>, Problem> {
        let description = Description::decode(bytes).map_err(wire)?;
        if !description.declares(capability::INSTALLATION) {
            return Ok(None);
        }
        if description.authentication != AUTHENTICATED_LESC
            || description.limits.transfer_version != 1
            || description.limits.message_bytes != MESSAGE_BYTES
        {
            return Err(Problem::new(C::Installation));
        }
        let characteristics = peripheral.characteristics();
        let find = |uuid, property| {
            characteristics
                .iter()
                .find(|c| {
                    c.service_uuid == SERVICE && c.uuid == uuid && c.properties.contains(property)
                })
                .cloned()
                .ok_or_else(|| {
                    Problem::new(C::Installation).detail(format!("缺少安装特征或必要属性：{uuid}"))
                })
        };
        let receipt_characteristic = find(RECEIPT, CharPropFlags::READ)?;
        let request = find(REQUEST, CharPropFlags::WRITE)?;
        let response = find(RESPONSE, CharPropFlags::NOTIFY)?;
        // This protected read causes native BLE security restoration. Fresh binding
        // deliberately disconnects before granting; the user must reconnect afterward.
        let receipt = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let bytes = peripheral
                    .read(&receipt_characteristic)
                    .await
                    .map_err(error)?;
                if bytes == [0; 80] {
                    tokio::time::sleep(Duration::from_millis(30)).await;
                    continue;
                }
                break Receipt::decode(&bytes).map_err(wire);
            }
        })
        .await
        .map_err(|_| Problem::new(C::Timeout))??;
        receipt
            .correlate(description.device, description.boot, description.session)
            .map_err(wire)?;
        if receipt.fragment_bytes > peripheral.mtu().saturating_sub(7) {
            return Err(Problem::new(C::Installation));
        }
        let stream = peripheral.notifications().await.map_err(error)?;
        let incoming = Receiver::spawn(stream, RESPONSE, receipt.fragment_bytes);
        peripheral.subscribe(&response).await.map_err(error)?;
        Ok(Some(Self {
            receipt,
            request,
            incoming,
            tx: Serial::default(),
            usable: true,
        }))
    }

    pub fn peer(&self) -> Option<crate::InstallationPeer> {
        (self.usable && self.incoming.healthy()).then_some(crate::InstallationPeer {
            device: self.receipt.device,
            boot: self.receipt.boot,
            session: self.receipt.session,
            authentication: AUTHENTICATED_LESC,
            fragment_bytes: self.receipt.fragment_bytes,
            message_bytes: MESSAGE_BYTES,
        })
    }
    pub fn correlate(&self, bytes: &[u8]) -> Result<(), Problem> {
        let description = Description::decode(bytes).map_err(wire)?;
        self.receipt
            .correlate(description.device, description.boot, description.session)
            .map_err(wire)
    }
    pub async fn write(&mut self, peripheral: &Peripheral, bytes: &[u8]) -> Result<(), Problem> {
        if self.peer().is_none() || bytes.len() > usize::from(self.receipt.fragment_bytes) {
            self.usable = false;
            return Err(Problem::new(C::Installation));
        }
        self.usable = false; // Cancelled or uncertain writes cannot reuse the sequence.
        let packet = self.tx.encode(bytes).map_err(wire)?;
        peripheral
            .write(&self.request, packet.bytes(), WriteType::WithResponse)
            .await
            .map_err(error)?;
        self.tx.advance().map_err(wire)?;
        self.usable = true;
        Ok(())
    }
    pub fn receive(&mut self) -> Result<Option<Vec<u8>>, Problem> {
        if self.peer().is_none() {
            return Err(Problem::new(C::Installation));
        }
        self.incoming.receive()
    }
}
