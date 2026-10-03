//! Repeat source ranges using caller-owned integer ticks, independent of audio or project files.
use alloc::{string::String, vec::Vec};

pub const MAX_LOOP_REGIONS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopPlays {
    /// Total passes, including the initial pass. Zero is invalid.
    Count(u32),
    UntilExit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopRegion {
    pub start: u64,
    pub end: u64,
    pub plays: LoopPlays,
}

pub struct LoopSchedule {
    duration: u64,
    regions: Vec<LoopRegion>,
}

impl LoopSchedule {
    /// All positions use one caller-defined timebase (for example audio sample frames).
    /// # Errors
    /// Reject empty duration, excess regions, zero counts, overlaps, ordering and invalid ranges.
    pub fn new(duration: u64, regions: Vec<LoopRegion>) -> Result<Self, String> {
        if duration == 0 || regions.len() > MAX_LOOP_REGIONS {
            return Err("演出长度须大于零，循环区段不能超过 128 段".into());
        }
        let mut previous_end = 0;
        for region in &regions {
            if region.start < previous_end
                || region.start >= region.end
                || region.end > duration
                || region.plays == LoopPlays::Count(0)
            {
                return Err("循环区段须有序且不重叠，范围和总播放次数须有效".into());
            }
            previous_end = region.end;
        }
        Ok(Self { duration, regions })
    }

    #[must_use]
    pub fn regions(&self) -> &[LoopRegion] {
        &self.regions
    }

    #[must_use]
    pub const fn duration(&self) -> u64 {
        self.duration
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopPosition {
    /// Original source position, never an expanded or wall-clock timeline.
    pub tick: u64,
    /// Cumulative backward distance since the last explicit seek, in the same timebase.
    pub repeated_ticks: u64,
    /// Index within the immutable schedule; absent between regions or at the end.
    pub region: Option<usize>,
    pub pass: Option<u64>,
    pub exit_requested: bool,
    pub ended: bool,
}

#[derive(Clone, Copy)]
struct Cursor {
    tick: u64,
    repeated_ticks: u64,
    next_region: usize,
    pass: u64,
    exit_requested: bool,
}

/// Owns a validated immutable schedule. Pausing means no calls to `advance`.
pub struct LoopPlayback {
    schedule: LoopSchedule,
    cursor: Cursor,
}

impl LoopPlayback {
    /// # Errors
    /// Reject a starting position beyond the source duration.
    pub fn new(schedule: LoopSchedule, tick: u64) -> Result<Self, String> {
        let mut player = Self {
            schedule,
            cursor: Cursor {
                tick: 0,
                repeated_ticks: 0,
                next_region: 0,
                pass: 1,
                exit_requested: false,
            },
        };
        player.seek(tick)?;
        Ok(player)
    }

    #[must_use]
    pub fn position(&self) -> LoopPosition {
        let region = self
            .schedule
            .regions
            .get(self.cursor.next_region)
            .filter(|r| self.cursor.tick >= r.start && self.cursor.tick < r.end)
            .map(|_| self.cursor.next_region);
        LoopPosition {
            tick: self.cursor.tick,
            repeated_ticks: self.cursor.repeated_ticks,
            region,
            pass: region.map(|_| self.cursor.pass),
            exit_requested: self.cursor.exit_requested,
            ended: self.cursor.tick == self.schedule.duration,
        }
    }

    /// Explicit seeking starts a new local pass and discards any pending boundary exit.
    /// # Errors
    /// Reject out-of-source positions without modifying the previous state.
    pub fn seek(&mut self, tick: u64) -> Result<LoopPosition, String> {
        if tick > self.schedule.duration {
            return Err("演出定位超出源时间范围".into());
        }
        self.cursor = Cursor {
            tick,
            repeated_ticks: 0,
            next_region: self.schedule.regions.partition_point(|r| r.end <= tick),
            pass: 1,
            exit_requested: false,
        };
        Ok(self.position())
    }

    /// Request/cancel an exit from this pass, without jumping. Repeated requests are idempotent.
    /// # Errors
    /// Reject a stale index or a request outside a currently active region.
    pub fn set_exit_at_end(&mut self, region: usize, requested: bool) -> Result<(), String> {
        if self.position().region != Some(region) {
            return Err("当前循环区段已变化，请重新确认退出目标".into());
        }
        self.cursor.exit_requested = requested;
        Ok(())
    }

    /// Advance by consumed source-independent ticks; success does not allocate or replay passes.
    /// # Errors
    /// Reject an overflowing pass counter and preserve the entire original cursor.
    pub fn advance(&mut self, mut ticks: u64) -> Result<LoopPosition, String> {
        let mut cursor = self.cursor;
        while ticks > 0 {
            let Some(region) = self.schedule.regions.get(cursor.next_region) else {
                cursor.tick += ticks.min(self.schedule.duration - cursor.tick);
                break;
            };
            if cursor.tick < region.start {
                let consumed = ticks.min(region.start - cursor.tick);
                cursor.tick += consumed;
                ticks -= consumed;
                continue;
            }
            let length = region.end - region.start;
            let remaining_passes = if cursor.exit_requested {
                Some(1)
            } else {
                match region.plays {
                    LoopPlays::Count(count) => Some(u64::from(count) - cursor.pass + 1),
                    LoopPlays::UntilExit => None,
                }
            };
            let until_exit = remaining_passes.map(|passes| {
                u128::from(region.end - cursor.tick) + u128::from(passes - 1) * u128::from(length)
            });
            if let Some(until_exit) = until_exit.filter(|&v| v <= u128::from(ticks)) {
                let repeats = remaining_passes.ok_or("循环剩余次数无效")? - 1;
                cursor.add_repeats(repeats, length)?;
                ticks -= u64::try_from(until_exit).map_err(|_| "循环剩余时间超出范围")?;
                cursor.tick = region.end;
                cursor.next_region += 1;
                cursor.pass = 1;
                cursor.exit_requested = false;
                continue;
            }
            let total = u128::from(cursor.tick - region.start) + u128::from(ticks);
            let completed =
                u64::try_from(total / u128::from(length)).map_err(|_| "循环播放次数超出范围")?;
            cursor.add_repeats(completed, length)?;
            cursor.pass = cursor
                .pass
                .checked_add(completed)
                .ok_or("循环播放次数超出范围")?;
            cursor.tick = region.start
                + u64::try_from(total % u128::from(length)).map_err(|_| "循环余量超出范围")?;
            break;
        }
        self.cursor = cursor;
        Ok(self.position())
    }
}

impl Cursor {
    fn add_repeats(&mut self, count: u64, length: u64) -> Result<(), String> {
        self.repeated_ticks = count
            .checked_mul(length)
            .and_then(|ticks| self.repeated_ticks.checked_add(ticks))
            .ok_or("循环累计回跳时间已耗尽")?;
        Ok(())
    }
}
