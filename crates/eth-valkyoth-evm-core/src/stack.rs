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

    /// Returns a word by depth, where depth zero is the top item.
    pub fn peek(&self, depth: usize) -> Result<EvmWord, EvmCoreError> {
        let offset = depth.checked_add(1).ok_or(EvmCoreError::StackUnderflow)?;
        let index = self
            .len
            .checked_sub(offset)
            .ok_or(EvmCoreError::StackUnderflow)?;
        self.values
            .get(index)
            .copied()
            .ok_or(EvmCoreError::StackUnderflow)
    }

    /// Duplicates a word by depth, where depth zero is `DUP1`.
    pub fn dup(&mut self, depth: usize) -> Result<(), EvmCoreError> {
        let value = self.peek(depth)?;
        self.push(value)
    }

    /// Swaps the top word with a deeper stack item, where depth one is `SWAP1`.
    pub fn swap_with_top(&mut self, depth: usize) -> Result<(), EvmCoreError> {
        if depth == 0 {
            return Err(EvmCoreError::StackUnderflow);
        }
        let top_index = self
            .len
            .checked_sub(1)
            .ok_or(EvmCoreError::StackUnderflow)?;
        let offset = depth.checked_add(1).ok_or(EvmCoreError::StackUnderflow)?;
        let other_index = self
            .len
            .checked_sub(offset)
            .ok_or(EvmCoreError::StackUnderflow)?;
        let top = self
            .values
            .get(top_index)
            .copied()
            .ok_or(EvmCoreError::StackUnderflow)?;
        let other = self
            .values
            .get(other_index)
            .copied()
            .ok_or(EvmCoreError::StackUnderflow)?;
        let top_slot = self
            .values
            .get_mut(top_index)
            .ok_or(EvmCoreError::StackUnderflow)?;
        *top_slot = other;
        let other_slot = self
            .values
            .get_mut(other_index)
            .ok_or(EvmCoreError::StackUnderflow)?;
        *other_slot = top;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn debug_raw_slot(&self, offset: usize) -> Option<EvmWord> {
        self.values.get(offset).copied()
    }
}
