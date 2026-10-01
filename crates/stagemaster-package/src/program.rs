use crate::effect_codec::{read_effect, write_effect};
use crate::{
    Error, MAX_LOADER_BYTES, MAX_PROGRAM_BYTES, MAX_STEPS, Mapping, Output, Program, StepLabel,
    Usage,
    codec::{array, count, encoder, end, id, read_text, text, values, write_values},
    occupy, own, reserve,
};
use alloc::vec::Vec;
use minicbor::{Decoder, data::Type};
use stagemaster_playback::{MAX_TIME_MS, Plan, Step};

/// Encode a program into a bounded CBOR block and validate it using the independent decoder.
/// # Errors
/// Rejects invalid labels, mappings, dimensions, encoded size and reference memory limits.
pub fn encode_program(program: &Program) -> Result<Vec<u8>, Error> {
    if program.plan.effect_time_offset_ms() != 0 {
        return Err(Error::Invalid("当前设备包不支持片段效果时间偏移"));
    }
    if program.plan.steps().len() > MAX_STEPS {
        return Err(Error::Limit("每个节目最多 128 步；请拆分列表"));
    }
    if program
        .plan
        .effects()
        .iter()
        .any(|channels| channels.len() > 128)
    {
        return Err(Error::Limit("每步最多 128 个效果通道"));
    }
    if program.labels.len() != program.plan.steps().len() {
        return Err(Error::Invalid("步骤标签数量"));
    }
    let mut e = encoder(MAX_PROGRAM_BYTES);
    let snap = program.plan.snap_attributes();
    e.array(if snap.is_empty() { 5 } else { 6 })?
        .u16(program.output.universe)?
        .array(program.output.mappings.len() as u64)?;
    for m in &program.output.mappings {
        e.array(2)?.u16(m.coarse)?.u16(m.fine.unwrap_or(0))?;
    }
    write_values(&mut e, program.plan.defaults())?;
    e.bool(program.plan.repeat())?
        .array(program.labels.len() as u64)?;
    for ((step, label), effects) in program
        .plan
        .steps()
        .iter()
        .zip(&program.labels)
        .zip(program.plan.effects())
    {
        e.array(8)?
            .bytes(&label.id)?
            .str(text(&label.name)?)?
            .str(text(&label.number)?)?;
        write_values(&mut e, &step.target)?;
        e.u64(step.delay_ms)?.u64(step.fade_ms)?;
        if let Some(wait) = step.wait_ms {
            e.u64(wait)?;
        } else {
            e.null()?;
        }
        e.array(effects.len() as u64)?;
        for channel in effects {
            write_effect(&mut e, channel)?;
        }
    }
    if !snap.is_empty() {
        e.array(snap.len() as u64)?;
        for &index in snap {
            e.u16(index)?;
        }
    }
    let bytes = e.into_writer().bytes;
    scan(&bytes, 0)?;
    Ok(bytes)
}
/// Scan before allocating the plan; catalogue memory must be included by archive callers.
/// # Errors
/// Rejects malformed, excessive or unsupported program content without starting playback.
pub fn decode_program(bytes: &[u8], catalog_bytes: usize) -> Result<(Program, Usage), Error> {
    let usage = scan(bytes, catalog_bytes)?.usage;
    let (_, program) = parse(bytes, true)?;
    Ok((program.ok_or(Error::Invalid("缺少装载结果"))?, usage))
}
pub(crate) struct Scanned {
    pub usage: Usage,
    pub universe: u16,
    pub held_scene: bool,
    pub semantics: u16,
}
pub(crate) fn scan(bytes: &[u8], catalog_bytes: usize) -> Result<Scanned, Error> {
    if bytes.len() > MAX_PROGRAM_BYTES {
        return Err(Error::Limit("单节目块 32 KiB"));
    }
    let (mut result, _) = parse(bytes, false)?;
    result.usage.encoded_bytes = bytes.len();
    result.usage.loader_peak_bytes = result
        .usage
        .resident_bytes
        .checked_add(bytes.len())
        .and_then(|n| n.checked_add(catalog_bytes))
        .and_then(|n| n.checked_add(8192))
        .ok_or(Error::Limit("装载峰值计算溢出"))?;
    if result.usage.loader_peak_bytes > MAX_LOADER_BYTES {
        return Err(Error::Limit("单节目装载峰值 64 KiB；请减少步骤或灯具属性"));
    }
    Ok(result)
}
pub(super) struct Accounting {
    pub(super) usage: Usage,
    strings: usize,
    pub(super) allocations: usize,
}
impl Accounting {
    fn new(attributes: usize, steps: usize) -> Self {
        Self {
            usage: Usage {
                attributes,
                steps,
                value_bytes: (steps + 3) * attributes * 2,
                ..Usage::default()
            },
            strings: 0,
            allocations: 8 + steps,
        }
    }
    fn label(&mut self, value: &str) {
        self.strings += value.len();
        self.allocations += 1;
    }
    fn finish(mut self) -> Usage {
        // Conservative sizes for Step (56), StepLabel (64), per-step effect Vec (24),
        // EffectChannel (<= 80), Keyframe (<= 8), Mapping (<= 8); allow 32 bytes per allocation.
        self.usage.resident_bytes = 512
            + self.usage.value_bytes
            + self.usage.snap_attributes * 2
            + self.usage.steps * 144
            + self.usage.attributes * 8
            + self.usage.effect_channels * 80
            + self.usage.keyframes * 8
            + self.strings
            + self.allocations * 32;
        self.usage
    }
}
fn duration(d: &mut Decoder<'_>) -> Result<u64, Error> {
    let time = d.u64()?;
    if time > MAX_TIME_MS {
        return Err(Error::Invalid("时间超过 86400 秒"));
    }
    Ok(time)
}
fn parse(bytes: &[u8], build: bool) -> Result<(Scanned, Option<Program>), Error> {
    let mut d = Decoder::new(bytes);
    let semantics = match d.array()? {
        Some(5) => 1,
        Some(6) => 2,
        _ => return Err(Error::Version),
    };
    let (universe, attributes, mappings) = read_output(&mut d, build)?;
    let defaults = values(&mut d, attributes, build)?;
    let repeat = d.bool()?;
    let step_count = count(&mut d, MAX_STEPS)?;
    if step_count == 0 {
        return Err(Error::Invalid("节目没有步骤"));
    }
    let mut stats = Accounting::new(attributes, step_count);
    let mut steps = reserve(if build { step_count } else { 0 })?;
    let mut effects = reserve(if build { step_count } else { 0 })?;
    let mut labels = reserve(if build { step_count } else { 0 })?;
    let mut ids = [[0; 16]; MAX_STEPS];
    let mut total_time = Some(0_u64);
    let mut held_scene = step_count == 1 && !repeat;
    let mut effect_attributes = [0_u64; 8];
    for index in 0..step_count {
        array(&mut d, 8)?;
        let id = id(&mut d)?;
        if ids[..index].contains(&id) {
            return Err(Error::Invalid("步骤标识重复"));
        }
        ids[index] = id;
        let name = read_text(&mut d)?;
        let number = read_text(&mut d)?;
        stats.label(name);
        stats.label(number);
        let target = values(&mut d, attributes, build)?;
        let delay_ms = duration(&mut d)?;
        let fade_ms = duration(&mut d)?;
        let wait_ms = if d.datatype()? == Type::Null {
            d.null()?;
            None
        } else {
            Some(duration(&mut d)?)
        };
        total_time = total_time
            .zip(wait_ms)
            .map(|(total, wait)| total + delay_ms + fade_ms + wait);
        held_scene &= delay_ms == 0 && fade_ms == 0 && wait_ms.is_none();
        let channels = read_effects(
            &mut d,
            attributes,
            &mut stats,
            build,
            &mut effect_attributes,
        )?;
        if build {
            steps.push(Step {
                target,
                delay_ms,
                fade_ms,
                wait_ms,
            });
            effects.push(channels);
            labels.push(StepLabel {
                id,
                name: own(name)?,
                number: own(number)?,
            });
        }
    }
    if repeat && total_time == Some(0) {
        return Err(Error::Invalid("自动循环时长为零"));
    }
    if stats.usage.effect_channels > stagemaster_playback::MAX_EFFECT_CHANNELS
        || stats.usage.keyframes > stagemaster_playback::MAX_KEYFRAMES
    {
        return Err(Error::Limit("效果计划容量"));
    }
    let snap = if semantics == 2 {
        read_snap(&mut d, attributes, &effect_attributes, &mut stats, build)?
    } else {
        Vec::new()
    };
    end(&d, bytes)?;
    let result = Scanned {
        usage: stats.finish(),
        universe,
        held_scene,
        semantics,
    };
    let program = if build {
        Some(Program {
            output: Output { universe, mappings },
            labels,
            plan: Plan::with_snap_attributes(defaults, steps, repeat, effects, snap)
                .map_err(Error::Plan)?,
        })
    } else {
        None
    };
    Ok((result, program))
}

