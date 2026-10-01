//! Read-only show-call sheet, preserving authored order and operator waits.
mod rows;
use crate::{
    Document,
    report_csv::{BoundedCsv, recognizes},
};
use sha2::{Digest, Sha256};

pub struct SequenceReport {
    bytes: Vec<u8>,
    sequence_id: String,
    sequence_name: String,
    step_count: usize,
}
impl SequenceReport {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub fn sequence_id(&self) -> &str {
        &self.sequence_id
    }
    #[must_use]
    pub fn sequence_name(&self) -> &str {
        &self.sequence_name
    }
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.step_count
    }
    #[must_use]
    pub fn recognizes(bytes: &[u8]) -> bool {
        recognizes(bytes, &rows::HEADERS, rows::FORMAT)
    }
}
impl Document {
    /// Build a crew handoff from one exact authored list without changing playback or history.
    /// # Errors
    /// Rejects a missing list, inconsistent references, failed encoding, or the 8 MiB limit.
    pub fn sequence_report(&self, sequence_id: &str) -> Result<SequenceReport, String> {
        let project = self.view();
        let sequence = project
            .sequences
            .iter()
            .find(|s| s.id == sequence_id)
            .ok_or("场景列表已不存在，请重新选择")?;
        let digest = format!("{:x}", Sha256::digest(self.encode()?));
        let rows = rows::SequenceRows::new(
            &project,
            sequence,
            crate::text(&self.root["project"], "revisionId"),
            &digest,
        );
        let mut writer = csv::WriterBuilder::new()
            .terminator(csv::Terminator::CRLF)
            .from_writer(BoundedCsv(b"\xef\xbb\xbf".to_vec()));
        writer
            .write_record(rows::HEADERS)
            .map_err(|e| e.to_string())?;
        for (position, step) in sequence.steps.iter().enumerate() {
            writer
                .write_record(rows.step(position, step)?)
                .map_err(|e| e.to_string())?;
        }
        Ok(SequenceReport {
            bytes: writer.into_inner().map_err(|e| e.to_string())?.0,
            sequence_id: sequence.id.clone(),
            sequence_name: sequence.name.clone(),
            step_count: sequence.steps.len(),
        })
    }
}
