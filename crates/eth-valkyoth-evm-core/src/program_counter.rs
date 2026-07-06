use crate::EvmCoreError;

/// Program counter domain with checked advancement.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProgramCounter(usize);

impl ProgramCounter {
    /// Constructs a program counter from a byte offset.
    #[must_use]
    pub const fn new(offset: usize) -> Self {
        Self(offset)
    }

    /// Returns the current byte offset.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }

    /// Advances the counter by `bytes`.
    pub fn advance(self, bytes: usize) -> Result<Self, EvmCoreError> {
        let next = self
            .0
            .checked_add(bytes)
            .ok_or(EvmCoreError::ProgramCounterOverflow)?;
        Ok(Self(next))
    }
}
