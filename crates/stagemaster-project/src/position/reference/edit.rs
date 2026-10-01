use super::{
    CAPABILITY, MAX_POINTS_PER_FIXTURE, PositionReference, ReferencePoint, ReferenceSource,
    matches_profile, read,
};
use crate::{PositionEdit, SpatialVector3, array, editing, text};
use serde_json::{Value, json};
use uuid::Uuid;

fn capture(
    root: &mut Value,
    scene_id: &str,
    fixture_id: &str,
    name: &str,
    target: SpatialVector3,
) -> Result<(), String> {
    let (fixture, profile) = super::super::fixture_profile(root, fixture_id)?;
    let mut record = read(fixture)?.unwrap_or(PositionReference {
        profile_id: text(profile, "id").into(),
        profile_revision: text(profile, "revision").into(),
        points: vec![],
    });
    if !matches_profile(&record, profile) {
        return Err("灯具档案已改变，请清除旧参考记录后重新记录".into());
    }
    if record.points.len() >= MAX_POINTS_PER_FIXTURE {
        return Err("这台灯具已有 16 个参考点".into());
    }
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 256 {
        return Err("参考点名称需要 1–256 个字符".into());
    }
    if record.points.iter().any(|p| p.name.trim() == name) {
        return Err("这台灯具已有同名参考点".into());
    }
    let scene = array(&root["lighting"], "scenes")
        .iter()
        .find(|s| s["id"] == scene_id)
        .ok_or("场景不存在")?;
    for effect in array(scene, "effects") {
        if effect["enabled"] == true
            && array(effect, "fixtureIds").contains(&fixture["id"])
            && array(effect, "channels")
                .iter()
                .any(|c| c["attribute"] == "pan" || c["attribute"] == "tilt")
        {
            return Err(format!(
                "位置正由效果“{}”控制，请先停用该效果",
                text(effect, "name")
            ));
        }
    }
    let value = |key| {
        let value = super::super::edit::scene_value(root, scene, fixture_id, key, profile);
        if super::super::fine(profile, key) {
            value
        } else {
            (value >> 8) * 257
        }
    };
    let point = ReferencePoint {
        id: Uuid::new_v4().to_string(),
        name: name.into(),
        target_meters: target,
        pan_value: value("pan"),
        tilt_value: value("tilt"),
        source: ReferenceSource::SceneSetpoint,
    };
    super::view::check(root, fixture, profile, &point)?;
    record.points.push(point);
    editing::find(editing::list(root, "fixtures")?, fixture_id)?["positionReference"] =
        json!(record);
    if !array(root, "requires")
        .iter()
        .any(|c| c["key"] == CAPABILITY)
    {
        root["requires"]
            .as_array_mut()
            .expect("validated capabilities")
            .push(json!({"key":CAPABILITY,"version":1}));
    }
    Ok(())
}
pub(in crate::position) fn apply(root: &mut Value, command: PositionEdit) -> Result<(), String> {
    match command {
        PositionEdit::CaptureReference {
            scene_id,
            fixture_id,
            name,
            target_meters,
        } => capture(root, &scene_id, &fixture_id, &name, target_meters),
        PositionEdit::RemoveReference {
            fixture_id,
            point_id,
        } => {
            let (fixture, _) = super::super::fixture_profile(root, &fixture_id)?;
            let mut record = read(fixture)?.ok_or("这台灯具没有参考点")?;
            let index = record
                .points
                .iter()
                .position(|p| p.id == point_id)
                .ok_or("参考点不存在")?;
            record.points.remove(index);
            let fixture = editing::find(editing::list(root, "fixtures")?, &fixture_id)?;
            if record.points.is_empty() {
                fixture
                    .as_object_mut()
                    .expect("fixture")
                    .remove("positionReference");
            } else {
                fixture["positionReference"] = json!(record);
            }
            Ok(())
        }
        PositionEdit::ClearReferences { fixture_id } => {
            let fixture = editing::find(editing::list(root, "fixtures")?, &fixture_id)?;
            if fixture
                .as_object_mut()
                .expect("fixture")
                .remove("positionReference")
                .is_none()
            {
                return Err("这台灯具没有参考点".into());
            }
            Ok(())
        }
        _ => unreachable!("only reference commands dispatched here"),
    }
}
