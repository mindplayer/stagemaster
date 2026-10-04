use crate::{Catalog, Selection, State};

pub(crate) fn catalog(catalog: &Catalog) -> Result<(), String> {
    crate::manual_validation::catalog(catalog)?;
    if catalog.protocol != 2
        || catalog.execution != "sourceGroup"
        || catalog.mode != "softwareOutput"
        || catalog.physical_output
        || !(1..=64).contains(&catalog.sources.len())
    {
        return Err("后台能力与当前客户端不兼容".into());
    }
    let audio: Vec<_> = catalog
        .sources
        .iter()
        .filter(|s| matches!(s.selection, Selection::AudioTimeline {}))
        .collect();
    let declared = catalog
        .capabilities
        .iter()
        .any(|c| c == "backgroundLinearAudio");
    let loops = catalog
        .capabilities
        .iter()
        .any(|c| c == "backgroundAudioLoops");
    if loops != catalog.audio.as_ref().is_some_and(|c| c.performance_loops) {
        return Err("后台循环能力与配置不一致".into());
    }
    let recovery = catalog
        .capabilities
        .iter()
        .any(|c| c == "backgroundAudioRecovery");
    if recovery != catalog.audio.as_ref().is_some_and(|c| c.provider_recovery) {
        return Err("后台音频恢复能力与配置不一致".into());
    }
    match &catalog.audio {
        None if audio.is_empty() && !declared => Ok(()),
        Some(config)
            if declared
                && audio.len() == 1
                && audio[0].id == config.group
                && audio[0].steps.is_empty()
                && config.duration_ms > 0 =>
        {
            identity(&config.group)
        }
        _ => Err("后台音乐能力、来源或配置不一致".into()),
    }
}
pub(crate) fn media_state(catalog: &Catalog, state: &State) -> Result<(), String> {
    let Some(config) = &catalog.audio else {
        return if state.audio.is_none() && state.media.is_empty() {
            Ok(())
        } else {
            Err("后台返回了未声明的音乐状态".into())
        };
    };
    let native = state.audio.as_ref().ok_or("后台缺少原生音频状态")?;
    let group = state.media.first().ok_or("后台缺少音乐同步组")?;
    if state.media.len() != 1
        || group.id != config.group
        || native.output != config.output
        || native.duration_ms != config.duration_ms
        || native.position_ms > config.duration_ms
        || group.position_ms > config.duration_ms
    {
        return Err("后台音频身份、输出或位置不一致".into());
    }
    decimal(&group.generation)?;
    decimal(&native.frames)?;
    if let Some(current) = &native.loop_state {
        if !config.performance_loops
            || current.region >= 128
            || current.name.trim().is_empty()
            || native.instance.is_none()
        {
            return Err("后台循环状态与能力不一致".into());
        }
        nonzero(&current.pass)?;
    }
    if let Some(instance) = &native.instance {
        nonzero(instance)?;
    }
    if let Some(control) = &group.control {
        nonzero(&control.request)?;
    }
    if let Some(terminal) = &group.termination {
        decimal(&terminal.generation)?;
    }
    Ok(())
}
pub(crate) fn decimal(value: &str) -> Result<u64, String> {
    let parsed: u64 = value.parse().map_err(|_| "后台计数格式无效")?;
    if parsed.to_string() != value {
        return Err("后台计数格式无效".into());
    }
    Ok(parsed)
}
fn nonzero(value: &str) -> Result<(), String> {
    if decimal(value)? == 0 {
        return Err("后台实例或请求标识无效".into());
    }
    Ok(())
}
pub(crate) fn identity(value: &str) -> Result<(), String> {
    let parsed = uuid::Uuid::parse_str(value).map_err(|_| "后台音乐组身份无效")?;
    if parsed.is_nil() || parsed.to_string() != value {
        return Err("后台音乐组身份无效".into());
    }
    Ok(())
}
