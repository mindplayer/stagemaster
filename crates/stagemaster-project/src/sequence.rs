//! Scene-list editing and validation; no running player state lives in the document.
use crate::{array, id, text};
use serde::Deserialize;
use serde_json::{Value, json};
use stagemaster_playback::MAX_TIME_MS;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum SequenceEdit {
    Add {
        name: String,
        scene_id: String,
    },
    Update {
        id: String,
        name: String,
        tracking: Tracking,
        repeat: Repeat,
    },
    Duplicate {
        id: String,
        name: String,
    },
    Remove {
        id: String,
    },
    InsertStep {
        id: String,
        scene_id: String,
        after_id: Option<String>,
    },
    UpdateStep {
        id: String,
        step_id: String,
        name: String,
        number: String,
        scene_id: String,
        delay_ms: u64,
        fade_ms: u64,
        wait_ms: Option<u64>,
    },
    UpdateStepScript {
        id: String,
        step_id: String,
        script: Option<crate::StepScript>,
    },
    MoveStep {
        id: String,
        step_id: String,
        index: usize,
    },
    DuplicateStep {
        id: String,
        step_id: String,
    },
    RemoveStep {
        id: String,
        step_id: String,
    },
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tracking {
    Inherited,
    Isolated,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Repeat {
    Once,
    Loop,
}

pub(super) fn apply(root: &mut Value, command: SequenceEdit) -> Result<(), String> {
    use crate::editing::{find, list, remove};
    match command {
        SequenceEdit::Add { name, scene_id } => {
            let scene = find(list(root, "scenes")?, &scene_id)?;
            let step = new_step(&scene_id, text(scene, "name"), "1");
            list(root, "sequences")?.push(json!({"id":id(),"name":name,"tracking":"inherited",
                "repeat":"once","release":"profile-defaults","steps":[step]}));
        }
        SequenceEdit::Update {
            id,
            name,
            tracking,
            repeat,
        } => {
            let seq = find(list(root, "sequences")?, &id)?;
            seq["name"] = name.into();
            seq["tracking"] = match tracking {
                Tracking::Inherited => "inherited",
                Tracking::Isolated => "isolated",
            }
            .into();
            seq["repeat"] = match repeat {
                Repeat::Once => "once",
                Repeat::Loop => "loop",
            }
            .into();
        }
        SequenceEdit::Duplicate { id: source, name } => duplicate(root, &source, &name)?,
        SequenceEdit::Remove { id } => remove(list(root, "sequences")?, &id)?,
        SequenceEdit::InsertStep {
            id,
            scene_id,
            after_id,
        } => {
            let name = text(find(list(root, "scenes")?, &scene_id)?, "name").to_string();
            let steps = steps_mut(root, &id)?;
            let number = next_number(steps)?;
            let index = after_id.map_or(Ok(steps.len()), |after| {
                position(steps, &after).map(|i| i + 1)
            })?;
            steps.insert(index, new_step(&scene_id, &name, &number));
        }
        SequenceEdit::UpdateStep {
            id,
            step_id,
            name,
            number,
            scene_id,
            delay_ms,
            fade_ms,
            wait_ms,
        } => {
            if [Some(delay_ms), Some(fade_ms), wait_ms]
                .into_iter()
                .flatten()
                .any(|v| v > MAX_TIME_MS)
            {
                return Err("每项时间须在 0–86400 秒内".into());
            }
            let step = find(steps_mut(root, &id)?, &step_id)?;
            step["name"] = name.into();
            step["number"] = number.into();
            step["sceneId"] = scene_id.into();
            step["delay"] = duration(delay_ms);
            step["fade"] = duration(fade_ms);
            step["advance"] = wait_ms.map_or_else(
                || json!({"kind":"manual"}),
                |wait| json!({"kind":"after","wait":duration(wait)}),
            );
        }
        SequenceEdit::UpdateStepScript {
            id,
            step_id,
            script,
        } => crate::sequence_script::set(root, &id, &step_id, script)?,
        SequenceEdit::MoveStep { id, step_id, index } => {
            let steps = steps_mut(root, &id)?;
            if index >= steps.len() {
                return Err("目标顺序超出列表范围".into());
            }
            let from = position(steps, &step_id)?;
            let step = steps.remove(from);
            steps.insert(index, step);
        }
        SequenceEdit::DuplicateStep { id, step_id } => duplicate_step(root, &id, &step_id)?,
        SequenceEdit::RemoveStep { id, step_id } => {
            let steps = steps_mut(root, &id)?;
            if steps.len() == 1 {
                return Err("列表需要至少一个步骤；如不再使用，请删除整个列表".into());
            }
            remove(steps, &step_id)?;
        }
    }
    crate::sequence_script::sync_capability(root);
    Ok(())
}
fn duplicate_step(root: &mut Value, sequence: &str, step_id: &str) -> Result<(), String> {
    let steps = steps_mut(root, sequence)?;
    let index = position(steps, step_id)?;
    let mut copy = steps[index].clone();
    copy["id"] = id().into();
    copy["number"] = next_number(steps)?.into();
    steps.insert(index + 1, copy);
    Ok(())
}
fn duplicate(root: &mut Value, source: &str, name: &str) -> Result<(), String> {
    use crate::editing::{find, list};
    let mut copy = find(list(root, "sequences")?, source)?.clone();
    copy["id"] = id().into();
    copy["name"] = name.into();
    for step in copy["steps"].as_array_mut().ok_or("列表步骤无效")? {
        step["id"] = id().into();
    }
    list(root, "sequences")?.push(copy);
    Ok(())
}

fn steps_mut<'a>(root: &'a mut Value, id: &str) -> Result<&'a mut Vec<Value>, String> {
    crate::editing::find(crate::editing::list(root, "sequences")?, id)?["steps"]
        .as_array_mut()
        .ok_or_else(|| "列表步骤无效".into())
}
fn position(steps: &[Value], id: &str) -> Result<usize, String> {
    steps
        .iter()
        .position(|s| s["id"] == id)
        .ok_or_else(|| "步骤已不存在".into())
}
fn next_number(steps: &[Value]) -> Result<String, String> {
    let used = steps
        .iter()
        .map(|s| number_key(text(s, "number")))
        .collect::<Result<BTreeSet<_>, _>>()?;
    (1..=999_999)
        .find(|n| !used.contains(&(n * 1000)))
        .map(|n| n.to_string())
        .ok_or_else(|| "没有可用编号".into())
}
fn duration(ms: u64) -> Value {
    json!({"ticks":ms.to_string(),"ticksPerSecond":"1000"})
}
fn new_step(scene_id: &str, name: &str, number: &str) -> Value {
    json!({"id":id(),"name":name,"number":number,"sceneId":scene_id,"actionIds":[],
        "delay":duration(0),"fade":duration(1000),"advance":{"kind":"manual"}})
}
fn number_key(number: &str) -> Result<u32, String> {
    let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
    let whole = whole.parse::<u32>().map_err(|_| "步骤编号无效")?;
    let fraction = format!("{fraction:0<3}")
        .parse::<u32>()
        .map_err(|_| "步骤编号无效")?;
    whole
        .checked_mul(1000)
        .and_then(|v| v.checked_add(fraction))
        .ok_or_else(|| "步骤编号超出范围".into())
}
pub(super) fn duration_ms(time: &Value) -> Result<u64, String> {
    let ticks = text(time, "ticks")
        .parse::<i64>()
        .map_err(|_| "时间刻度超出整数范围")?;
    let rate = text(time, "ticksPerSecond")
        .parse::<u64>()
        .map_err(|_| "时间时基无效")?;
    if ticks < 0 || rate == 0 || rate > 1_000_000_000 {
        return Err("时间或时基超出范围".into());
    }
    let scaled = u128::try_from(ticks).map_err(|_| "时间不能为负")? * 1000;
    if scaled % u128::from(rate) != 0 {
        return Err("当前播放支持毫秒精度，请使用可精确表示的时间".into());
    }
    let ms = scaled / u128::from(rate);
    if ms > u128::from(MAX_TIME_MS) {
        return Err("每项时间须在 0–86400 秒内".into());
    }
    Ok(u64::try_from(ms).expect("bounded milliseconds"))
}
pub(super) fn validate(root: &Value) -> Result<(), String> {
    let lighting = &root["lighting"];
    let scenes: BTreeSet<_> = array(lighting, "scenes")
        .iter()
        .map(|s| text(s, "id"))
        .collect();
    for seq in array(lighting, "sequences") {
        let mut numbers = BTreeSet::new();
        let mut total_ms = 0;
        let mut automatic = true;
        for step in array(seq, "steps") {
            let context = format!("列表“{}”第 {} 步", text(seq, "name"), text(step, "number"));
            if !scenes.contains(text(step, "sceneId")) {
                return Err(format!("{context}的场景引用不存在；请先移除列表中的引用"));
            }
            if !numbers.insert(number_key(text(step, "number"))?) {
                return Err(format!("{context}编号重复（1 与 1.0 视为相同）"));
            }
            if !array(step, "actionIds").is_empty() {
                return Err(format!("{context}包含尚不支持的外部动作"));
            }
            let times = (|| {
                let mut ms = duration_ms(&step["delay"])? + duration_ms(&step["fade"])?;
                if step["advance"]["kind"] == "after" {
                    ms += duration_ms(&step["advance"]["wait"])?;
                } else {
                    automatic = false;
                }
                Ok::<u64, String>(ms)
            })()
            .map_err(|error| format!("{context}：{error}"))?;
            total_ms += times;
        }
        if seq["repeat"] == "loop" && automatic && total_ms == 0 {
            return Err(format!(
                "列表“{}”的自动循环总时长不能为零",
                text(seq, "name")
            ));
        }
    }
    Ok(())
}
