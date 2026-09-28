use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use stagemaster_device_auth::{
    Address, Binding, LocalIdentity, Secret, persistence::STORAGE_BYTES,
};
use std::{
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::{Context, Poll, Waker},
};

pub fn run<T>(f: impl Future<Output = T>) -> T {
    let mut f = std::pin::pin!(f);
    match f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("single-owner synchronous NOR operation unexpectedly yielded"),
    }
}
pub fn local() -> LocalIdentity {
    LocalIdentity::new(
        Address::new(true, [9, 8, 7, 6, 5, 0xc4]).unwrap(),
        Secret::new([99; 16]).unwrap(),
    )
    .unwrap()
}
pub fn binding(n: u8) -> Binding {
    Binding::new(
        [n; 16],
        Address::new(false, [n; 6]).unwrap(),
        Secret::new([n + 20; 16]).unwrap(),
        None,
    )
    .unwrap()
}
#[derive(Clone, Copy, Debug)]
pub struct Fault;
impl NorFlashError for Fault {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Failure {
    Before,
    Partial,
    After,
}
pub struct State {
    pub bytes: Vec<u8>,
    pub writes: usize,
    pub ops: usize,
    pub fail: Option<(usize, Failure)>,
}
#[derive(Clone)]
pub struct Flash(pub Rc<RefCell<State>>);
impl Flash {
    pub fn new() -> Self {
        Self::from(vec![0xff; STORAGE_BYTES])
    }
    pub fn from(bytes: Vec<u8>) -> Self {
        Self(Rc::new(RefCell::new(State {
            bytes,
            writes: 0,
            ops: 0,
            fail: None,
        })))
    }
    fn change(&self, start: usize, data: &[u8], erase: bool) -> Result<(), Fault> {
        let mut s = self.0.borrow_mut();
        s.ops += 1;
        let fail = s.fail.filter(|(at, _)| *at == s.ops).map(|(_, f)| f);
        if matches!(fail, Some(Failure::Before)) {
            return Err(Fault);
        }
        let len = if matches!(fail, Some(Failure::Partial)) {
            data.len() / 2
        } else {
            data.len()
        };
        s.writes += 1;
        let target = &mut s.bytes[start..start + len];
        for (dst, src) in target.iter_mut().zip(data) {
            if !erase {
                assert_eq!(*dst & *src, *src, "NOR zero-to-one write");
            }
            *dst = *src;
        }
        if fail.is_some() { Err(Fault) } else { Ok(()) }
    }
}
impl ErrorType for Flash {
    type Error = Fault;
}
impl ReadNorFlash for Flash {
    const READ_SIZE: usize = 4;
    fn capacity(&self) -> usize {
        self.0.borrow().bytes.len()
    }
    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Fault> {
        assert!(offset.is_multiple_of(4) && bytes.len().is_multiple_of(4));
        let mut s = self.0.borrow_mut();
        s.ops += 1;
        if s.fail.is_some_and(|(n, _)| n == s.ops) {
            return Err(Fault);
        }
        bytes.copy_from_slice(&s.bytes[offset as usize..][..bytes.len()]);
        Ok(())
    }
}
impl NorFlash for Flash {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 4096;
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Fault> {
        assert!(offset.is_multiple_of(4) && bytes.len().is_multiple_of(4));
        self.change(offset as usize, bytes, false)
    }
    fn erase(&mut self, start: u32, end: u32) -> Result<(), Fault> {
        assert!(start.is_multiple_of(4096) && end.is_multiple_of(4096));
        self.change(start as usize, &vec![0xff; (end - start) as usize], true)
    }
}
