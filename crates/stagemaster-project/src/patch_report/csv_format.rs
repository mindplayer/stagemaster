use std::io::{self, Write};

pub(super) const FORMAT: &str = "StageMaster 配灯表/1";
pub(super) const HEADERS: [&str; 28] = [
    "资料格式",
    "工程名称",
    "工程标识",
    "来源保存修订",
    "工程快照 SHA256",
    "灯具标识",
    "灯具名称",
    "厂家",
    "型号",
    "模式",
    "档案修订",
    "输出域",
    "输出域标识",
    "线路",
    "起始地址",
    "结束地址",
    "通道数",
    "配适状态",
    "布置状态",
    "所属空间",
    "支撑体",
    "世界 X（米）",
    "世界 Y（米）",
    "世界 Z（米）",
    "安装旋转 X（度）",
    "安装旋转 Y（度）",
    "安装旋转 Z（度）",
    "灯组",
];
pub(super) const LIMIT: usize = crate::MAX_BYTES;

/// Untrusted text remains literal in spreadsheet applications. Numbers use a separate path.
pub(super) fn literal(text: &str) -> String {
    let start = text.trim_start_matches(|c: char| c.is_whitespace() || c.is_control());
    if start.starts_with(['=', '+', '-', '@']) || text.starts_with(['\t', '\r', '\n']) {
        format!("'{text}")
    } else {
        text.into()
    }
}

pub(super) struct BoundedCsv(pub Vec<u8>);
impl Write for BoundedCsv {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > LIMIT.saturating_sub(self.0.len()) {
            return Err(io::Error::other("配灯表超过 8 MiB，请缩短工程中的文字信息"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn recognizes(bytes: &[u8]) -> bool {
    if bytes.len() > LIMIT || !bytes.starts_with(b"\xef\xbb\xbf") {
        return false;
    }
    let mut csv = csv::Reader::from_reader(bytes);
    if !csv.headers().is_ok_and(|h| h.iter().eq(HEADERS)) {
        return false;
    }
    csv.records()
        .all(|row| row.is_ok_and(|r| r.len() == HEADERS.len() && r.get(0) == Some(FORMAT)))
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
