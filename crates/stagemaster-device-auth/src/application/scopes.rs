/// Application operations are independent from device capabilities, operator leases
/// and file playback licenses. No scope implies another scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Installation = 1,
    Observe = 2,
    Control = 4,
}

/// A nonempty selection made by trusted local provisioning, never a wire credential.
/// No integer conversion or deserialization can turn peer claims into permissions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permissions(u8);
impl Permissions {
    #[must_use]
    pub const fn only(scope: Scope) -> Self {
        Self(scope as u8)
    }

    #[must_use]
    pub const fn with(self, scope: Scope) -> Self {
        Self(self.0 | scope as u8)
    }

    #[must_use]
    pub const fn contains(self, scope: Scope) -> bool {
        self.0 & scope as u8 != 0
    }
}
