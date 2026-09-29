use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use stagemaster_nor_store::{CacheError, CachedNor};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Fault;
impl NorFlashError for Fault {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}
struct State {
    bytes: Vec<u8>,
    reads: usize,
    fail: bool,
}
struct Flash(Rc<RefCell<State>>);
impl ErrorType for Flash {
    type Error = Fault;
}
impl ReadNorFlash for Flash {
    const READ_SIZE: usize = 4;
    fn capacity(&self) -> usize {
        self.0.borrow().bytes.len()
    }
    fn read(&mut self, offset: u32, out: &mut [u8]) -> Result<(), Fault> {
        let mut state = self.0.borrow_mut();
        state.reads += 1;
        if state.fail {
            out.fill(0xab);
            return Err(Fault);
        }
        out.copy_from_slice(&state.bytes[offset as usize..offset as usize + out.len()]);
        Ok(())
    }
}
impl NorFlash for Flash {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 1024;
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Fault> {
        let mut state = self.0.borrow_mut();
        state.bytes[offset as usize..offset as usize + 4].copy_from_slice(&bytes[..4]);
        if state.fail {
            return Err(Fault);
        }
        state.bytes[offset as usize..offset as usize + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
    fn erase(&mut self, from: u32, to: u32) -> Result<(), Fault> {
        let mut state = self.0.borrow_mut();
        state.bytes[from as usize..to as usize].fill(0xff);
        if state.fail {
            return Err(Fault);
        }
        Ok(())
    }
}
fn setup() -> (Flash, Rc<RefCell<State>>) {
    let state = Rc::new(RefCell::new(State {
        bytes: vec![0x42; 4096],
        reads: 0,
        fail: false,
    }));
    (Flash(state.clone()), state)
}

#[test]
fn cached_reads_cross_blocks_and_collisions_without_allocating() {
    let (flash, state) = setup();
    let mut memory = [0; 2056];
    let mut cache = CachedNor::new(flash, &mut memory).unwrap();
    let mut out = [0; 1032];
    cache.read(1016, &mut out).unwrap();
    assert_eq!(out, [0x42; 1032]);
    assert_eq!(state.borrow().reads, 2);
    cache.read(1016, &mut out).unwrap();
    assert_eq!(state.borrow().reads, 2);
    cache.read(2048, &mut out[..4]).unwrap();
    cache.read(0, &mut out[..4]).unwrap();
    assert_eq!(state.borrow().reads, 4);
    assert_eq!(cache.stats().hits, 2);
}

#[test]
fn mutations_and_failed_partial_writes_cannot_return_stale_cache() {
    let (flash, state) = setup();
    let mut memory = [0; 2056];
    let mut cache = CachedNor::new(flash, &mut memory).unwrap();
    let mut out = [0; 8];
    cache.read(0, &mut out).unwrap();
    cache.write(0, &[1; 8]).unwrap();
    cache.read(0, &mut out).unwrap();
    assert_eq!(out, [1; 8]);
    assert_eq!(state.borrow().reads, 2);
    state.borrow_mut().fail = true;
    assert_eq!(cache.write(0, &[2; 8]), Err(CacheError::Device(Fault)));
    state.borrow_mut().fail = false;
    cache.read(0, &mut out).unwrap();
    assert_eq!(out, [2, 2, 2, 2, 1, 1, 1, 1]);
    state.borrow_mut().fail = true;
    assert!(cache.erase(0, 1024).is_err());
    state.borrow_mut().fail = false;
    cache.read(0, &mut out).unwrap();
    assert_eq!(out, [0xff; 8]);
    assert_eq!(state.borrow().reads, 4);
}

#[test]
fn failed_reads_are_never_published_and_invalid_ranges_never_reach_flash() {
    let (flash, state) = setup();
    let mut memory = [0; 1028];
    let mut cache = CachedNor::new(flash, &mut memory).unwrap();
    let mut out = [0; 8];
    state.borrow_mut().fail = true;
    assert!(cache.read(0, &mut out).is_err());
    state.borrow_mut().fail = false;
    cache.read(0, &mut out).unwrap();
    assert_eq!(out, [0x42; 8]);
    for offset in [1, 4092, u32::MAX] {
        assert_eq!(cache.read(offset, &mut out), Err(CacheError::Bounds));
    }
    assert_eq!(cache.erase(1024, 0), Err(CacheError::Bounds));
    assert_eq!(cache.write(4092, &out), Err(CacheError::Bounds));
    assert_eq!(state.borrow().reads, 2);
}

#[test]
fn no_cache_fallback_and_partial_last_block_preserve_reads() {
    let (flash, state) = setup();
    let mut cache = CachedNor::new(flash, &mut []).unwrap();
    let mut out = [0; 4];
    cache.read(0, &mut out).unwrap();
    cache.read(0, &mut out).unwrap();
    assert_eq!(state.borrow().reads, 2);
    let (flash, state) = setup();
    state.borrow_mut().bytes.truncate(1032);
    let mut memory = [0; 1028];
    let mut cache = CachedNor::new(flash, &mut memory).unwrap();
    cache.read(1028, &mut out).unwrap();
    assert_eq!(out, [0x42; 4]);
    cache.read(1028, &mut out).unwrap();
    assert_eq!(state.borrow().reads, 1);
}
