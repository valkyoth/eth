use crate::{EvmCoreError, EvmWord};

/// Ethereum stack item limit.
pub const EVM_STACK_LIMIT: usize = 1024;

/// Fixed-capacity EVM stack with no allocator dependency.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvmStack<const N: usize> {
    values: [EvmWord; N],
    len: usize,
}

impl<const N: usize> EvmStack<N> {
    /// Creates an empty stack after validating the compile-time capacity.
    pub const fn try_new() -> Result<Self, EvmCoreError> {
        if N == 0 {
            return Err(EvmCoreError::StackCapacityZero);
        }
        if N > EVM_STACK_LIMIT {
            return Err(EvmCoreError::StackCapacityTooLarge);
        }
        Ok(Self {
            values: [EvmWord::ZERO; N],
            len: 0,
        })
    }

    /// Returns the current stack depth.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the stack is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the configured capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Pushes one word onto the stack.
    pub fn push(&mut self, value: EvmWord) -> Result<(), EvmCoreError> {
        let slot = self
            .values
            .get_mut(self.len)
            .ok_or(EvmCoreError::StackOverflow)?;
        *slot = value;
        self.len = self.len.checked_add(1).ok_or(EvmCoreError::StackOverflow)?;
        Ok(())
    }

    /// Pops one word from the stack and clears the vacated slot.
    pub fn pop(&mut self) -> Result<EvmWord, EvmCoreError> {
        let next_len = self
            .len
            .checked_sub(1)
            .ok_or(EvmCoreError::StackUnderflow)?;
        let slot = self
            .values
            .get_mut(next_len)
            .ok_or(EvmCoreError::StackUnderflow)?;
        let value = *slot;
        *slot = EvmWord::ZERO;
        self.len = next_len;
        Ok(value)
    }
}
