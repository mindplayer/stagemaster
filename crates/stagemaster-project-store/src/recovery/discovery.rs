//! Stream directory names into the existing lexical first-64 selection.
use super::{MAX_RECOVERY_RECORDS, valid_id};
use std::{collections::BinaryHeap, fs, path::Path};

pub(super) struct Discovery {
    pub ids: Vec<String>,
    pub omitted: usize,
}
struct Candidates {
    largest: BinaryHeap<String>,
    count: usize,
}
impl Candidates {
    fn new() -> Self {
        Self {
            largest: BinaryHeap::with_capacity(MAX_RECOVERY_RECORDS),
            count: 0,
        }
    }
    fn push(&mut self, id: &str) -> Result<(), String> {
        self.count = self
            .count
            .checked_add(1)
            .ok_or("恢复目录项目数量超过支持范围")?;
        if self.largest.len() < MAX_RECOVERY_RECORDS {
            self.largest.push(id.to_owned());
        } else if let Some(mut largest) = self.largest.peek_mut()
            && id < largest.as_str()
        {
            id.clone_into(&mut largest);
        }
        Ok(())
    }
    fn finish(self) -> Discovery {
        Discovery {
            omitted: self.count - self.largest.len(),
            ids: self.largest.into_sorted_vec(),
        }
    }
}
pub(super) fn scan(root: &Path) -> Result<Discovery, String> {
    let mut candidates = Candidates::new();
    for entry in fs::read_dir(root).map_err(|_| "无法读取恢复目录")? {
        let entry = entry.map_err(|_| "无法读取恢复目录项目")?;
        if let Some(id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_suffix(".recovery.json"))
            && valid_id(id).is_ok()
        {
            candidates.push(id)?;
        }
    }
    Ok(candidates.finish())
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
