//! Only records read requests; it does not measure allocator peaks or hardware timing.
use stagemaster_package::{Error, MAX_PROGRAM_BYTES, ReadAt};
use std::cell::RefCell;
pub struct Reader<'a> {
    pub bytes: &'a [u8],
    pub reads: RefCell<Vec<(usize, usize)>>,
    pub fail_at: Option<usize>,
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            reads: RefCell::new(vec![]),
            fail_at: None,
        }
    }
    pub fn assert_complete_hash(&self) {
        let reads = self.reads.borrow();
        assert_eq!(reads[0], (0, 64));
        let mut cursor = 64;
        for &(offset, length) in &reads[1..=self.bytes.len().saturating_sub(64).div_ceil(1024)] {
            assert_eq!(offset, cursor);
            assert_eq!(length, 1024.min(self.bytes.len() - cursor));
            cursor += length;
        }
        assert_eq!(cursor, self.bytes.len());
    }
}
impl ReadAt for Reader<'_> {
    fn len(&self) -> usize {
        self.bytes.len()
    }
    fn read_exact(&self, offset: usize, target: &mut [u8]) -> Result<(), Error> {
        assert!(target.len() <= MAX_PROGRAM_BYTES, "unbounded source read");
        self.reads.borrow_mut().push((offset, target.len()));
        if self.fail_at == Some(offset) {
            return Err(Error::Read);
        }
        self.bytes.read_exact(offset, target)
    }
}
