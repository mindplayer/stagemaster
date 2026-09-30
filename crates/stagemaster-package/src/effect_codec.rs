//! Independent bounded effect encoding and scanning.
use crate::program::Accounting;
use crate::{
    Error,
    codec::{Encode, array, count},
    reserve,
};
use minicbor::Decoder;
use stagemaster_playback::{Curve, EffectChannel, Keyframe, Transition};

pub(super) fn write_effect(e: &mut Encode, c: &EffectChannel) -> Result<(), Error> {
    let (kind, frames) = match &c.curve {
        Curve::Smooth => (0, &[][..]),
        Curve::Triangle => (1, &[][..]),
        Curve::Pulse => (2, &[][..]),
        Curve::Keyframes(frames) => (3, frames.as_slice()),
    };
    e.array(8)?
        .u64(c.index as u64)?
        .u16(c.low)?
        .u16(c.high)?
        .u32(c.period_ms)?
        .u16(c.phase)?
        .u8(c.duty_percent)?
        .u8(kind)?
        .array(frames.len() as u64)?;
    for f in frames {
        let transition = match f.transition {
            Transition::Hold => 0,
            Transition::Linear => 1,
            Transition::Smooth => 2,
        };
        e.array(3)?.u16(f.phase)?.u16(f.value)?.u8(transition)?;
    }
    Ok(())
}
pub(super) fn read_effect(
    d: &mut Decoder<'_>,
    attributes: usize,
    occupied: &mut [bool; 512],
    stats: &mut Accounting,
    build: bool,
) -> Result<Option<EffectChannel>, Error> {
    array(d, 8)?;
    let index = usize::from(d.u16()?);
    if index >= attributes || occupied[index] {
        return Err(Error::Invalid("效果属性越界或重复"));
    }
    occupied[index] = true;
    let low = d.u16()?;
    let high = d.u16()?;
    let period_ms = d.u32()?;
    let phase = d.u16()?;
    let duty_percent = d.u8()?;
    if !(100..=3_600_000).contains(&period_ms) || !(1..=99).contains(&duty_percent) {
        return Err(Error::Invalid("效果周期或比例"));
    }
    let kind = d.u8()?;
    if kind > 3 {
        return Err(Error::Invalid("未知效果曲线"));
    }
    let n = count(d, 32)?;
    if (kind == 3 && n < 2) || (kind != 3 && n != 0) {
        return Err(Error::Invalid("关键帧数量与曲线类型不符"));
    }
    stats.usage.keyframes += n;
    if n > 0 {
        stats.allocations += 1;
    }
    let mut frames = reserve(if build { n } else { 0 })?;
    let mut previous = None;
    for _ in 0..n {
        array(d, 3)?;
        let phase = d.u16()?;
        let value = d.u16()?;
        if previous.map_or(phase != 0, |p| phase <= p) {
            return Err(Error::Invalid("关键帧须从零开始严格递增"));
        }
        previous = Some(phase);
        let transition = match d.u8()? {
            0 => Transition::Hold,
            1 => Transition::Linear,
            2 => Transition::Smooth,
            _ => return Err(Error::Invalid("未知关键帧过渡")),
        };
        if build {
            frames.push(Keyframe {
                phase,
                value,
                transition,
            });
        }
    }
    let curve = match kind {
        0 => Curve::Smooth,
        1 => Curve::Triangle,
        2 => Curve::Pulse,
        _ => Curve::Keyframes(frames),
    };
    Ok(build.then_some(EffectChannel {
        index,
        low,
        high,
        period_ms,
        phase,
        curve,
        duty_percent,
    }))
}
