//! Space membership is organizational; placements and platforms always use world coordinates.
use crate::{array, id, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stagemaster_spatial::polygon::floor_plan;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpatialVector3 {
    pub x: String,
    pub y: String,
    pub z: String,
}
impl SpatialVector3 {
    /// Convert validated decimal fields for calculation, keeping strings in the document.
    /// # Errors
    /// Rejects non-finite numbers or values outside the caller's limit.
    pub fn numbers(&self, limit: f64) -> Result<[f64; 3], String> {
        Ok([
            decimal(&self.x, -limit, limit)?,
            decimal(&self.y, -limit, limit)?,
            decimal(&self.z, -limit, limit)?,
        ])
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageSpace {
    pub id: String,
    pub name: String,
    pub outline_meters: Vec<[String; 2]>,
    pub floor_elevation_meters: String,
    pub clear_height_meters: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ConstructionShape {
    Enclosure {
        space_id: String,
        wall_thickness_meters: String,
        floor_thickness_meters: String,
        ceiling_thickness_meters: Option<String>,
    },
    Platform {
        space_id: Option<String>,
        outline_meters: Vec<[String; 2]>,
        base_elevation_meters: String,
        height_meters: String,
    },
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageConstruction {
    pub id: String,
    pub name: String,
    pub shape: ConstructionShape,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FixturePlacement {
    pub fixture_id: String,
    pub space_id: Option<String>,
    pub position_meters: SpatialVector3,
    #[serde(rename = "rotationDegreesXYZ")]
    pub rotation_degrees_xyz: SpatialVector3,
}
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageView {
    pub spaces: Vec<StageSpace>,
    pub constructions: Vec<StageConstruction>,
    pub placements: Vec<FixturePlacement>,
}
#[derive(Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StageEdit {
    PutSpace {
        id: Option<String>,
        name: String,
        outline_meters: Vec<[String; 2]>,
        floor_elevation_meters: String,
        clear_height_meters: Option<String>,
    },
    DuplicateSpace {
        id: String,
        name: String,
    },
    RemoveSpace {
        id: String,
        detach_members: bool,
    },
    PutConstruction {
        id: Option<String>,
        name: String,
        shape: ConstructionShape,
    },
    DuplicateConstruction {
        id: String,
        name: String,
    },
    RemoveConstruction {
        id: String,
    },
    PutPlacement {
        placement: FixturePlacement,
    },
    RemovePlacement {
        fixture_id: String,
    },
}

fn read(root: &Value) -> Result<StageView, String> {
    serde_json::from_value(json!({
        "spaces": array(&root["stage"], "spaces"),
        "constructions": array(&root["stage"], "constructions"),
        "placements": array(&root["stage"], "placements")
    }))
    .map_err(|_| "场地数据无效".into())
}
pub(super) fn view(root: &Value) -> StageView {
    read(root).expect("validated stage projection")
}
pub(super) fn validate(root: &Value) -> Result<(), String> {
    if !array(&root["stage"], "nodes").is_empty() {
        return Err("当前版本尚不支持旧场地节点／外部模型，工程未打开".into());
    }
    let stage = read(root)?;
    let spaces = stage
        .spaces
        .iter()
        .map(|s| s.id.as_str())
        .collect::<BTreeSet<_>>();
    for space in &stage.spaces {
        outline(&space.outline_meters).map_err(|e| format!("空间“{}”：{e}", space.name))?;
        decimal(&space.floor_elevation_meters, -10_000.0, 10_000.0)?;
        if let Some(height) = &space.clear_height_meters {
            decimal(height, 0.1, 1_000.0)?;
        }
    }
    let mut enclosed = BTreeSet::new();
    for construction in &stage.constructions {
        match &construction.shape {
            ConstructionShape::Enclosure {
                space_id,
                wall_thickness_meters,
                floor_thickness_meters,
                ceiling_thickness_meters,
            } => {
                member(Some(space_id), &spaces)?;
                if !enclosed.insert(space_id) {
                    return Err("一个空间只能有一组围护构件".into());
                }
                let space = stage
                    .spaces
                    .iter()
                    .find(|s| s.id == *space_id)
                    .expect("checked membership");
                if space.clear_height_meters.is_none() {
                    return Err("添加围护前需要设置空间净高".into());
                }
                decimal(wall_thickness_meters, 0.001, 10.0)?;
                decimal(floor_thickness_meters, 0.001, 10.0)?;
                if let Some(value) = ceiling_thickness_meters {
                    decimal(value, 0.001, 10.0)?;
                }
            }
            ConstructionShape::Platform {
                space_id,
                outline_meters,
                base_elevation_meters,
                height_meters,
            } => {
                member(space_id.as_deref(), &spaces)?;
                outline(outline_meters).map_err(|e| format!("构件“{}”：{e}", construction.name))?;
                decimal(base_elevation_meters, -10_000.0, 10_000.0)?;
                decimal(height_meters, 0.001, 1_000.0)?;
            }
        }
    }
    let fixtures = array(&root["lighting"], "fixtures")
        .iter()
        .map(|f| text(f, "id"))
        .collect::<BTreeSet<_>>();
    let mut placed = BTreeSet::new();
    for placement in &stage.placements {
        member(placement.space_id.as_deref(), &spaces)?;
        if !fixtures.contains(placement.fixture_id.as_str()) {
            return Err("灯位引用的灯具不存在；请先移除灯位再删除灯具".into());
        }
        if !placed.insert(&placement.fixture_id) {
            return Err("同一灯具不能重复布置".into());
        }
        placement.position_meters.numbers(100_000.0)?;
        placement.rotation_degrees_xyz.numbers(3_600.0)?;
    }
    Ok(())
}
fn decimal(value: &str, min: f64, max: f64) -> Result<f64, String> {
    let number = value.parse::<f64>().map_err(|_| "空间尺寸需要有效数字")?;
    if !number.is_finite() || number < min || number > max {
        return Err(format!("空间数值必须在 {min}–{max} 之间"));
    }
    Ok(number)
}
fn outline(points: &[[String; 2]]) -> Result<(), String> {
    let points = points
        .iter()
        .map(|p| {
            Ok([
                decimal(&p[0], -100_000.0, 100_000.0)?,
                decimal(&p[1], -100_000.0, 100_000.0)?,
            ])
        })
        .collect::<Result<Vec<_>, String>>()?;
    floor_plan(&points).map(|_| ())
}
fn member(id: Option<&str>, spaces: &BTreeSet<&str>) -> Result<(), String> {
    if id.is_some_and(|id| !spaces.contains(id)) {
        Err("所属空间不存在".into())
    } else {
        Ok(())
    }
}

pub(super) fn apply(root: &mut Value, command: StageEdit) -> Result<(), String> {
    if root.get("stage").is_none() {
        root["stage"] = json!({"nodes":[], "spaces":[], "constructions":[], "placements":[]});
    }
    for key in ["spaces", "constructions", "placements"] {
        if root["stage"].get(key).is_none() {
            root["stage"][key] = json!([]);
        }
    }
    let capabilities = root["requires"].as_array_mut().ok_or("工程能力声明缺失")?;
    for key in ["stage.layout", "stage.spaces"] {
        if !capabilities.iter().any(|c| c["key"] == key) {
            capabilities.push(json!({"key":key,"version":1}));
        }
    }
    match command {
        StageEdit::PutSpace {
            id,
            name,
            outline_meters,
            floor_elevation_meters,
            clear_height_meters,
        } => {
            put(
                root,
                "spaces",
                id.as_deref(),
                json!({"name":name,"outlineMeters":outline_meters,"floorElevationMeters":floor_elevation_meters,"clearHeightMeters":clear_height_meters}),
            )?;
        }
        StageEdit::PutConstruction { id, name, shape } => {
            put(
                root,
                "constructions",
                id.as_deref(),
                json!({"name":name,"shape":shape}),
            )?;
        }
        StageEdit::DuplicateSpace { id: source, name } => {
            let copy_id = duplicate(root, "spaces", &source, &name)?;
            let mut enclosure = array(&root["stage"], "constructions")
                .iter()
                .find(|c| c["shape"]["kind"] == "enclosure" && c["shape"]["spaceId"] == source)
                .cloned();
            if let Some(ref mut enclosure) = enclosure {
                enclosure["id"] = id().into();
                enclosure["name"] = format!("{name}围护").into();
                enclosure["shape"]["spaceId"] = copy_id.into();
                list(root, "constructions")?.push(enclosure.clone());
            }
        }
        StageEdit::DuplicateConstruction { id, name } => {
            if array(&root["stage"], "constructions")
                .iter()
                .any(|c| c["id"] == id && c["shape"]["kind"] == "enclosure")
            {
                return Err("围护随空间复制，不能在同一空间重复创建".into());
            }
            duplicate(root, "constructions", &id, &name)?;
        }
        StageEdit::RemoveSpace { id, detach_members } => remove_space(root, &id, detach_members)?,
        StageEdit::RemoveConstruction { id } => {
            crate::editing::remove(list(root, "constructions")?, &id)?;
        }
        StageEdit::PutPlacement { placement } => {
            let values = list(root, "placements")?;
            let index = values
                .iter()
                .position(|p| p["fixtureId"] == placement.fixture_id);
            let value = json!(placement);
            if let Some(index) = index {
                values[index] = value;
            } else {
                values.push(value);
            }
        }
        StageEdit::RemovePlacement { fixture_id } => {
            let values = list(root, "placements")?;
            let index = values
                .iter()
                .position(|p| p["fixtureId"] == fixture_id)
                .ok_or("灯具尚未布置")?;
            values.remove(index);
        }
    }
    Ok(())
}
fn list<'a>(root: &'a mut Value, key: &str) -> Result<&'a mut Vec<Value>, String> {
    root["stage"][key]
        .as_array_mut()
        .ok_or_else(|| "场地数据缺失".into())
}
fn put(
    root: &mut Value,
    key: &str,
    existing: Option<&str>,
    mut value: Value,
) -> Result<(), String> {
    let values = list(root, key)?;
    if let Some(id) = existing {
        value["id"] = id.into();
        *crate::editing::find(values, id)? = value;
    } else {
        value["id"] = id().into();
        values.push(value);
    }
    Ok(())
}
fn duplicate(root: &mut Value, key: &str, source: &str, name: &str) -> Result<String, String> {
    let values = list(root, key)?;
    let mut copy = crate::editing::find(values, source)?.clone();
    let new_id = id();
    copy["id"] = new_id.clone().into();
    copy["name"] = name.into();
    values.push(copy);
    Ok(new_id)
}
fn remove_space(root: &mut Value, id: &str, detach: bool) -> Result<(), String> {
    let stage = &root["stage"];
    let used = array(stage, "placements")
        .iter()
        .any(|p| p["spaceId"] == id)
        || array(stage, "constructions")
            .iter()
            .any(|c| c["shape"]["spaceId"] == id);
    if used && !detach {
        return Err("空间仍有灯位或构件；需要明确选择解除归属并移除围护".into());
    }
    crate::editing::remove(list(root, "spaces")?, id)?;
    for placement in list(root, "placements")? {
        if placement["spaceId"] == id {
            placement["spaceId"] = Value::Null;
        }
    }
    let constructions = list(root, "constructions")?;
    constructions.retain(|c| !(c["shape"]["kind"] == "enclosure" && c["shape"]["spaceId"] == id));
    for construction in constructions {
        if construction["shape"]["spaceId"] == id {
            construction["shape"]["spaceId"] = Value::Null;
        }
    }
    Ok(())
}
