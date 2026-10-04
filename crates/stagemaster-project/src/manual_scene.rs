//! Sparse recording of confirmed manual values. No UI, runtime transport or clock.
use crate::{Document, FunctionTable, ProfileDefault, array, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualSceneReading {
    pub fixture_id: String,
    pub attribute: String,
    pub value: u16,
}

#[derive(Clone)]
pub struct ManualSceneCapture {
    project_id: String,
    fixtures: Vec<(String, [u8; 32])>,
    readings: Vec<ManualSceneReading>,
    assignments: Vec<Value>,
}
impl ManualSceneCapture {
    pub(crate) fn assignments(&self) -> &[Value] {
        &self.assignments
    }
    #[must_use]
    pub fn readings(&self) -> &[ManualSceneReading] {
        &self.readings
    }
    /// Check only the recorded fixtures; unrelated scene edits do not invalidate compatibility.
    /// # Errors
    /// Reject a different project or changed fixture semantics, calibration or patch.
    pub fn check(&self, document: &Document) -> Result<(), String> {
        if text(&document.root["project"], "id") != self.project_id {
            return Err("手动记录属于另一工程".into());
        }
        for (id, expected) in &self.fixtures {
            if signature(&document.root, id)? != *expected {
                return Err("录入灯具的定义、校准或配适与后台不一致，请重新载入后台后采集".into());
            }
        }
        Ok(())
    }
}
impl Document {
    /// Capture a bounded set from the exact immutable document used by an execution layout.
    /// The caller supplies authoritative pre-level held values, never final mixed output.
    /// # Errors
    /// Reject wrong source identity, invalid/duplicate/empty readings and invalid function values.
    pub fn capture_manual_scene(
        &self,
        layout: &str,
        readings: Vec<ManualSceneReading>,
    ) -> Result<ManualSceneCapture, String> {
        if format!("{:x}", Sha256::digest(self.encode()?)) != layout {
            return Err("后台固定工程与执行布局身份不一致".into());
        }
        if readings.is_empty() || readings.len() > 512 {
            return Err("请选择具有已持有属性的灯具，最多录入 512 项属性".into());
        }
        let mut seen = HashSet::new();
        let mut fixtures = Vec::new();
        let mut assignments = Vec::new();
        for r in &readings {
            if !seen.insert((&r.fixture_id, &r.attribute)) {
                return Err("手动记录包含重复属性".into());
            }
            if !fixtures.iter().any(|(id, _)| id == &r.fixture_id) {
                fixtures.push((r.fixture_id.clone(), signature(&self.root, &r.fixture_id)?));
            }
            let profile = crate::fixture_value::profile(&self.root["lighting"], &r.fixture_id)?;
            let channel = array(profile, "channels")
                .iter()
                .find(|c| c["attribute"] == r.attribute)
                .ok_or("录入属性没有通道定义")?;
            let value = if let Some(functions) = crate::fixture_value::functions(channel)? {
                ProfileDefault::Function(
                    FunctionTable::new(&functions, channel["encoding"] == "u16-be")?
                        .decode(r.value)?,
                )
            } else {
                ProfileDefault::Normalized(r.value)
            };
            let value = crate::fixture_value::stored_default(&value);
            if crate::fixture_value::encode(profile, &r.attribute, &value)? != r.value {
                return Err("录入属性不能精确重放".into());
            }
            assignments.push(
                json!({"target":{"fixtureId":r.fixture_id,"attribute":r.attribute},
                "operation":"set","source":{"kind":"literal","value":value}}),
            );
        }
        Ok(ManualSceneCapture {
            project_id: text(&self.root["project"], "id").into(),
            fixtures,
            readings,
            assignments,
        })
    }

    /// Create one sparse scene atomically, retaining every recorded zero and function selection.
    /// # Errors
    /// Reject changed context, invalid names and project capacity without changing the document.
    pub fn record_manual_scene(
        &mut self,
        capture: &ManualSceneCapture,
        name: &str,
    ) -> Result<String, String> {
        capture.check(self)?;
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 256 || name.chars().any(char::is_control) {
            return Err("场景名称需要 1–256 个有效字符".into());
        }
        if array(&self.root["lighting"], "scenes")
            .iter()
            .any(|s| text(s, "name") == name)
        {
            return Err("已有同名场景，请换一个名称".into());
        }
        let id = crate::id();
        let mut next = self.root.clone();
        crate::editing::list(&mut next, "scenes")?.push(json!({
            "id":id, "name":name, "assignments":capture.assignments
        }));
        crate::validation::validate(&next)?;
        crate::encoding::validate_capacity(&next)?;
        self.root = next;
        Ok(id)
    }
}
fn signature(root: &Value, id: &str) -> Result<[u8; 32], String> {
    let lighting = &root["lighting"];
    let mut fixture = array(lighting, "fixtures")
        .iter()
        .find(|f| f["id"] == id)
        .cloned()
        .ok_or("录入灯具已不存在")?;
    fixture
        .as_object_mut()
        .ok_or("灯具定义无效")?
        .remove("name");
    let mut profile = crate::fixture_value::profile(lighting, id)?.clone();
    for field in ["name", "revision", "manufacturer", "model", "mode"] {
        profile.as_object_mut().ok_or("灯具档案无效")?.remove(field);
    }
    let patches: Vec<_> = array(lighting, "patches")
        .iter()
        .filter(|p| p["fixtureId"] == id)
        .collect();
    Ok(Sha256::digest(
        serde_json::to_vec(&json!({"fixture":fixture,"profile":profile,"patches":patches}))
            .map_err(|e| e.to_string())?,
    )
    .into())
}
