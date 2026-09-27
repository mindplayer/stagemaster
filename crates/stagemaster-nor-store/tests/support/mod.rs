use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use stagemaster_install::{Commit, Identity, Installer, Phase};
use stagemaster_nor_store::{Layout, NorDevice, NorStore};
use stagemaster_package::Archive;
use stagemaster_project::{Document, PackageSelection};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Injected,
}
impl NorFlashError for Failure {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Read,
    Write,
    Erase,
}
#[derive(Clone, Copy, Debug)]
pub struct Operation {
    pub kind: Kind,
    pub offset: usize,
    pub length: usize,
}
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Before,
    Partial,
    After,
    Silent,
}
#[derive(Clone)]
pub struct State {
    pub bytes: Vec<u8>,
    // Track physical program history, including interrupted commands, across MCU restarts.
    programmed: Vec<bool>,
    pub operations: Vec<Operation>,
    pub fault: Option<(usize, Mode)>,
}
impl State {
    pub fn blank(length: usize) -> Self {
        Self {
            bytes: vec![0xff; length],
            programmed: vec![false; length],
            operations: vec![],
            fault: None,
        }
    }
    pub fn reset(&mut self) {
        self.operations.clear();
        self.fault = None;
    }
    pub fn mutations(&self) -> usize {
        self.operations
            .iter()
            .filter(|o| o.kind != Kind::Read)
            .count()
    }
    fn start(&mut self, kind: Kind, offset: usize, length: usize) -> Option<Mode> {
        assert!(
            offset
                .checked_add(length)
                .is_some_and(|end| end <= self.bytes.len())
        );
        self.operations.push(Operation {
            kind,
            offset,
            length,
        });
        self.fault
            .filter(|(index, _)| *index == self.operations.len())
            .map(|(_, mode)| mode)
    }
}
pub struct Model<const R: usize = 4, const W: usize = 4, const E: usize = 4096>(
    pub Rc<RefCell<State>>,
);
impl<const R: usize, const W: usize, const E: usize> ErrorType for Model<R, W, E> {
    type Error = Failure;
}
impl<const R: usize, const W: usize, const E: usize> ReadNorFlash for Model<R, W, E> {
    const READ_SIZE: usize = R;
    fn capacity(&self) -> usize {
        self.0.borrow().bytes.len()
    }
    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        assert!(offset.is_multiple_of(R) && bytes.len().is_multiple_of(R));
        assert!(
            (bytes.as_ptr() as usize).is_multiple_of(4),
            "ESP direct read alignment"
        );
        let mut state = self.0.borrow_mut();
        let fault = state.start(Kind::Read, offset, bytes.len());
        if matches!(fault, Some(Mode::Before)) {
            return Err(Failure::Injected);
        }
        let length = if matches!(fault, Some(Mode::Partial)) {
            bytes.len() / 2
        } else {
            bytes.len()
        };
        bytes[..length].copy_from_slice(&state.bytes[offset..offset + length]);
        if matches!(fault, Some(Mode::Silent)) {
            bytes[0] ^= 1;
            return Ok(());
        }
        if fault.is_some() {
            Err(Failure::Injected)
        } else {
            Ok(())
        }
    }
}
impl<const R: usize, const W: usize, const E: usize> NorFlash for Model<R, W, E> {
    const WRITE_SIZE: usize = W;
    const ERASE_SIZE: usize = E;
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        assert!(offset.is_multiple_of(W) && bytes.len().is_multiple_of(W));
        assert!(
            (bytes.as_ptr() as usize).is_multiple_of(4),
            "ESP direct write alignment"
        );
        let mut state = self.0.borrow_mut();
        let fault = state.start(Kind::Write, offset, bytes.len());
        if matches!(fault, Some(Mode::Before)) {
            return Err(Failure::Injected);
        }
        assert!(
            state.programmed[offset..offset + bytes.len()]
                .iter()
                .all(|b| !b),
            "same word programmed twice without erase"
        );
        for (i, byte) in bytes.iter().enumerate() {
            assert_eq!(state.bytes[offset + i] & byte, *byte, "NOR tried 0 -> 1");
        }
        let length = if matches!(fault, Some(Mode::Partial | Mode::Silent)) {
            bytes.len() / 2
        } else {
            bytes.len()
        };
        state.programmed[offset..offset + bytes.len()].fill(true);
        for (i, byte) in bytes[..length].iter().enumerate() {
            state.bytes[offset + i] &= byte;
        }
        if matches!(fault, Some(Mode::Before | Mode::Partial | Mode::After)) {
            Err(Failure::Injected)
        } else {
            Ok(())
        }
    }
    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let (from, to) = (from as usize, to as usize);
        assert!(from.is_multiple_of(E) && to.is_multiple_of(E) && from < to);
        let mut state = self.0.borrow_mut();
        let fault = state.start(Kind::Erase, from, to - from);
        if matches!(fault, Some(Mode::Before)) {
            return Err(Failure::Injected);
        }
        if matches!(fault, Some(Mode::Partial | Mode::Silent)) {
            // Distributed bit erasure, not an unrealistically atomic prefix of whole words.
            for (i, byte) in state.bytes[from..to].iter_mut().enumerate() {
                *byte |= 1 << (i % 8);
            }
        } else {
            state.bytes[from..to].fill(0xff);
            state.programmed[from..to].fill(false);
        }
        if matches!(fault, Some(Mode::Before | Mode::Partial | Mode::After)) {
            Err(Failure::Injected)
        } else {
            Ok(())
        }
    }
}
pub fn package(level: u16) -> Vec<u8> {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    json["lighting"]["scenes"][0]["assignments"][0]["source"]["value"]["value"] = level.into();
    let doc = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    let view = doc.view();
    let items: Vec<_> = view
        .scenes
        .iter()
        .map(|s| PackageSelection::Scene { id: s.id.clone() })
        .chain(
            view.sequences
                .iter()
                .map(|s| PackageSelection::Sequence { id: s.id.clone() }),
        )
        .collect();
    doc.build_package(&items).unwrap().bytes
}
pub type Device = NorDevice<Model>;
pub type Install = Installer<NorStore<Model>>;
pub fn device(state: &Rc<RefCell<State>>, layout: Layout) -> Device {
    NorDevice::new(Model(state.clone()), layout).unwrap()
}
pub fn open(device: &Device) -> Install {
    Installer::open(device.open_for_installation().unwrap(), [1; 16])
        .unwrap()
        .0
}
pub fn identity(bytes: &[u8]) -> Identity {
    Identity::from_archive(&Archive::open(bytes).unwrap())
}
pub fn install<F: NorFlash>(
    installer: &mut Installer<NorStore<F>>,
    bytes: &[u8],
    counter: u64,
    chunk: usize,
) -> Result<Commit, stagemaster_install::Error<stagemaster_nor_store::Error<F::Error>>> {
    let transaction = installer.transaction(counter);
    let progress = installer.begin(transaction, identity(bytes))?;
    if progress.phase == Phase::Committed {
        return Ok(progress.commit);
    }
    for (i, block) in bytes.chunks(chunk).enumerate() {
        installer.write(transaction, i * chunk, block)?;
    }
    installer.verify(transaction)?;
    installer.commit(transaction)
}
