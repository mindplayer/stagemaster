//! Shared physical GATT records; the first application endpoint owns this connection.
mod gate;
mod port;
mod protocol;
use super::Server;
use embassy_time::{Duration, Instant, with_timeout};
pub(super) use gate::Mode;
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
pub(super) fn request_mode(server: &Server<'_>, handle: u16) -> Option<Mode> {
    #[cfg(feature = "runtime-gatt")]
    let runtime = Some(server.secure_runtime.request.handle);
    #[cfg(not(feature = "runtime-gatt"))]
    let runtime = None;
    if handle == server.secure_installation.request.handle {
        Some(Mode::Installation)
    } else if Some(handle) == runtime {
        Some(Mode::Runtime)
    } else {
        None
    }
}
pub(super) fn response<'a>(
    server: &'a Server<'_>,
    mode: Mode,
) -> Option<&'a Characteristic<[u8; 244]>> {
    match mode {
        Mode::Installation => Some(&server.secure_installation.response),
        #[cfg(feature = "runtime-gatt")]
        Mode::Runtime => Some(&server.secure_runtime.response),
        #[cfg(not(feature = "runtime-gatt"))]
        Mode::Runtime => None,
    }
}
pub(super) struct Channel {
    context: Context,
    epoch: Epoch,
    description: Description,
    mode: Option<Mode>,
    protocol: Option<protocol::Protocol<port::Queues>>,
    rx: Receiver,
    tx: Sender,
    sending: bool,
    business: bool,
}
impl Channel {
    pub fn new(desc: Description, epoch: Epoch) -> Self {
        port::clear();
        Self {
            context: Context {
                device: desc.device,
                boot: desc.boot,
                connection: desc.session,
            },
            epoch,
            description: desc,
            mode: None,
            protocol: None,
            rx: Receiver::new(20, now()).unwrap(),
            tx: Sender::new(20, now()).unwrap(),
            sending: false,
            business: false,
        }
    }
    pub const fn started(&self) -> bool {
        self.protocol.is_some()
    }
    pub const fn sending(&self) -> bool {
        self.sending
    }
    pub fn receive(&mut self, mode: Mode, bytes: &[u8], config: &Configuration, mtu: u16) -> bool {
        let result = (|| {
            let capability = match mode {
                Mode::Installation => stagemaster_device_info::capability::INSTALLATION,
                Mode::Runtime => stagemaster_device_info::capability::RUNTIME_APPLICATION,
            };
            if !self.description.declares(capability)
                || self.mode.is_some_and(|active| active != mode)
            {
                return Err("应用入口未就绪或本连接已经选择其他入口");
            }
            if self.protocol.is_none() {
                let budget = usize::from(mtu.saturating_sub(3).min(244));
                self.rx = Receiver::new(budget, now()).map_err(|_| "接收预算无效")?;
                self.tx = Sender::new(budget, now()).map_err(|_| "发送预算无效")?;
                self.protocol = Some(protocol::Protocol::new(
                    protocol::Settings {
                        context: self.context,
                        epoch: self.epoch,
                        mode,
                    },
                    config,
                    entropy,
                    now(),
                    port::Queues(mode),
                )?);
                self.mode = Some(mode);
            }
            let Some(record) = self.rx.push(bytes, now()).map_err(|_| "安全分片拒绝")? else {
                return Ok(());
            };
            let mut out = [0; CIPHERTEXT_BYTES];
            if let Some(n) = self.protocol.as_mut().ok_or("应用握手尚未开始")?.receive(
                record.bytes(),
                &mut out,
                now(),
            )? {
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
        server: &Server<'_>,
        conn: &GattConnection<'_, '_, P>,
    ) -> bool {
        let result = self.tick_inner(server, conn).await;
        self.checked(result)
    }
    async fn tick_inner<P: PacketPool>(
        &mut self,
        server: &Server<'_>,
        conn: &GattConnection<'_, '_, P>,
    ) -> Result<(), &'static str> {
        let Some(protocol) = self.protocol.as_mut() else {
            while crate::installation::COMPLETIONS.try_receive().is_ok() {}
            while crate::installation::runtime_io::COMPLETIONS
                .try_receive()
                .is_ok()
            {}
            return Ok(());
        };
        protocol.poll(now())?;
        self.rx.poll(now()).map_err(|_| "接收记录超时")?;
        self.tx.poll(now()).map_err(|_| "发送记录超时")?;
        if !self.sending {
            let mut out = [0; CIPHERTEXT_BYTES];
            if let Some(n) = protocol.outbound(&mut out, now())? {
                self.tx
                    .queue(&out[..n], now())
                    .map_err(|_| "应用发送拒绝")?;
                self.sending = true;
                self.business = true;
            }
        }
        let Some(packet) = self.tx.fragment(now()).map_err(|_| "发送记录失效")? else {
            return Ok(());
        };
        let response =
            response(server, self.mode.ok_or("应用入口不存在")?).ok_or("此镜像不支持运行入口")?;
        if !response.should_notify(conn)
            || !matches!(
                with_timeout(
                    Duration::from_millis(250),
                    response.notify_raw(conn, packet.bytes(), false)
                )
                .await,
                Ok(Ok(()))
            )
        {
            return Err("应用通知发送失败");
        }
        if self.tx.sent(now()).map_err(|_| "记录交付失败")? {
            self.sending = false;
            if self.business {
                protocol.sent(now())?;
            }
        }
        protocol.poll(now())
    }
    fn checked(&mut self, result: Result<(), &'static str>) -> bool {
        if let Err(reason) = result {
            if let Some(protocol) = self.protocol.as_mut() {
                protocol.close();
            }
            port::clear();
            self.rx.close();
            self.tx.close();
            esp_println::println!("加密应用连接已关闭：{}", reason);
        }
        result.is_ok()
    }
}
