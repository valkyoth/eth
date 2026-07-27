use eth_valkyoth_primitives::Address;

use crate::HostCapabilityError;

/// Maximum iterative call-frame capacity admitted by the EVM.
pub const MAX_ITERATIVE_CALL_FRAMES: usize = 1024;
/// Maximum borrowed transaction-memory capacity in this release.
pub const MAX_TRANSACTION_MEMORY_BYTES: usize = 16_777_216;

/// One iterative EVM call-frame descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IterativeCallFrame {
    /// Executing account.
    pub address: Address,
    /// Whether state-changing operations are forbidden.
    pub is_static: bool,
}

/// Resettable bounded transaction arena contract.
pub trait TransactionArena {
    /// Destructively clears transaction-local memory and frames.
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError>;

    /// Makes `required_len` zero-initialized memory bytes visible.
    fn reserve_memory(&mut self, required_len: usize) -> Result<(), HostCapabilityError>;

    /// Enters one iterative child frame.
    fn enter_frame(&mut self, frame: IterativeCallFrame) -> Result<usize, HostCapabilityError>;

    /// Leaves the active iterative child frame.
    fn leave_frame(&mut self) -> Result<IterativeCallFrame, HostCapabilityError>;

    /// Current iterative frame depth.
    fn frame_depth(&self) -> usize;
}

/// Allocation-free borrowed transaction arena.
#[derive(Debug, Eq, PartialEq)]
pub struct BorrowedTransactionArena<'a, const FRAMES: usize> {
    memory: &'a mut [u8],
    memory_len: usize,
    frames: [Option<IterativeCallFrame>; FRAMES],
    frame_len: usize,
}

impl<'a, const FRAMES: usize> BorrowedTransactionArena<'a, FRAMES> {
    /// Creates and clears a bounded arena.
    pub fn try_new(memory: &'a mut [u8]) -> Result<Self, HostCapabilityError> {
        if FRAMES == 0 || FRAMES > MAX_ITERATIVE_CALL_FRAMES {
            return Err(HostCapabilityError::InvalidFrameCapacity);
        }
        if memory.len() > MAX_TRANSACTION_MEMORY_BYTES {
            return Err(HostCapabilityError::MemoryCapacityExceeded);
        }
        memory.fill(0);
        Ok(Self {
            memory,
            memory_len: 0,
            frames: [None; FRAMES],
            frame_len: 0,
        })
    }

    /// Currently visible transaction memory.
    #[must_use]
    pub fn memory(&self) -> &[u8] {
        self.memory.get(..self.memory_len).unwrap_or(&[])
    }
}

impl<const FRAMES: usize> TransactionArena for BorrowedTransactionArena<'_, FRAMES> {
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.memory.fill(0);
        self.memory_len = 0;
        self.frames.fill(None);
        self.frame_len = 0;
        Ok(())
    }

    fn reserve_memory(&mut self, required_len: usize) -> Result<(), HostCapabilityError> {
        if required_len > self.memory.len() {
            return Err(HostCapabilityError::MemoryCapacityExceeded);
        }
        if required_len > self.memory_len {
            let extension = self
                .memory
                .get_mut(self.memory_len..required_len)
                .ok_or(HostCapabilityError::MemoryCapacityExceeded)?;
            extension.fill(0);
            self.memory_len = required_len;
        }
        Ok(())
    }

    fn enter_frame(&mut self, frame: IterativeCallFrame) -> Result<usize, HostCapabilityError> {
        let slot = self
            .frames
            .get_mut(self.frame_len)
            .ok_or(HostCapabilityError::CallDepthExceeded)?;
        *slot = Some(frame);
        self.frame_len = self
            .frame_len
            .checked_add(1)
            .ok_or(HostCapabilityError::CallDepthExceeded)?;
        Ok(self.frame_len)
    }

    fn leave_frame(&mut self) -> Result<IterativeCallFrame, HostCapabilityError> {
        let index = self
            .frame_len
            .checked_sub(1)
            .ok_or(HostCapabilityError::CallFrameMissing)?;
        let frame = self
            .frames
            .get_mut(index)
            .and_then(Option::take)
            .ok_or(HostCapabilityError::CallFrameMissing)?;
        self.frame_len = index;
        Ok(frame)
    }

    fn frame_depth(&self) -> usize {
        self.frame_len
    }
}
