use crate::{Catalog, ManualAttribute, ManualEdit, ManualTarget, ManualValue, Selection, State};
use std::collections::HashSet;
const CAPABILITY: &str = "manualOwnership";
fn declared(catalog: &Catalog) -> bool {
    catalog.capabilities.iter().any(|c| c == CAPABILITY)
}
fn named(value: &str) -> bool {
    !value.trim().is_empty() && value.chars().count() <= 256
}
fn attribute<'a>(catalog: &'a Catalog, fixture: &str, key: &str) -> Option<&'a ManualAttribute> {
    catalog
        .fixtures
        .as_ref()?
        .iter()
        .find(|f| f.id == fixture)?
        .attributes
        .iter()
        .find(|a| a.key == key)
}
pub(crate) fn catalog(catalog: &Catalog) -> Result<(), String> {
    if !declared(catalog) {
        return Ok(());
    }
    let fixtures = catalog.fixtures.as_ref().ok_or("后台缺少手动灯具目录")?;
    let limits = catalog.limits.as_ref().ok_or("后台缺少手动操作容量")?;
    if !catalog
        .capabilities
        .iter()
        .any(|c| c == "semanticManualPatch")
        || !(1..=512).contains(&limits.manual_changes)
        || !(512..=8192).contains(&limits.request_bytes)
        || fixtures.is_empty()
        || fixtures.len() > 512
    {
        return Err("后台手动能力或容量不兼容".into());
    }
    let mut identities = HashSet::new();
    let mut total = 0;
    for fixture in fixtures {
        crate::validation::identity(&fixture.id)?;
        if !identities.insert(&fixture.id) || !named(&fixture.name) {
            return Err("后台灯具身份重复或名称无效".into());
        }
        let mut keys = HashSet::new();
        for a in &fixture.attributes {
            total += 1;
            if !named(&a.key) || a.key.len() > 128 || !named(&a.label) || !keys.insert(&a.key) {
                return Err("后台灯具属性无效或重复".into());
            }
            if let Some(table) = &a.function {
                let mut names = HashSet::new();
                let max = if table.fine { u16::MAX } else { 255 };
                if table.functions.is_empty() || table.functions.len() > 64 {
                    return Err("后台功能定义数量无效".into());
                }
                for (index, f) in table.functions.iter().enumerate() {
                    if !named(&f.key)
                        || f.key.len() > 128
                        || !named(&f.name)
                        || !names.insert(&f.key)
                        || !matches!(f.mode.as_str(), "slot" | "range")
                        || f.dmx_from > f.dmx_default
                        || f.dmx_default > f.dmx_to
                        || f.dmx_to > max
                        || (f.mode == "range" && f.dmx_from == f.dmx_to)
                        || table.functions[..index]
                            .iter()
                            .any(|other| f.dmx_from <= other.dmx_to && other.dmx_from <= f.dmx_to)
                    {
                        return Err("后台功能定义无效、重复或重叠".into());
                    }
                }
                value(
                    a,
                    &ManualValue::Function {
                        function_key: table.default.function_key.clone(),
                        position: table.default.position,
                    },
                )?;
            }
        }
    }
    if total > 512 {
        return Err("后台手动属性超过当前布局容量".into());
    }
    Ok(())
}
fn value(a: &ManualAttribute, value: &ManualValue) -> Result<(), String> {
    match (value, &a.function) {
        (ManualValue::Release {}, _) | (ManualValue::Normalized { .. }, None) => Ok(()),
        (
            ManualValue::Function {
                function_key,
                position,
            },
            Some(table),
        ) if table
            .functions
            .iter()
            .any(|f| &f.key == function_key && (f.mode == "range" || *position == 0)) =>
        {
            Ok(())
        }
        _ => Err("手动属性类型或功能位置与后台档案不一致".into()),
    }
}
pub(crate) fn edits(catalog: &Catalog, source: &str, changes: &[ManualEdit]) -> Result<(), String> {
    if !declared(catalog)
        || !catalog
            .sources
            .iter()
            .any(|s| s.id == source && matches!(s.selection, Selection::Manual {}))
    {
        return Err("当前后台未提供手动编程能力".into());
    }
    let max = catalog
        .limits
        .as_ref()
        .ok_or("后台手动容量不可用")?
        .manual_changes;
    if changes.is_empty() || changes.len() > max {
        return Err("手动批量超过后台容量".into());
    }
    let mut seen = HashSet::new();
    for change in changes {
        if !seen.insert((&change.fixture_id, &change.attribute)) {
            return Err("手动批量含重复属性".into());
        }
        let a = attribute(catalog, &change.fixture_id, &change.attribute)
            .ok_or("手动目标不在后台灯具目录中")?;
        value(a, &change.value)?;
    }
    Ok(())
}
pub(crate) fn state(catalog: &Catalog, state: &State) -> Result<(), String> {
    for source in &state.sources {
        let manual = catalog
            .sources
            .iter()
            .any(|s| s.id == source.id && matches!(s.selection, Selection::Manual {}));
        match (&source.held, manual && declared(catalog)) {
            (None, false) => {}
            (Some(held), true) => targets(catalog, held)?,
            _ => return Err("后台手动持有状态与能力不一致".into()),
        }
    }
    Ok(())
}
fn targets(catalog: &Catalog, targets: &[ManualTarget]) -> Result<(), String> {
    if targets.len() > 512 {
        return Err("后台手动持有状态超限".into());
    }
    let mut seen = HashSet::new();
    for t in targets {
        if !seen.insert((&t.fixture_id, &t.attribute))
            || attribute(catalog, &t.fixture_id, &t.attribute).is_none()
        {
            return Err("后台手动持有目标重复或无效".into());
        }
    }
    Ok(())
}
