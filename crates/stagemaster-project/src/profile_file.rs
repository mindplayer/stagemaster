//! Portable authorable modes. Importing never carries fixture identities or patches.
use crate::{Document, EditCommand, FixtureEdit, ProfileDefinition, array, text};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_PROFILE_FILE_BYTES: usize = 512 * 1024;
const FORMAT: &str = "stagemaster-fixture-profile";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileSource {
    pub profile_id: String,
    pub revision: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    format: String,
    format_version: u16,
    source: ProfileSource,
    definition: ProfileDefinition,
}

pub struct ProfileFile(Envelope);
impl ProfileFile {
    /// Inspect a bounded, supported file without modifying any active project.
    /// # Errors
    /// Rejects unknown formats/fields, invalid source identities and unsupported definitions.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_PROFILE_FILE_BYTES {
            return Err("灯具模式文件超过 512 KiB 限制".into());
        }
        let file: Envelope = serde_json::from_slice(bytes).map_err(|e| {
            format!(
                "灯具模式字段或 JSON 格式无效（第 {} 行，第 {} 列）",
                e.line(),
                e.column()
            )
        })?;
        if file.format != FORMAT || file.format_version != 1 {
            return Err("不支持此灯具模式文件或版本，请使用兼容版本导出".into());
        }
        for id in [&file.source.profile_id, &file.source.revision] {
            uuid::Uuid::parse_str(id).map_err(|_| "灯具模式来源标识无效")?;
        }
        validated_profile(&file.definition)?;
        Ok(Self(file))
    }
    /// Encode this complete definition, including provenance but no source project path.
    /// # Errors
    /// Rejects encoding failures and an oversized formatted file.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let bytes = serde_json::to_vec_pretty(&self.0).map_err(|_| "灯具模式编码失败")?;
        if bytes.len() > MAX_PROFILE_FILE_BYTES {
            return Err("灯具模式文件超过 512 KiB 限制，请缩短文字信息".into());
        }
        Ok(bytes)
    }
    #[must_use]
    pub fn definition(&self) -> &ProfileDefinition {
        &self.0.definition
    }
    #[must_use]
    pub fn source(&self) -> &ProfileSource {
        &self.0.source
    }
}

impl Document {
    /// Capture one losslessly representable mode; export never changes its revision.
    /// # Errors
    /// Rejects missing modes or data the current authoring contract cannot preserve.
    pub fn profile_file(&self, profile_id: &str) -> Result<ProfileFile, String> {
        let original = array(&self.root["lighting"], "profiles")
            .iter()
            .find(|p| text(p, "id") == profile_id)
            .ok_or("灯具模式已不存在，请重新选择")?;
        let p = crate::fixture_view::profile(original);
        if !p.authorable {
            return Err("此模式包含当前编辑器尚未支持的定义，暂不能导出".into());
        }
        let definition = ProfileDefinition {
            name: p.name,
            manufacturer: p.manufacturer,
            model: p.model,
            mode: p.mode,
            positioning: p.positioning,
            footprint: u16::try_from(p.footprint).map_err(|_| "模式通道数无效")?,
            channels: p.channels,
        };
        let rebuilt = validated_profile(&definition)?;
        if without_identity(original.clone()) != without_identity(rebuilt) {
            return Err(
                "此模式含当前文件格式无法完整保留的数据，未导出；请复制并检查新模式".into(),
            );
        }
        let file = ProfileFile(Envelope {
            format: FORMAT.into(),
            format_version: 1,
            source: ProfileSource {
                profile_id: p.id,
                revision: p.revision,
            },
            definition,
        });
        file.encode()?;
        Ok(file)
    }
}

fn validated_profile(definition: &ProfileDefinition) -> Result<Value, String> {
    // Reuse the exact authoring transaction, including schema, functions and mechanical limits.
    let mut scratch = Document::new("灯具模式检查")?;
    scratch.edit(EditCommand::Fixture {
        command: FixtureEdit::SaveProfile {
            id: None,
            definition: Box::new(definition.clone()),
        },
    })?;
    Ok(array(&scratch.root["lighting"], "profiles")
        .last()
        .expect("added mode")
        .clone())
}
fn without_identity(mut profile: Value) -> Value {
    let object = profile.as_object_mut().expect("validated profile");
    object.remove("id");
    object.remove("revision");
    profile
}
