//! Sequential staging. Erase each payload sector immediately before its first program.
use crate::{Buffer, Code, Error, IO_BYTES, MAX_WORD_BYTES, Shared, io};
use embedded_storage::nor_flash::NorFlash;
use stagemaster_install::Slot;

pub(super) struct Stage {
    pub slot: Slot,
    pub bytes: usize,
    pub received: usize,
    pub programmed: usize,
    pub tail: [u8; MAX_WORD_BYTES],
    pub tail_len: usize,
    pub poisoned: bool,
    pub sealed: bool,
    pub commit_attempted: bool,
    erased: usize,
}
impl Stage {
    pub fn new(slot: Slot, bytes: usize) -> Self {
        Self {
            slot,
            bytes,
            received: 0,
            programmed: 0,
            tail: [0xff; MAX_WORD_BYTES],
            tail_len: 0,
            poisoned: true,
            sealed: false,
            commit_attempted: false,
            erased: 0,
        }
    }

    pub fn write<F: NorFlash>(
        &mut self,
        shared: &Shared<F>,
        offset: usize,
        bytes: &[u8],
    ) -> Result<(), Error<F::Error>> {
        if bytes.is_empty()
            || bytes.len() > IO_BYTES
            || offset != self.received
            || offset
                .checked_add(bytes.len())
                .is_none_or(|n| n > self.bytes)
        {
            return Err(Code::Bounds.into());
        }
        self.poisoned = true;
        let mut buffer = Buffer::erased();
        let mut input = bytes;
        while !input.is_empty() {
            let carry = self.tail_len;
            let count = input.len().min(IO_BYTES - carry);
            buffer.0[..carry].copy_from_slice(&self.tail[..carry]);
            buffer.0[carry..carry + count].copy_from_slice(&input[..count]);
            let length = carry + count;
            let complete = length / F::WRITE_SIZE * F::WRITE_SIZE;
            if complete != 0 {
                self.program(shared, &buffer.0[..complete])?;
            }
            self.tail_len = length - complete;
            self.tail[..self.tail_len].copy_from_slice(&buffer.0[complete..length]);
            input = &input[count..];
        }
        self.received += bytes.len();
        self.poisoned = false;
        Ok(())
    }

    pub fn sync<F: NorFlash>(&mut self, shared: &Shared<F>) -> Result<(), Error<F::Error>> {
        if self.sealed {
            return Ok(());
        }
        self.poisoned = true;
        if self.tail_len != 0 {
            let mut buffer = Buffer::erased();
            buffer.0[..self.tail_len].copy_from_slice(&self.tail[..self.tail_len]);
            self.program(shared, &buffer.0[..F::WRITE_SIZE])?;
            self.tail_len = 0;
        }
        self.sealed = true;
        self.poisoned = false;
        Ok(())
    }

    fn program<F: NorFlash>(
        &mut self,
        shared: &Shared<F>,
        bytes: &[u8],
    ) -> Result<(), Error<F::Error>> {
        let end = self
            .programmed
            .checked_add(bytes.len())
            .ok_or(Code::Bounds)?;
        let required = end.div_ceil(F::ERASE_SIZE) * F::ERASE_SIZE;
        if required > shared.layout.slot_bytes() {
            return Err(Code::Bounds.into());
        }
        let base = shared.layout.payload_offset(self.slot);
        // Count progress only after physical erasure AND readback succeed. Any failure
        // leaves this stage poisoned; cancellation/new preparation is mandatory.
        while self.erased < required {
            io::erase(
                shared,
                base + self.erased,
                base + self.erased + F::ERASE_SIZE,
            )?;
            self.erased += F::ERASE_SIZE;
        }
        io::write(shared, base + self.programmed, bytes)?;
        self.programmed = end;
        Ok(())
    }
}
