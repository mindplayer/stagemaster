//! Actual firmware handshake. Contains no board I/O; tested unchanged on the host.
use super::gate::{Gate, Mode, Port, Result};
use stagemaster_device_auth::application::{Configuration, DevelopmentPermit, Session};
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Channel, Context, Error, HANDSHAKE_BYTES, HANDSHAKE_MS, Handshake,
};
use stagemaster_install_worker::Epoch;

#[allow(clippy::large_enum_variant)] // One bounded connection, no per-record allocation.
enum Phase {
    Handshake(Handshake),
    Confirm(Channel),
    Active(Gate),
    Closed,
}
#[derive(Clone, Copy)]
pub(super) struct Settings {
    pub context: Context,
    pub epoch: Epoch,
    pub mode: Mode,
}
pub(super) struct Protocol<P: Port> {
    phase: Phase,
    context: Context,
    epoch: Epoch,
    mode: Mode,
    until: u64,
    last_ms: u64,
    permit: DevelopmentPermit,
    port: P,
}
impl<P: Port> Protocol<P> {
    pub fn new(
        settings: Settings,
        config: &Configuration,
        entropy: fn(&mut [u8]) -> core::result::Result<(), Error>,
        now: u64,
        mut port: P,
    ) -> Result<Self> {
        port.publish(None);
        let Settings {
            context,
            epoch,
            mode,
        } = settings;
        let permit = match mode {
            Mode::Installation => config.permit(),
            Mode::Runtime => config.runtime_permit(),
        }
        .map_err(|_| "当前开发配置没有此入口权限")?;
        let until = now.checked_add(HANDSHAKE_MS).ok_or("握手计时异常")?;
        let phase = Phase::Handshake(
            Handshake::respond(context, config.key(), entropy, now)
                .map_err(|_| "握手初始化失败")?,
        );
        Ok(Self {
            phase,
            context,
            epoch,
            mode,
            until,
            last_ms: now,
            permit,
            port,
        })
    }
    pub fn close(&mut self) {
        self.port.publish(None);
        self.phase = Phase::Closed;
    }
    pub fn poll(&mut self, now: u64) -> Result<()> {
        let result = (|| {
            if now < self.last_ms {
                return Err("应用计时回退");
            }
            self.last_ms = now;
            while let Some(completion) = self.port.completion() {
                if let Phase::Active(gate) = &mut self.phase {
                    gate.complete(completion, now)?;
                }
            }
            match &mut self.phase {
                Phase::Handshake(_) if now < self.until => Ok(()),
                Phase::Handshake(_) => Err("握手超时"),
                Phase::Confirm(channel) => channel.poll(now).map_err(|_| "安全确认失效"),
                Phase::Active(gate) => gate.live(now).map(|_| ()).ok_or("应用权限失效"),
                Phase::Closed => Err("连接权限已关闭"),
            }
        })();
        self.finish(result, now)
    }
    pub fn receive(
        &mut self,
        bytes: &[u8],
        out: &mut [u8; CIPHERTEXT_BYTES],
        now: u64,
    ) -> Result<Option<usize>> {
        self.poll(now)?;
        let result = self.receive_inner(bytes, out, now);
        self.finish(result, now)
    }
    fn receive_inner(
        &mut self,
        bytes: &[u8],
        out: &mut [u8; CIPHERTEXT_BYTES],
        now: u64,
    ) -> Result<Option<usize>> {
        let phase = core::mem::replace(&mut self.phase, Phase::Closed);
        let command;
        let reply;
        match phase {
            Phase::Handshake(mut handshake) => {
                handshake.read(bytes, now).map_err(|_| "控制端握手拒绝")?;
                let mut response = [0; HANDSHAKE_BYTES];
                let n = handshake
                    .write(&mut response, now)
                    .map_err(|_| "握手回复失败")?;
                out[..n].copy_from_slice(&response[..n]);
                self.phase = Phase::Confirm(handshake.finish(now).map_err(|_| "握手未完成")?);
                return Ok(Some(n));
            }
            Phase::Confirm(mut channel) => {
                channel.confirm(bytes, now).map_err(|_| "控制端确认拒绝")?;
                let n = channel.confirmation(out, now).map_err(|_| "设备确认失败")?;
                let session = Session::admit(channel, self.permit, self.context, now)
                    .map_err(|_| "控制端准入拒绝")?;
                let (gate, queued) = Gate::open(session, self.mode, self.epoch, now)?;
                self.phase = Phase::Active(gate);
                command = queued;
                reply = Some(n);
            }
            Phase::Active(mut gate) => {
                command = gate.receive(bytes, now)?;
                self.phase = Phase::Active(gate);
                reply = None;
            }
            Phase::Closed => return Err("安全消息顺序错误"),
        }
        // Publish fresh authority before making the command visible on the other core.
        self.publish(now)?;
        if let Some(command) = command {
            self.port.enqueue(command)?;
        }
        Ok(reply)
    }
    pub fn outbound(
        &mut self,
        out: &mut [u8; CIPHERTEXT_BYTES],
        now: u64,
    ) -> Result<Option<usize>> {
        self.poll(now)?;
        let result = (|| {
            if let Phase::Active(gate) = &mut self.phase
                && let Some(bytes) = gate.outbound(now)?
            {
                out[..bytes.len()].copy_from_slice(bytes);
                return Ok(Some(bytes.len()));
            }
            Ok(None)
        })();
        self.finish(result, now)
    }
    pub fn sent(&mut self, now: u64) -> Result<()> {
        self.poll(now)?;
        let result = match &mut self.phase {
            Phase::Active(gate) => gate.sent(now),
            _ => Err("业务会话不存在"),
        };
        self.finish(result, now)
    }
    fn publish(&mut self, now: u64) -> Result<()> {
        if let Phase::Active(gate) = &mut self.phase {
            let live = gate.live(now);
            self.port.publish(live);
            live.map(|_| ()).ok_or("应用权限失效")
        } else {
            self.port.publish(None);
            Ok(())
        }
    }
    fn finish<T>(&mut self, result: Result<T>, now: u64) -> Result<T> {
        let result = result.and_then(|value| self.publish(now).map(|()| value));
        if result.is_err() {
            self.close();
        }
        result
    }
}
impl<P: Port> Drop for Protocol<P> {
    fn drop(&mut self) {
        self.close();
    }
}
