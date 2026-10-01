//! Read-only crew handoff, never a replacement for the project or an output plan.
mod csv_format;
mod rows;
use crate::Document;
use crate::report_csv::BoundedCsv;
use csv_format::HEADERS;
use sha2::{Digest, Sha256};

pub struct PatchReport {
    bytes: Vec<u8>,
    fixture_count: usize,
}
impl PatchReport {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub fn fixture_count(&self) -> usize {
        self.fixture_count
    }
    /// Recognize only this report version before replacing an existing export.
    #[must_use]
    pub fn recognizes(bytes: &[u8]) -> bool {
        csv_format::recognizes(bytes)
    }
}
impl Document {
    /// Export every fixture from this exact snapshot, including unpatched/unplaced fixtures.
    /// # Errors
    /// Returns an error if the source cannot encode or the CSV exceeds its bounded byte budget.
    pub fn patch_report(&self) -> Result<PatchReport, String> {
        let source_digest = format!("{:x}", Sha256::digest(self.encode()?));
        let project = self.view();
        let rows = rows::ReportRows::new(
            &project,
            crate::text(&self.root["project"], "revisionId"),
            &source_digest,
        );
        let mut fixtures: Vec<_> = project.fixtures.iter().collect();
        fixtures.sort_by_key(|f| {
            (
                &f.domain_name,
                &f.domain_id,
                f.universe.unwrap_or(u64::MAX),
                f.address.unwrap_or(u64::MAX),
                &f.id,
            )
        });
        let mut csv = csv::WriterBuilder::new()
            .terminator(csv::Terminator::CRLF)
            .from_writer(BoundedCsv(b"\xef\xbb\xbf".to_vec()));
        csv.write_record(HEADERS).map_err(|e| e.to_string())?;
        for f in fixtures {
            let row = rows.fixture(f);
            csv.write_record(row).map_err(|e| e.to_string())?;
        }
        let bytes = csv.into_inner().map_err(|e| e.to_string())?.0;
        Ok(PatchReport {
            bytes,
            fixture_count: project.fixtures.len(),
        })
    }
}
