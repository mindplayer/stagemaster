use crate::{Binding, Code, LocalIdentity};

pub const MAX_BINDINGS: usize = 4;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vault {
    pub(crate) generation: u64,
    pub(crate) local: LocalIdentity,
    pub(crate) bindings: [Option<Binding>; MAX_BINDINGS],
}
impl Vault {
    #[must_use]
    pub fn new(local: LocalIdentity) -> Self {
        Self {
            generation: 1,
            local,
            bindings: core::array::from_fn(|_| None),
        }
    }
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    #[must_use]
    pub const fn local(&self) -> &LocalIdentity {
        &self.local
    }
    pub fn bindings(&self) -> impl Iterator<Item = &Binding> {
        self.bindings.iter().flatten()
    }
    #[must_use]
    pub fn find(&self, principal: [u8; 16]) -> Option<&Binding> {
        self.bindings().find(|b| b.principal == principal)
    }
    /// Create a proposed revision; this does NOT make it durable or authorize a connection.
    /// # Errors
    /// Refuses duplicate peer identities and silent replacement of another principal.
    pub fn enroll(&self, binding: Binding) -> Result<Self, Code> {
        if self
            .bindings()
            .any(|b| b.principal != binding.principal && b.same_identity(&binding))
        {
            return Err(Code::Duplicate);
        }
        let index = if let Some(index) = self
            .bindings
            .iter()
            .position(|b| b.as_ref().is_some_and(|b| b.principal == binding.principal))
        {
            if self.bindings[index]
                .as_ref()
                .is_none_or(|previous| !previous.same_identity(&binding))
            {
                return Err(Code::Conflict);
            }
            index
        } else {
            self.bindings
                .iter()
                .position(Option::is_none)
                .ok_or(Code::Full)?
        };
        let mut next = self.next()?;
        next.bindings[index] = Some(binding);
        Ok(next)
    }
    /// # Errors
    /// Refuses unknown principals and revision exhaustion; requires durable commit afterwards.
    pub fn revoke(&self, principal: [u8; 16]) -> Result<Self, Code> {
        let index = self
            .bindings
            .iter()
            .position(|b| b.as_ref().is_some_and(|b| b.principal == principal))
            .ok_or(Code::Missing)?;
        let mut next = self.next()?;
        for i in index..MAX_BINDINGS - 1 {
            next.bindings[i] = next.bindings[i + 1].take();
        }
        next.bindings[MAX_BINDINGS - 1] = None;
        Ok(next)
    }
    fn next(&self) -> Result<Self, Code> {
        let mut next = self.clone();
        next.generation = next.generation.checked_add(1).ok_or(Code::Exhausted)?;
        Ok(next)
    }
}
