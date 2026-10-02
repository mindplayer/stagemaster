use crate::{Observer, Player, Status, render};
use alloc::string::String;

impl Player {
    pub(super) fn advance_with(
        &mut self,
        now_ms: u64,
        observer: &mut impl Observer,
    ) -> Result<(), String> {
        let delta = now_ms.checked_sub(self.last_ms).ok_or("播放时钟不能倒退")?;
        self.last_ms = now_ms;
        if self.status != Status::Running {
            return Ok(());
        }
        self.elapsed_ms = self.elapsed_ms.saturating_add(delta);
        loop {
            let Some(index) = self.index else {
                return Err("播放状态缺少活动步骤".into());
            };
            let step = &self.plan.steps[index];
            if !self.activated && self.elapsed_ms >= step.delay_ms {
                observer.activated(index, self.reassert, &mut self.from);
                self.activated = true;
            }
            let Some(duration) = step.duration() else {
                break;
            };
            if self.elapsed_ms < duration {
                break;
            }
            let remaining = self.elapsed_ms - duration;
            self.elapsed_ms = duration;
            self.render();
            self.elapsed_ms = remaining;
            if index + 1 == self.plan.steps.len() {
                if !self.plan.repeat {
                    self.status = Status::Finished;
                    self.elapsed_ms = duration;
                    return Ok(());
                }
                // The end target is known. Preserve bounded catch-up while observers fold
                // the ownership/claim effects of complete cycles that were not sampled.
                if let Some(cycle) = self.plan.cycle_ms {
                    if self.elapsed_ms >= cycle {
                        observer.cycles_skipped();
                    }
                    self.elapsed_ms %= cycle;
                }
                self.index = Some(0);
            } else {
                self.index = Some(index + 1);
            }
            self.activated = false;
            self.reassert = false;
            self.from.copy_from_slice(&self.values);
        }
        self.render();
        Ok(())
    }

    fn render(&mut self) {
        let index = self.index.expect("active step");
        render::step(
            &self.plan,
            index,
            self.elapsed_ms,
            &self.from,
            &mut self.values,
        );
    }
}
