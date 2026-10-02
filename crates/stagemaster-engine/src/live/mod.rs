//! Indexed host composition. Allocation happens at preparation, never per live command/frame.
mod commands;
mod render;
mod types;
pub use types::{
    Attribute, Error, Frame, Handle, Kind, Layout, LayoutId, MAX_ATTRIBUTES, MAX_SOURCES, Source,
    SourceState,
};

#[derive(Clone, Copy, Default)]
struct Cell {
    value: Option<u16>,
    order: u64,
}
struct Slot {
    state: Option<SourceState>,
    cells: Vec<Cell>,
}
pub struct LiveMixer {
    boot: [u8; 16],
    order: u64,
    layout: Layout,
    slots: Vec<Slot>,
}
impl LiveMixer {
    /// Caller supplies a fresh identity on every reconstruction and a bounded source budget.
    /// # Errors
    /// Reject invalid identity/capacity or preparation allocation failure.
    pub fn new(boot: [u8; 16], layout: Layout, sources: usize) -> Result<Self, Error> {
        if boot == [0; 16] {
            return Err(Error::Identity);
        }
        if !(1..=MAX_SOURCES).contains(&sources) {
            return Err(Error::Budget);
        }
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(sources)
            .map_err(|_| Error::Allocation)?;
        for _ in 0..sources {
            let mut cells = Vec::new();
            cells
                .try_reserve_exact(layout.attributes.len())
                .map_err(|_| Error::Allocation)?;
            cells.resize(layout.attributes.len(), Cell::default());
            slots.push(Slot { state: None, cells });
        }
        Ok(Self {
            boot,
            order: 0,
            layout,
            slots,
        })
    }
    #[must_use]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }
    pub fn sources(&self) -> impl Iterator<Item = SourceState> + '_ {
        self.slots.iter().filter_map(|s| s.state)
    }
    /// Inspect existing source metadata without copying property buffers.
    /// # Errors
    /// Reject invalid/stale capabilities.
    pub fn source(&self, handle: Handle) -> Result<SourceState, Error> {
        self.state(handle)
    }
    /// Exact reserved vector payload, excluding allocator metadata and caller-owned objects.
    #[must_use]
    pub fn reserved_bytes(&self) -> usize {
        self.layout.attributes.capacity() * core::mem::size_of::<Attribute>()
            + self.slots.capacity() * core::mem::size_of::<Slot>()
            + self
                .slots
                .iter()
                .map(|s| s.cells.capacity() * core::mem::size_of::<Cell>())
                .sum::<usize>()
    }
    fn state(&self, handle: Handle) -> Result<SourceState, Error> {
        self.slots
            .get(handle.slot)
            .and_then(|slot| slot.state)
            .filter(|state| state.handle == handle)
            .ok_or(Error::Handle)
    }
    fn command(&self, handle: Handle, serial: u64) -> Result<SourceState, Error> {
        let state = self.state(handle)?;
        if serial == 0 || serial <= state.serial {
            return Err(Error::Sequence);
        }
        Ok(state)
    }
    fn update_command(&self, handle: Handle, serial: u64) -> Result<SourceState, Error> {
        let state = self.command(handle, serial)?;
        // Reserve the final per-source command for an explicit close.
        if serial == u64::MAX {
            return Err(Error::Exhausted);
        }
        Ok(state)
    }
    fn next_order(&mut self) -> Result<u64, Error> {
        let next = self.order.checked_add(1).ok_or(Error::Exhausted)?;
        self.order = next;
        Ok(next)
    }
}
#[cfg(test)]
mod tests;
