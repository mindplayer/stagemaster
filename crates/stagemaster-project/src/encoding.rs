//! One capacity policy for editing, saved files and recovery snapshots.
use crate::{MAX_BYTES, array};
use serde_json::Value;
use std::io::{self, Write};

const CAPACITY_ERROR: &str = "工程内容和必要修订信息超过 8 MiB 限制";

pub(super) fn validate_capacity(root: &Value) -> Result<(), String> {
    // UUIDs in the validated schema are always 36 bytes. A future save replaces
    // the revision UUID and changes [] to ["<uuid>"] at most; reserve that growth
    // even for a new document so its second save remains possible as well.
    let revision_growth = if array(&root["project"], "parentRevisionIds").is_empty() {
        38
    } else {
        0
    };
    let mut counter = Bounded::new(io::sink(), MAX_BYTES - revision_growth);
    serde_json::to_writer(&mut counter, root).map_err(|_| counter.error().into())
}

pub(super) fn encode(root: &Value) -> Result<Vec<u8>, String> {
    validate_capacity(root)?;
    let mut bytes = Vec::new();
    let mut pretty = Bounded::new(&mut bytes, MAX_BYTES - 1);
    match serde_json::to_writer_pretty(&mut pretty, root) {
        Ok(()) => {
            bytes.push(b'\n');
            return Ok(bytes);
        }
        Err(_) if pretty.exceeded => {}
        Err(_) => return Err(pretty.error().into()),
    }
    // Keep the same bounded buffer instead of allocating another whole document.
    bytes.clear();
    let mut compact = Bounded::new(&mut bytes, MAX_BYTES);
    serde_json::to_writer(&mut compact, root).map_err(|_| compact.error())?;
    if bytes.len() < MAX_BYTES {
        bytes.push(b'\n');
    }
    Ok(bytes)
}

struct Bounded<W> {
    inner: W,
    remaining: usize,
    exceeded: bool,
}
impl<W> Bounded<W> {
    fn new(inner: W, limit: usize) -> Self {
        Self {
            inner,
            remaining: limit,
            exceeded: false,
        }
    }
    fn error(&self) -> &'static str {
        if self.exceeded {
            CAPACITY_ERROR
        } else {
            "工程编码失败"
        }
    }
}
impl<W: Write> Write for Bounded<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.remaining {
            self.exceeded = true;
            return Err(io::Error::other(CAPACITY_ERROR));
        }
        let written = self.inner.write(bytes)?;
        self.remaining -= written;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
