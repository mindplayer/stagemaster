use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};
use stagemaster_project::Document;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Record {
    pub format: String,
    pub version: u32,
    pub session_id: String,
    pub checkpoint_id: String,
    pub project_id: String,
    pub project_name: String,
    pub captured_at_ms: u64,
    pub source_file: Option<String>,
    pub checksum: String,
    pub document: Box<RawValue>,
}
impl Record {
    pub fn new(
        id: &str,
        document: &Document,
        source_file: Option<String>,
        now: u64,
    ) -> Result<Self, String> {
        let encoded = String::from_utf8(document.encode()?).map_err(|_| "工程编码无效")?;
        let document_json = RawValue::from_string(encoded).map_err(|_| "工程恢复编码失败")?;
        let project = document.view();
        Ok(Self {
            format: "stagemaster.recovery".into(),
            version: 1,
            session_id: id.into(),
            checkpoint_id: uuid::Uuid::new_v4().to_string(),
            project_id: project.id,
            project_name: project.name,
            captured_at_ms: now,
            source_file,
            checksum: digest(document_json.get().as_bytes()),
            document: document_json,
        })
    }
    pub fn decode(bytes: &[u8], id: &str) -> Result<Self, String> {
        let record: Self = serde_json::from_slice(bytes).map_err(|_| "恢复文件损坏或格式不支持")?;
        if record.format != "stagemaster.recovery" || record.version != 1 {
            return Err("恢复文件版本不支持，请保留文件并使用兼容版本".into());
        }
        if record.session_id != id
            || uuid::Uuid::parse_str(&record.checkpoint_id).is_err()
            || record.captured_at_ms > 9_007_199_254_740_991
            || digest(record.document.get().as_bytes()) != record.checksum
        {
            return Err("恢复文件身份、时间或内容校验失败".into());
        }
        Ok(record)
    }
    pub fn document(&self) -> Result<Document, String> {
        let document = Document::decode(self.document.get().as_bytes())?;
        let view = document.view();
        if self.project_id != view.id || self.project_name != view.name {
            return Err("恢复文件摘要与工程内容不一致".into());
        }
        Ok(document)
    }
}
pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_valid_checksum_does_not_hide_duplicate_keys_or_invalid_domain_data() {
        let id = uuid::Uuid::new_v4().to_string();
        let mut record = Record::new(&id, &Document::new("严格校验").unwrap(), None, 1).unwrap();
        let original = record.document.get().to_owned();
        let duplicate = original.replacen('{', "{\"format\":\"duplicated\",", 1);
        record.document = RawValue::from_string(duplicate).unwrap();
        record.checksum = digest(record.document.get().as_bytes());
        let bytes = serde_json::to_vec(&record).unwrap();
        let decoded = Record::decode(&bytes, &id).unwrap();
        assert!(decoded.document().is_err());
        record.document = RawValue::from_string(original).unwrap();
        record.checksum = digest(record.document.get().as_bytes());
        record.project_name = "伪造摘要".into();
        assert!(record.document().is_err());
    }
}
