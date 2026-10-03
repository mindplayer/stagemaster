//! Format outside the transport lock; each transport call has a bounded byte count.
use core::fmt::{self, Write};

const CHUNK_BYTES: usize = 32;

struct Printer<F>(F);

impl<F: FnMut(&[u8])> Write for Printer<F> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for chunk in text.as_bytes().chunks(CHUNK_BYTES) {
            (self.0)(chunk);
        }
        Ok(())
    }
}

pub fn write_line(args: fmt::Arguments<'_>, sink: impl FnMut(&[u8])) -> fmt::Result {
    let mut printer = Printer(sink);
    printer.write_fmt(args)?;
    printer.write_char('\n')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_writes_preserve_long_unicode_and_numeric_reports() {
        let text = "运行设备的状态、帧数据与中文提示".repeat(40);
        let mut bytes = Vec::new();
        write_line(
            format_args!("{text} {:08x} {}", 0x123_u32, u64::MAX),
            |chunk| {
                assert!(chunk.len() <= CHUNK_BYTES);
                bytes.extend_from_slice(chunk);
            },
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            format!("{text} 00000123 {}\n", u64::MAX)
        );
    }
}