fn read_output(d: &mut Decoder<'_>, build: bool) -> Result<(u16, usize, Vec<Mapping>), Error> {
    let universe = d.u16()?;
    if universe == 0 {
        return Err(Error::Invalid("线路编号不能为零"));
    }
    let attributes = count(d, 512)?;
    if attributes == 0 {
        return Err(Error::Invalid("没有输出属性"));
    }
    let mut slots = [false; 512];
    let mut mappings = reserve(if build { attributes } else { 0 })?;
    for _ in 0..attributes {
        array(d, 2)?;
        let coarse = d.u16()?;
        let fine = d.u16()?;
        occupy(&mut slots, coarse)?;
        if fine != 0 {
            occupy(&mut slots, fine)?;
        }
        if build {
            mappings.push(Mapping {
                coarse,
                fine: (fine != 0).then_some(fine),
            });
        }
    }
    Ok((universe, attributes, mappings))
}

fn read_snap(
    d: &mut Decoder<'_>,
    attributes: usize,
    effect_attributes: &[u64; 8],
    stats: &mut Accounting,
    build: bool,
) -> Result<Vec<u16>, Error> {
    let n = count(d, attributes)?;
    if n == 0 {
        return Err(Error::Invalid("直接切换索引不能为空"));
    }
    let mut snap = reserve(if build { n } else { 0 })?;
    stats.usage.snap_attributes = n;
    stats.allocations += 1;
    let mut previous = None;
    for _ in 0..n {
        let index = d.u16()?;
        if usize::from(index) >= attributes || previous.is_some_and(|p| index <= p) {
            return Err(Error::Invalid("直接切换索引越界、重复或未递增"));
        }
        let index_usize = usize::from(index);
        if effect_attributes[index_usize / 64] & (1 << (index_usize % 64)) != 0 {
            return Err(Error::Invalid("直接切换属性与动态效果冲突"));
        }
        previous = Some(index);
        if build {
            snap.push(index);
        }
    }
    Ok(snap)
}

fn read_effects(
    d: &mut Decoder<'_>,
    attributes: usize,
    stats: &mut Accounting,
    build: bool,
    effect_attributes: &mut [u64; 8],
) -> Result<Vec<stagemaster_playback::EffectChannel>, Error> {
    let channel_count = count(d, 128)?;
    let mut occupied = [false; 512];
    let mut channels = reserve(if build { channel_count } else { 0 })?;
    if channel_count > 0 {
        stats.allocations += 1;
    }
    stats.usage.effect_channels += channel_count;
    for _ in 0..channel_count {
        let channel = read_effect(d, attributes, &mut occupied, stats, build)?;
        if let Some(channel) = channel {
            channels.push(channel);
        }
    }
    for (index, used) in occupied.into_iter().enumerate() {
        if used {
            effect_attributes[index / 64] |= 1 << (index % 64);
        }
    }
    Ok(channels)
}
