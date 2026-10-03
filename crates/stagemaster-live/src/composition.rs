use crate::{Frame, Session};
use stagemaster_playback::Command;

#[derive(Clone, Copy)]
pub(super) struct Claim {
    at_ms: u64,
    id: [u8; 16],
    source: usize,
    attribute: usize,
}
impl Session {
    /// Advance every source against the previous complete composition, then publish one frame.
    /// # Errors
    /// Reject backwards time; internal failures latch a fault and invalidate the frame.
    pub fn tick(&mut self, now_ms: u64) -> Result<(), String> {
        self.ready(now_ms)?;
        let result = self.advance(now_ms).and_then(|()| self.compose(now_ms));
        self.finish(result)
    }
    fn advance(&mut self, now_ms: u64) -> Result<(), String> {
        for group in &mut self.media {
            group.expire(now_ms)?;
        }
        for entry in &mut self.sources {
            if entry.media_group.is_some() {
                continue;
            }
            if let Some(player) = &mut entry.player {
                player.apply(Command::Advance, now_ms, &self.mixer, entry.handle)?;
            }
        }
        Ok(())
    }
    pub(super) fn compose(&mut self, now_ms: u64) -> Result<(), String> {
        self.claims.clear();
        for (source, entry) in self.sources.iter_mut().enumerate() {
            if let Some(player) = &entry.player {
                player
                    .copy_contribution(&mut entry.values, &mut entry.times)
                    .map_err(|e| e.to_string())?;
            }
            if entry.media_group.is_some() {
                for (value, time) in entry.values.iter().zip(&mut entry.times) {
                    if let Some(at) = entry.reassert_at_ms {
                        *time = value.map(|_| at);
                    } else if time.is_some() {
                        *time = Some(entry.sampled_at_ms);
                    }
                }
            }
            for (attribute, time) in entry.times.iter().enumerate() {
                if let Some(at_ms) = time {
                    if *at_ms > now_ms || entry.values[attribute].is_none() {
                        return Err("来源接管事件与贡献不一致".into());
                    }
                    self.claims.push(Claim {
                        at_ms: *at_ms,
                        id: entry.id,
                        source,
                        attribute,
                    });
                }
            }
            // Install final values/withdrawals privately. Reassertions below give every
            // new acquisition its true order; no intermediate frame escapes this method.
            entry.assertions.fill(false);
            entry.publish(&mut self.mixer)?;
        }
        self.claims
            .sort_unstable_by_key(|e| (e.at_ms, e.id, e.attribute));
        let mut cursor = 0;
        while cursor < self.claims.len() {
            let first = self.claims[cursor];
            let entry = &mut self.sources[first.source];
            entry.assertions.fill(false);
            while let Some(event) = self.claims.get(cursor) {
                if event.source != first.source || event.at_ms != first.at_ms {
                    break;
                }
                entry.assertions[event.attribute] = true;
                cursor += 1;
            }
            entry.publish(&mut self.mixer)?;
        }
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or("合成帧序号已耗尽，须重新准备")?;
        let mut slots = [0; 512];
        let universe = self.output.render(&self.mixer, &mut slots)?;
        for entry in &mut self.sources {
            entry.times.fill(None);
            entry.reassert_at_ms = None;
            if let Some(player) = &mut entry.player {
                player.acknowledge_contribution();
            }
        }
        self.sequence = sequence;
        self.now_ms = now_ms;
        self.frame = Some(Frame {
            sequence,
            sampled_ms: now_ms,
            universe,
            slots,
        });
        Ok(())
    }
}
