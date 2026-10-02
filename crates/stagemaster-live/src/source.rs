use stagemaster_engine::live::{Frame, Handle, LiveMixer};
use stagemaster_project::LiveSequencePlayer;

pub(super) struct Entry {
    pub id: [u8; 16],
    pub handle: Handle,
    pub serial: u64,
    pub level: u16,
    pub player: Option<LiveSequencePlayer>,
    pub values: Vec<Option<u16>>,
    pub times: Vec<Option<u64>>,
    pub assertions: Vec<bool>,
}
impl Entry {
    pub fn next_serial(&mut self) -> Result<u64, String> {
        let next = self
            .serial
            .checked_add(1)
            .filter(|v| *v < u64::MAX)
            .ok_or("来源命令序号已耗尽，须重新准备")?;
        self.serial = next;
        Ok(next)
    }
    pub fn publish(&mut self, mixer: &mut LiveMixer) -> Result<(), String> {
        let serial = self.next_serial()?;
        mixer
            .publish(
                self.handle,
                Frame {
                    layout: mixer.layout().id(),
                    serial,
                    values: &self.values,
                    assert: &self.assertions,
                },
            )
            .map_err(|e| e.to_string())
    }
}
