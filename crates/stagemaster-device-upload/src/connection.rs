use crate::{PackageInfo, package::hex};
use stagemaster_device_host::{DeviceLimits, InstallationPeer, Request, Service, Transport};
use stagemaster_transfer::Frame;
use std::future::Future;

/// A trusted platform observation, never constructed from an IPC permission claim.
#[derive(Clone, Debug)]
pub struct Target {
    pub epoch: u32,
    pub peer: InstallationPeer,
    pub name: String,
    pub limits: DeviceLimits,
}
impl Target {
    pub(crate) fn check(&self, info: &PackageInfo, expected: &str) -> Result<(), String> {
        if hex(&self.peer.device) != expected {
            return Err("当前连接不是原定安装设备，请重新选择正确设备".into());
        }
        let l = &self.limits;
        if l.package_version != 1 {
            return Err("设备不支持当前播放包版本".into());
        }
        if info.bytes > l.package_bytes as usize
            || info.bytes > l.slot_bytes as usize
            || info.programs > usize::from(l.programs)
        {
            return Err("播放包超出设备容量，请减少节目后重新生成".into());
        }
        if l.loader_bytes > 0 && info.loader_bytes > l.loader_bytes as usize {
            return Err("节目装载内存超出设备声明的预算".into());
        }
        Ok(())
    }
}
/// An existing application connection. No discovery, pairing or automatic reconnect here.
pub trait Connection: Send + Sync + 'static {
    /// # Errors
    /// Require fresh authenticated installation permission at the supplied connection epoch.
    fn target(&self, epoch: u32) -> Result<Target, String>;
    fn exchange(
        &self,
        epoch: u32,
        frame: Frame,
    ) -> impl Future<Output = Result<Frame, String>> + Send;
}
impl<B: Transport> Connection for Service<B> {
    fn target(&self, epoch: u32) -> Result<Target, String> {
        let state = self.request(Request::Status).map_err(|e| e.to_string())?;
        let peer = self
            .installation_peer(epoch)
            .map_err(|e| e.to_string())?
            .ok_or("当前设备尚未取得节目安装权限")?;
        if state.epoch != epoch {
            return Err("设备连接已变化，请重新读取状态".into());
        }
        let description = state.description.ok_or("设备描述已失效")?;
        Ok(Target {
            epoch,
            peer,
            limits: description.limits,
            name: state.selected.map_or_else(|| "播放设备".into(), |d| d.name),
        })
    }
    async fn exchange(&self, epoch: u32, frame: Frame) -> Result<Frame, String> {
        self.exchange_installation(epoch, frame)
            .await
            .map_err(|e| e.to_string())
    }
}
