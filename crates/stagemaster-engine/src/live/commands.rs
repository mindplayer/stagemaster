use super::{Cell, Error, Frame, Handle, LayoutId, LiveMixer, Source, SourceState};

impl LiveMixer {
    /// Register one trusted source against the exact compiled layout.
    /// # Errors
    /// Reject duplicate/zero identities, different layouts, full budget or exhausted generation.
    pub fn open(
        &mut self,
        source: Source,
        priority: i16,
        layout: LayoutId,
    ) -> Result<Handle, Error> {
        if layout != self.layout.id {
            return Err(Error::Layout);
        }
        if source.id == [0; 16] || self.sources().any(|s| s.source.id == source.id) {
            return Err(Error::Identity);
        }
        let slot = self
            .slots
            .iter()
            .position(|s| s.state.is_none())
            .ok_or(Error::Budget)?;
        let generation = self.next_order()?;
        let handle = Handle {
            boot: self.boot,
            generation,
            slot,
        };
        self.slots[slot].state = Some(SourceState {
            handle,
            source,
            priority,
            level: u16::MAX,
            serial: 0,
        });
        Ok(handle)
    }
    /// Atomic complete contribution update. Routine sampling must not assert existing attributes.
    /// New present attributes assert automatically; None removes this source only.
    /// # Errors
    /// Reject stale handle/serial/layout, mismatched buffers, invalid assertions or exhausted order.
    pub fn publish(&mut self, handle: Handle, frame: Frame<'_>) -> Result<(), Error> {
        let mut state = self.update_command(handle, frame.serial)?;
        if frame.layout != self.layout.id {
            return Err(Error::Layout);
        }
        let count = self.layout.attributes.len();
        if frame.values.len() != count || frame.assert.len() != count {
            return Err(Error::Shape);
        }
        if frame
            .values
            .iter()
            .zip(frame.assert)
            .any(|(v, a)| *a && v.is_none())
        {
            return Err(Error::Assertion);
        }
        let needs_order = self.slots[handle.slot]
            .cells
            .iter()
            .zip(frame.values)
            .zip(frame.assert)
            .any(|((previous, next), assert)| {
                next.is_some() && (previous.value.is_none() || *assert)
            });
        let order = if needs_order {
            self.next_order()?
        } else {
            self.order
        };
        for ((cell, value), assert) in self.slots[handle.slot]
            .cells
            .iter_mut()
            .zip(frame.values)
            .zip(frame.assert)
        {
            if value.is_none() {
                cell.order = 0;
            } else if cell.value.is_none() || *assert {
                cell.order = order;
            }
            cell.value = *value;
        }
        state.serial = frame.serial;
        self.slots[handle.slot].state = Some(state);
        Ok(())
    }
    /// Intensity-only attenuation. Zero does not release colors, positions or ownership.
    /// # Errors
    /// Reject stale handle or command order without changing the level.
    pub fn set_level(&mut self, handle: Handle, serial: u64, level: u16) -> Result<(), Error> {
        let mut state = self.update_command(handle, serial)?;
        state.level = level;
        state.serial = serial;
        self.slots[handle.slot].state = Some(state);
        Ok(())
    }
    /// Immediately return every attribute to the remaining sources, then defaults.
    /// # Errors
    /// Reject old handles/commands; old owners cannot close a reused slot.
    pub fn close(&mut self, handle: Handle, serial: u64) -> Result<(), Error> {
        self.command(handle, serial)?;
        let slot = &mut self.slots[handle.slot];
        slot.state = None;
        slot.cells.fill(Cell::default());
        Ok(())
    }
}
