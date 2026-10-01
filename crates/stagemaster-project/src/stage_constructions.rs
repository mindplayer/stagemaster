//! Cross-object construction validation, including bounded generated geometry.
use crate::stage::{ConstructionShape, StageView, decimal, member, outline};
use std::collections::BTreeSet;
pub(super) fn validate(stage: &StageView, spaces: &BTreeSet<&str>) -> Result<(), String> {
    let mut count = 0;
    for c in &stage.constructions {
        if let ConstructionShape::Seating(seating) = &c.shape {
            count += seating.seat_count()?;
            if count > crate::seating::MAX_PROJECT_SEATS {
                return Err("工程最多容纳 1024 个参数化座位".into());
            }
        }
    }
    let mut enclosed = BTreeSet::new();
    for construction in &stage.constructions {
        match &construction.shape {
            ConstructionShape::Seating(seating) => {
                member(seating.space_id.as_deref(), spaces)?;
                seating
                    .layout()
                    .map_err(|e| format!("座区“{}”：{e}", construction.name))?;
            }
            ConstructionShape::Rig(rig) => {
                member(rig.space_id.as_deref(), spaces)?;
                rig.validate()?;
            }
            ConstructionShape::Enclosure {
                space_id,
                wall_thickness_meters,
                floor_thickness_meters,
                ceiling_thickness_meters,
            } => {
                member(Some(space_id), spaces)?;
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
                member(space_id.as_deref(), spaces)?;
                outline(outline_meters).map_err(|e| format!("构件“{}”：{e}", construction.name))?;
                decimal(base_elevation_meters, -10_000.0, 10_000.0)?;
                decimal(height_meters, 0.001, 1_000.0)?;
            }
        }
    }
    Ok(())
}
