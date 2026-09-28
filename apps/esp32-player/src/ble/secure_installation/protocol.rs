use super::{Configuration, Context, Epoch, entropy, now};
use crate::installation::{COMPLETIONS, LIVE_EPOCH, REQUESTS};
use core::sync::atomic::Ordering;
use stagemaster_device_auth::application::Session;
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Channel, HANDSHAKE_BYTES, HANDSHAKE_MS, Handshake,
};
use stagemaster_install_worker::{Command, secure::Gateway};
type Result<T> = core::result::Result<T, &'static str>;

#[allow(clippy::large_enum_variant)] // One bounded connection, no per-record allocation.
enum Phase {
    Idle,
    Handshake(Handshake),
    Confirm(Channel),
    Active(Gateway),
    Closed,
}
pub(super) struct Protocol {
    phase: Phase,
    context: Option<Context>,
    epoch: Option<Epoch>,
    until: u64,
}
impl Protocol {
    pub fn new() -> Self {
        LIVE_EPOCH.store(0, Ordering::Release);
        Self {
            phase: Phase::Idle,
            context: None,
            epoch: None,
            until: 0,
        }
    }
    pub fn start(&mut self, context: Context, epoch: Epoch, config: &Configuration) -> Result<()> {
        if !matches!(self.phase, Phase::Idle) {
            return Err("握手顺序错误");
        }
        self.context = Some(context);
        self.epoch = Some(epoch);
        self.until = now().checked_add(HANDSHAKE_MS).ok_or("握手计时异常")?;
        self.phase = Phase::Handshake(
            Handshake::respond(context, config.key(), entropy, now())
                .map_err(|_| "握手初始化失败")?,
        );
        Ok(())
    }
    pub fn close(&mut self) {
        LIVE_EPOCH.store(0, Ordering::Release);
        self.phase = Phase::Closed;
    }
    pub fn poll(&mut self) -> Result<()> {
        // Always drain obsolete completions, including before authentication.
        while let Ok(completion) = COMPLETIONS.try_receive() {
            if let Phase::Active(gateway) = &mut self.phase {
                gateway
                    .complete(completion, now())
                    .map_err(|_| "安装工作回执失效")?;
            }
        }
        match &mut self.phase {
            Phase::Idle => Ok(()),
            Phase::Handshake(_) if now() < self.until => Ok(()),
            Phase::Handshake(_) => Err("握手超时"),
            Phase::Confirm(channel) => channel.poll(now()).map_err(|_| "安全确认失效"),
            Phase::Active(gateway) => {
                let epoch = gateway.live_epoch(now());
                LIVE_EPOCH.store(epoch.map_or(0, Epoch::get), Ordering::Release);
                epoch.map(|_| ()).ok_or("安装权限失效")
            }
            Phase::Closed => Err("连接权限已关闭"),
        }
    }
    pub fn receive(
        &mut self,
        bytes: &[u8],
        config: &Configuration,
        out: &mut [u8; CIPHERTEXT_BYTES],
    ) -> Result<Option<usize>> {
        self.poll()?;
        let phase = core::mem::replace(&mut self.phase, Phase::Closed);
        match phase {
            Phase::Handshake(mut handshake) => {
                handshake.read(bytes, now()).map_err(|_| "控制端握手拒绝")?;
                let mut reply = [0; HANDSHAKE_BYTES];
                let n = handshake
                    .write(&mut reply, now())
                    .map_err(|_| "握手回复失败")?;
                out[..n].copy_from_slice(&reply[..n]);
                self.phase = Phase::Confirm(handshake.finish(now()).map_err(|_| "握手未完成")?);
                Ok(Some(n))
            }
            Phase::Confirm(mut channel) => {
                channel
                    .confirm(bytes, now())
                    .map_err(|_| "控制端确认拒绝")?;
                let n = channel
                    .confirmation(out, now())
                    .map_err(|_| "设备确认失败")?;
                let access = Session::admit(
                    channel,
                    config.permit().map_err(|_| "开发许可无效")?,
                    self.context.ok_or("无连接上下文")?,
                    now(),
                )
                .map_err(|_| "控制端没有安装权限")?;
                let (mut gateway, command) =
                    Gateway::open(access, self.epoch.ok_or("无工作代次")?, now())
                        .map_err(|_| "安装通道拒绝")?;
                enqueue(&mut gateway, command)?;
                self.phase = Phase::Active(gateway);
                esp_println::println!("加密安装准入成功；等待存储工作器确认");
                Ok(Some(n))
            }
            Phase::Active(mut gateway) => {
                if let Some(command) = gateway
                    .receive(bytes, now())
                    .map_err(|_| "加密安装请求拒绝")?
                {
                    enqueue(&mut gateway, command)?;
                }
                self.phase = Phase::Active(gateway);
                Ok(None)
            }
            _ => Err("安全消息顺序错误"),
        }
    }
    pub fn outbound(&mut self, out: &mut [u8; CIPHERTEXT_BYTES]) -> Result<Option<usize>> {
        if let Phase::Active(gateway) = &mut self.phase
            && let Some(bytes) = gateway.outbound(now()).map_err(|_| "安装回复失效")?
        {
            out[..bytes.len()].copy_from_slice(bytes);
            return Ok(Some(bytes.len()));
        }
        Ok(None)
    }
    pub fn sent(&mut self) -> Result<()> {
        let Phase::Active(gateway) = &mut self.phase else {
            return Err("业务会话不存在");
        };
        gateway.sent(now()).map_err(|_| "安装交付确认失败")
    }
}
fn enqueue(gateway: &mut Gateway, command: Command) -> Result<()> {
    let epoch = gateway.live_epoch(now()).ok_or("排队前权限已失效")?;
    LIVE_EPOCH.store(epoch.get(), Ordering::Release);
    REQUESTS.try_send(command).map_err(|_| "安装队列已满")
}
impl Drop for Protocol {
    fn drop(&mut self) {
        self.close();
    }
}
