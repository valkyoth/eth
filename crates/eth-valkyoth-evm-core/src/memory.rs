use crate::EvmCoreError;

/// Hard cap for one bounded memory view in this bootstrap release.
pub const EVM_MEMORY_LIMIT_BYTES: usize = 16_777_216;

/// Borrowed EVM memory view with an explicit release cap.
#[derive(Debug, Eq, PartialEq)]
pub struct EvmMemory<'a> {
    bytes: &'a mut [u8],
}

impl<'a> EvmMemory<'a> {
    /// Validates a memory view length without requiring an allocation.
    pub const fn validate_len(len: usize) -> Result<(), EvmCoreError> {
        if len > EVM_MEMORY_LIMIT_BYTES {
            return Err(EvmCoreError::MemoryTooLarge);
        }
        Ok(())
    }

    /// Creates a bounded borrowed memory view.
    pub fn try_new(bytes: &'a mut [u8]) -> Result<Self, EvmCoreError> {
        Self::validate_len(bytes.len())?;
        Ok(Self { bytes })
    }

    /// Returns the memory view length in bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns whether the memory view is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Reads one byte at `offset`.
    pub fn read_byte(&self, offset: usize) -> Result<u8, EvmCoreError> {
        self.bytes
            .get(offset)
            .copied()
            .ok_or(EvmCoreError::MemoryOffsetOutOfBounds)
    }

    /// Writes one byte at `offset`.
    pub fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), EvmCoreError> {
        let slot = self
            .bytes
            .get_mut(offset)
            .ok_or(EvmCoreError::MemoryOffsetOutOfBounds)?;
        *slot = value;
        Ok(())
    }

    /// Writes a checked byte range into memory.
    pub fn write_range(&mut self, offset: usize, value: &[u8]) -> Result<(), EvmCoreError> {
        let end = offset
            .checked_add(value.len())
            .ok_or(EvmCoreError::MemoryOffsetOutOfBounds)?;
        let target = self
            .bytes
            .get_mut(offset..end)
            .ok_or(EvmCoreError::MemoryOffsetOutOfBounds)?;
        target.copy_from_slice(value);
        Ok(())
    }

    /// Returns a checked mutable memory range.
    pub(crate) fn checked_range_mut(
        &mut self,
        offset: usize,
        len: usize,
    ) -> Result<&mut [u8], EvmCoreError> {
        let end = offset
            .checked_add(len)
            .ok_or(EvmCoreError::MemoryOffsetOutOfBounds)?;
        self.bytes
            .get_mut(offset..end)
            .ok_or(EvmCoreError::MemoryOffsetOutOfBounds)
    }

    /// Checks whether a memory range is fully inside the view.
    pub fn check_range(&self, offset: usize, len: usize) -> Result<(), EvmCoreError> {
        let end = offset
            .checked_add(len)
            .ok_or(EvmCoreError::MemoryOffsetOutOfBounds)?;
        if end > self.len() {
            return Err(EvmCoreError::MemoryOffsetOutOfBounds);
        }
        Ok(())
    }

    /// Borrows the memory bytes.
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
        self.bytes
    }

    /// Mutably borrows the memory bytes.
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.bytes
    }
}
