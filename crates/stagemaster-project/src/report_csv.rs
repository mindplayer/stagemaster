//! Shared bounded CSV transport for human handoff documents.
use std::io::{self, Write};

pub(crate) const LIMIT: usize = crate::MAX_BYTES;

/// Untrusted text remains literal in spreadsheet applications. Numbers use a separate path.
pub(crate) fn literal(text: &str) -> String {
    let start = text.trim_start_matches(|c: char| c.is_whitespace() || c.is_control());
    if start.starts_with(['=', '+', '-', '@']) || text.starts_with(['\t', '\r', '\n']) {
        format!("'{text}")
    } else {
        text.into()
    }
}

pub(crate) struct BoundedCsv(pub Vec<u8>);
impl Write for BoundedCsv {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > LIMIT.saturating_sub(self.0.len()) {
            return Err(io::Error::other(
                "交接资料超过 8 MiB，请缩短工程中的文字信息",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn recognizes(bytes: &[u8], headers: &[&str], format: &str) -> bool {
    if bytes.len() > LIMIT || !bytes.starts_with(b"\xef\xbb\xbf") {
        return false;
    }
    let mut csv = csv::Reader::from_reader(bytes);
    if !csv
        .headers()
        .is_ok_and(|h| h.iter().eq(headers.iter().copied()))
    {
        return false;
    }
    csv.records()
        .all(|row| row.is_ok_and(|r| r.len() == headers.len() && r.get(0) == Some(format)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_overflow_without_partial_append_and_keeps_text_literal() {
        let mut buffer = BoundedCsv(vec![0; LIMIT - 2]);
        assert!(buffer.write_all(b"abc").is_err());
        assert_eq!(buffer.0.len(), LIMIT - 2);
        buffer.write_all(b"ab").unwrap();
        assert_eq!(buffer.0.len(), LIMIT);
        for text in ["=1+1", "  +1", "\t@SUM(A1)", "\nplain", "-灯具", "\rtext"] {
            assert_eq!(literal(text), format!("'{text}"));
        }
        assert_eq!(literal("前区\n灯具,\"甲\""), "前区\n灯具,\"甲\"");
    }
}
