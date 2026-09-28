use crate::{Prepared, Snapshot, Target, Task};

pub(crate) struct Job {
    pub prepared: Prepared,
    pub target: Target,
    pub connected: bool,
    pub cancel_applied: bool,
}
impl Job {
    pub fn resume(&mut self, target: Target) -> Result<(), String> {
        if self.connected && self.target.epoch == target.epoch && self.target.peer == target.peer {
            self.prepared.upload.retry().map_err(|e| e.to_string())?;
        } else {
            self.prepared.upload.disconnect();
            self.connected = false;
            self.prepared
                .upload
                .connect(target.peer.session)
                .map_err(|e| e.to_string())?;
        }
        self.target = target;
        self.connected = true;
        Ok(())
    }
}
#[derive(Default)]
pub(crate) struct State {
    pub view: Snapshot,
    pub job: Option<Job>,
    pub closed: bool,
    pub cursor: Option<(Target, Option<u64>)>,
}
impl State {
    pub fn next_id(&self, target: &Target) -> Result<u64, String> {
        if let Some((old, next)) = &self.cursor
            && old.peer.device == target.peer.device
            && old.peer.session == target.peer.session
        {
            if old.epoch != target.epoch || old.peer != target.peer {
                return Err("设备重复使用了旧业务会话，请重新认证".into());
            }
            return next.ok_or_else(|| "上次通信结果不确定或序号已用尽，请重连设备后再操作".into());
        }
        Ok(1)
    }
    pub fn task(&mut self, id: &str) -> Result<&mut Task, String> {
        if self.closed {
            return Err("安装任务服务已关闭".into());
        }
        self.view
            .task
            .as_mut()
            .filter(|t| t.id == id)
            .ok_or_else(|| "安装任务已变化，请重新读取状态".into())
    }
    pub fn touch(&mut self) {
        self.view.revision += 1;
    }
}
