//! Physical GATT record adapter. Permissions and installation are owned by Gateway.
mod protocol;
use super::SecureInstallationService;
use embassy_time::{Duration, Instant, with_timeout};
use stagemaster_device_auth::application::{Configuration, Role};
use stagemaster_device_info::Description;
use stagemaster_device_link::secure::{Receiver, Sender};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Context, Error};
use stagemaster_install_worker::Epoch;
use trouble_host::prelude::*;

#[allow(clippy::unnecessary_wraps)]
fn entropy(out: &mut [u8]) -> Result<(), Error> {
    esp_hal::rng::Rng::new().read(out);
    Ok(())
}
fn now() -> u64 {
    Instant::now().as_millis()
}
pub(super) fn configuration(identity: &crate::identity::Identity) -> Configuration {
    let bytes = include_bytes!(env!("STAGEMASTER_DEVICE_CONFIGURATION"));
    let config = Configuration::import(bytes, Role::Device).expect("专用开发配置无效");
    let desc = Description::decode(&identity.describe(1, false)).unwrap();
    assert_eq!(config.device(), desc.device, "配置不属于当前实板");
    config
}
pub(super) struct Channel {
    context: Context,
    epoch: Epoch,
    allowed: bool,
    protocol: protocol::Protocol,
    rx: Receiver,
    tx: Sender,
    started: bool,
    sending: bool,
    business: bool,
}
impl Channel {
    pub fn new(desc: Description, epoch: Epoch) -> Self {
        Self {
            context: Context {
                device: desc.device,
                boot: desc.boot,
                connection: desc.session,
            },
            epoch,
            allowed: desc.declares(stagemaster_device_info::capability::INSTALLATION),
            protocol: protocol::Protocol::new(),
            rx: Receiver::new(20, now()).unwrap(),
            tx: Sender::new(20, now()).unwrap(),
            started: false,
            sending: false,
            business: false,
        }
    }
    pub const fn started(&self) -> bool {
        self.started
    }
    pub const fn sending(&self) -> bool {
        self.sending
    }
    pub fn receive(&mut self, bytes: &[u8], config: &Configuration, mtu: u16) -> bool {
        let result = (|| {
            if !self.allowed {
                return Err("安装工作器未就绪");
            }
            if !self.started {
                let budget = usize::from(mtu.saturating_sub(3).min(244));
                self.rx = Receiver::new(budget, now()).map_err(|_| "接收预算无效")?;
                self.tx = Sender::new(budget, now()).map_err(|_| "发送预算无效")?;
                self.protocol.start(self.context, self.epoch, config)?;
                self.started = true;
            }
            let Some(record) = self.rx.push(bytes, now()).map_err(|_| "安全分片拒绝")? else {
                return Ok(());
            };
            let mut out = [0; CIPHERTEXT_BYTES];
            if let Some(n) = self.protocol.receive(record.bytes(), config, &mut out)? {
                if self.sending {
                    return Err("握手响应尚未发完");
                }
                self.tx
                    .queue(&out[..n], now())
                    .map_err(|_| "握手发送拒绝")?;
                self.sending = true;
                self.business = false;
            }
            Ok(())
        })();
        self.checked(result)
    }
    pub async fn tick<P: PacketPool>(
        &mut self,
        service: &SecureInstallationService,
        conn: &GattConnection<'_, '_, P>,
    ) -> bool {
        let result = self.tick_inner(service, conn).await;
        self.checked(result)
    }
    async fn tick_inner<P: PacketPool>(
        &mut self,
        service: &SecureInstallationService,
        conn: &GattConnection<'_, '_, P>,
    ) -> Result<(), &'static str> {
        self.protocol.poll()?;
        self.rx.poll(now()).map_err(|_| "接收记录超时")?;
        self.tx.poll(now()).map_err(|_| "发送记录超时")?;
        if !self.sending {
            let mut out = [0; CIPHERTEXT_BYTES];
            if let Some(n) = self.protocol.outbound(&mut out)? {
                self.tx
                    .queue(&out[..n], now())
                    .map_err(|_| "安装发送拒绝")?;
                self.sending = true;
                self.business = true;
            }
        }
        let Some(packet) = self.tx.fragment(now()).map_err(|_| "发送记录失效")? else {
            return Ok(());
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
            return Err("安装通知发送失败");
        }
        if self.tx.sent(now()).map_err(|_| "记录交付失败")? {
            self.sending = false;
            if self.business {
                self.protocol.sent()?;
            }
        }
        self.protocol.poll()
    }
    fn checked(&mut self, result: Result<(), &'static str>) -> bool {
        if let Err(reason) = result {
            self.protocol.close();
            self.rx.close();
            self.tx.close();
            esp_println::println!("加密安装连接已关闭：{}", reason);
        }
        result.is_ok()
    }
}
