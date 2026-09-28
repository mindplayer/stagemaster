//! Isolated radio acceptance only. Echoes caller data, never authorizes installation.
mod protocol;
use super::SecureProbeService;
use embassy_time::{Duration, Instant, with_timeout};
use stagemaster_device_info::Description;
use stagemaster_device_link::secure::{Receiver, Sender};
use stagemaster_device_session::{Context, Error, SecretKey};
use trouble_host::prelude::*;

#[allow(clippy::unnecessary_wraps)]
fn entropy(bytes: &mut [u8]) -> Result<(), Error> {
    esp_hal::rng::Rng::new().read(bytes);
    Ok(())
}
fn now() -> u64 {
    Instant::now().as_millis()
}
pub(super) fn key(identity: &crate::identity::Identity) -> SecretKey {
    let key = SecretKey::generate(entropy).unwrap();
    let desc = Description::decode(&identity.describe(1, false)).unwrap();
    esp_println::println!(
        "SECURE-PROBE-TRUST {{\"device\":{:?},\"boot\":{:?},\"public_key\":{:?}}}",
        desc.device,
        desc.boot,
        key.public()
    );
    key
}

pub(super) struct Probe {
    context: Context,
    protocol: protocol::Protocol,
    rx: Receiver,
    tx: Sender,
    started: bool,
    sending: bool,
    initial_mtu: u16,
}
impl Probe {
    pub fn new(desc: Description, mtu: u16) -> Self {
        let budget = usize::from(mtu.saturating_sub(3).min(244));
        Self {
            context: Context {
                device: desc.device,
                boot: desc.boot,
                connection: desc.session,
            },
            protocol: protocol::Protocol::new(),
            rx: Receiver::new(budget, now()).unwrap(),
            tx: Sender::new(budget, now()).unwrap(),
            started: false,
            sending: false,
            initial_mtu: mtu,
        }
    }
    pub const fn started(&self) -> bool {
        self.started
    }
    pub const fn sending(&self) -> bool {
        self.sending
    }
    pub fn receive(&mut self, bytes: &[u8], key: &SecretKey, mtu: u16) -> bool {
        let result = (|| {
            if self.sending {
                return Err("响应尚未发完");
            }
            if !self.started {
                // CoreBluetooth can finish MTU exchange after connection acceptance.
                // No bytes are pending before this first write; freeze the budget now.
                let budget = usize::from(mtu.saturating_sub(3).min(244));
                esp_println::println!(
                    "安全分片预算：初始MTU={} 当前MTU={} 首片={} 预算={}",
                    self.initial_mtu,
                    mtu,
                    bytes.len(),
                    budget
                );
                self.rx = Receiver::new(budget, now()).map_err(|_| "接收预算无效")?;
                self.tx = Sender::new(budget, now()).map_err(|_| "发送预算无效")?;
                self.protocol
                    .start(self.context, key, now())
                    .map_err(|_| "开始握手失败")?;
                self.started = true;
            }
            let Some(record) = self.rx.push(bytes, now()).map_err(|error| {
                esp_println::println!("安全分片拒绝：{}", error);
                "接收分片失败"
            })?
            else {
                return Ok(());
            };
            let mut output = [0; stagemaster_device_session::CIPHERTEXT_BYTES];
            let length = self
                .protocol
                .receive(record.bytes(), &mut output, now())
                .map_err(|_| "加密协议拒绝")?;
            self.tx
                .queue(&output[..length], now())
                .map_err(|_| "发送队列拒绝")?;
            self.sending = true;
            Ok(())
        })();
        if let Err(reason) = result {
            esp_println::println!("安全无线实验失败：{}", reason);
        }
        result.is_ok()
    }
    pub async fn tick<P: PacketPool>(
        &mut self,
        service: &SecureProbeService,
        conn: &GattConnection<'_, '_, P>,
    ) -> bool {
        if self.protocol.poll(now()).is_err()
            || self.rx.poll(now()).is_err()
            || self.tx.poll(now()).is_err()
        {
            esp_println::println!(
                "安全无线实验期限结束；heap={} peak={}",
                esp_alloc::HEAP.used(),
                esp_alloc::HEAP.stats().max_usage
            );
            return false;
        }
        let packet = match self.tx.fragment(now()) {
            Ok(Some(packet)) => packet,
            Ok(None) => return true,
            Err(_) => return false,
        };
        if !service.response.should_notify(conn)
            || !matches!(
                with_timeout(
                    Duration::from_millis(250),
                    service.response.notify_raw(conn, packet.bytes(), false)
                )
                .await,
                Ok(Ok(()))
            )
        {
            return false;
        }
        match self.tx.sent(now()) {
            Ok(done) => {
                self.sending = !done;
                true
            }
            Err(_) => false,
        }
    }
}
