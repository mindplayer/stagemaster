use stagemaster_playback::Plan;

/// Explicit synthetic command times, not an alternative player or wall clock.
pub struct Schedule {
    pub next: u64,
    pub pause: u64,
    pub resume: u64,
    pub automatic: u64,
    pub reconnect: u64,
    pub stop: u64,
}
impl Schedule {
    pub fn from(plan: &Plan) -> Result<Self, String> {
        let steps = plan.steps();
        if steps.len() != 3
            || plan.repeat()
            || steps[0].wait_ms.is_some()
            || steps[1].wait_ms.is_none()
            || steps[1].fade_ms < 2
            || steps[2].wait_ms.is_some()
        {
            return Err("本验收只接收人工保持→定时渐变推进→人工保持的单次三步列表".into());
        }
        let next = steps[0].delay_ms + steps[0].fade_ms + 500;
        let pause = next + steps[1].delay_ms + steps[1].fade_ms / 2;
        let resume = pause + 500;
        let automatic =
            next + steps[1].delay_ms + steps[1].fade_ms + steps[1].wait_ms.unwrap() + 500;
        let reconnect = automatic + 200;
        let stop = reconnect + steps[2].delay_ms + steps[2].fade_ms + 300;
        if stop > 60_000 {
            return Err("本验收的完整脚本须在 60000 毫秒内".into());
        }
        Ok(Self {
            next,
            pause,
            resume,
            automatic,
            reconnect,
            stop,
        })
    }
    pub fn checkpoints(&self) -> Vec<u64> {
        vec![
            0,
            self.next - 1,
            self.next,
            self.pause,
            self.resume - 1,
            self.resume,
            self.resume + 1,
            self.automatic - 1,
            self.automatic,
            self.automatic + 1,
            self.reconnect,
            self.stop,
        ]
    }
}
