//! Platform-independent business routing, shared by firmware and host acceptance tests.
use stagemaster_device_auth::application::Session;
use stagemaster_install_worker::{Epoch, runtime_queue, secure};

pub(super) type Result<T> = core::result::Result<T, &'static str>;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Installation,
    Runtime,
}
#[allow(clippy::large_enum_variant)] // One fixed queue slot; no per-message allocation.
pub(super) enum Command {
    Installation(stagemaster_install_worker::Command),
    Runtime(runtime_queue::Command),
}
#[allow(clippy::large_enum_variant)]
pub(super) enum Completion {
    Installation(stagemaster_install_worker::Completion),
    Runtime(runtime_queue::Completion),
}
#[derive(Clone, Copy)]
pub(super) enum Validity {
    Installation(Epoch),
    Runtime(runtime_queue::Live),
}

/// Owns publication cleanup; implementations atomically publish the complete value.
pub(super) trait Port {
    fn publish(&mut self, value: Option<Validity>);
    fn enqueue(&mut self, command: Command) -> Result<()>;
    fn completion(&mut self) -> Option<Completion>;
}

#[allow(clippy::large_enum_variant)] // Only one application mode on a physical connection.
pub(super) enum Gate {
    Installation(secure::Gateway),
    Runtime(runtime_queue::Gateway),
}
impl Gate {
    pub fn open(
        session: Session,
        mode: Mode,
        epoch: Epoch,
        now: u64,
    ) -> Result<(Self, Option<Command>)> {
        match mode {
            Mode::Installation => {
                let (gate, command) =
                    secure::Gateway::open(session, epoch, now).map_err(|_| "安装准入拒绝")?;
                Ok((
                    Self::Installation(gate),
                    Some(Command::Installation(command)),
                ))
            }
            Mode::Runtime => Ok((
                Self::Runtime(
                    runtime_queue::Gateway::new(session, epoch, now).map_err(|_| "运行准入拒绝")?,
                ),
                None,
            )),
        }
    }
    pub fn live(&mut self, now: u64) -> Option<Validity> {
        match self {
            Self::Installation(g) => g.live_epoch(now).map(Validity::Installation),
            Self::Runtime(g) => g.live(now).map(Validity::Runtime),
        }
    }
    pub fn receive(&mut self, bytes: &[u8], now: u64) -> Result<Option<Command>> {
        match self {
            Self::Installation(g) => g.receive(bytes, now).map(|c| c.map(Command::Installation)),
            Self::Runtime(g) => {
                return g
                    .receive(bytes, now)
                    .map(|c| c.map(Command::Runtime))
                    .map_err(|_| "运行请求拒绝");
            }
        }
        .map_err(|_| "安装请求拒绝")
    }
    pub fn complete(&mut self, completion: Completion, now: u64) -> Result<()> {
        match (self, completion) {
            (Self::Installation(g), Completion::Installation(c)) => {
                g.complete(c, now).map(|_| ()).map_err(|_| "安装回执失效")
            }
            (Self::Runtime(g), Completion::Runtime(c)) => {
                g.complete(c, now).map(|_| ()).map_err(|_| "运行回执失效")
            }
            _ => Err("应用回执入口不匹配"),
        }
    }
    pub fn outbound(&mut self, now: u64) -> Result<Option<&[u8]>> {
        match self {
            Self::Installation(g) => g.outbound(now).map_err(|_| "安装回复失效"),
            Self::Runtime(g) => g.outbound(now).map_err(|_| "运行回复失效"),
        }
    }
    pub fn sent(&mut self, now: u64) -> Result<()> {
        match self {
            Self::Installation(g) => g.sent(now).map_err(|_| "安装交付确认失败"),
            Self::Runtime(g) => g.sent(now).map_err(|_| "运行交付确认失败"),
        }
    }
}
