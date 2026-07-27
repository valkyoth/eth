use core::fmt;

use eth_valkyoth_primitives::{Address, B256};

use crate::{ExecutionEnvironment, StateView};

/// Maximum iterative call-frame capacity admitted by the EVM.
pub const MAX_ITERATIVE_CALL_FRAMES: usize = 1024;
/// Maximum borrowed transaction-memory capacity in this release.
pub const MAX_TRANSACTION_MEMORY_BYTES: usize = 16_777_216;

/// Warm/cold result from a transaction-global access tracker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessStatus {
    /// Address or slot was already warm.
    Warm,
    /// Address or slot was cold and is now warm.
    Cold,
}

/// Block and fork data visible to execution.
pub trait BlockEnvironment {
    /// Returns the immutable environment selected during admission.
    fn execution_environment(&self) -> ExecutionEnvironment;
}

impl BlockEnvironment for ExecutionEnvironment {
    fn execution_environment(&self) -> ExecutionEnvironment {
        *self
    }
}

/// Journaled state mutations with explicit child checkpoints.
pub trait StateJournal {
    /// Non-forgeable implementation-defined checkpoint token.
    type Checkpoint;

    /// Clears transaction-local mutation state.
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError>;

    /// Starts one child-call checkpoint.
    fn checkpoint(&mut self) -> Result<Self::Checkpoint, HostCapabilityError>;

    /// Commits the active child checkpoint.
    fn commit(&mut self, checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError>;

    /// Reverts the active child checkpoint.
    fn revert(&mut self, checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError>;

    /// Writes one storage slot into the current journal view.
    fn set_storage(
        &mut self,
        address: Address,
        slot: B256,
        value: B256,
    ) -> Result<(), HostCapabilityError>;
}

/// Transaction-global EIP-2929-style warmth tracking.
///
/// No child-checkpoint operation exists: warmth survives child failure and
/// revert for the remainder of the transaction.
pub trait AccessTracker {
    /// Clears all transaction-global warmth.
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError>;

    /// Warms an address and reports its previous state.
    fn warm_address(&mut self, address: Address) -> Result<AccessStatus, HostCapabilityError>;

    /// Warms an address/storage pair and reports its previous state.
    fn warm_storage(
        &mut self,
        address: Address,
        slot: B256,
    ) -> Result<AccessStatus, HostCapabilityError>;
}

/// Cryptographic powers supplied to execution by a reviewed backend.
pub trait CryptoProvider {
    /// Computes Ethereum Keccak-256.
    fn keccak256(&mut self, input: &[u8]) -> Result<B256, HostCapabilityError>;

    /// Recovers an Ethereum address from a digest and 65-byte signature.
    fn recover_address(
        &mut self,
        digest: B256,
        signature: &[u8; 65],
    ) -> Result<Address, HostCapabilityError>;
}

/// Immutable execution event exposed to observation-only inspectors.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectorEvent {
    /// Transaction-local host state was reset.
    TransactionStarted,
    /// A child frame started.
    ChildStarted {
        /// Zero-based frame depth after entry.
        depth: usize,
    },
    /// A child frame committed.
    ChildCommitted {
        /// Zero-based frame depth before exit.
        depth: usize,
    },
    /// A child frame reverted.
    ChildReverted {
        /// Zero-based frame depth before exit.
        depth: usize,
    },
}

/// Observation-only execution hook.
///
/// Inspectors return no consensus decision and receive no mutable state.
pub trait Inspector {
    /// Observes one immutable lifecycle event.
    fn observe(&mut self, event: InspectorEvent);
}

/// Inspector implementation for hosts that do not need observation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoInspector;

impl Inspector for NoInspector {
    fn observe(&mut self, _event: InspectorEvent) {}
}

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

/// Child-call token binding journal and iterative-frame state.
#[derive(Debug)]
pub struct ChildCheckpoint<C> {
    journal: C,
    depth: usize,
}

/// Explicit bundle of powers available to an execution machine.
pub struct ExecutionHost<'a, V, J, B, A, C, I, R>
where
    V: StateView + ?Sized,
    J: StateJournal,
    B: BlockEnvironment + ?Sized,
    A: AccessTracker,
    C: CryptoProvider,
    I: Inspector,
    R: TransactionArena,
{
    /// Snapshot-pure state reads.
    pub state: &'a V,
    /// Journaled state writes.
    pub journal: &'a mut J,
    /// Immutable block and fork context.
    pub block: &'a B,
    /// Transaction-global warmth.
    pub access: &'a mut A,
    /// Reviewed cryptographic operations.
    pub crypto: &'a mut C,
    /// Observation-only hooks.
    pub inspector: &'a mut I,
    /// Resettable memory and iterative frames.
    pub arena: &'a mut R,
}

impl<V, J, B, A, C, I, R> ExecutionHost<'_, V, J, B, A, C, I, R>
where
    V: StateView + ?Sized,
    J: StateJournal,
    B: BlockEnvironment + ?Sized,
    A: AccessTracker,
    C: CryptoProvider,
    I: Inspector,
    R: TransactionArena,
{
    /// Destructively starts one transaction-local host lifecycle.
    pub fn begin_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.journal.reset_transaction()?;
        self.access.reset_transaction()?;
        self.arena.reset_transaction()?;
        self.inspector.observe(InspectorEvent::TransactionStarted);
        Ok(())
    }

    /// Starts one iterative child frame and journal checkpoint.
    pub fn begin_child(
        &mut self,
        frame: IterativeCallFrame,
    ) -> Result<ChildCheckpoint<J::Checkpoint>, HostCapabilityError> {
        let checkpoint = self.journal.checkpoint()?;
        match self.arena.enter_frame(frame) {
            Ok(depth) => {
                self.inspector
                    .observe(InspectorEvent::ChildStarted { depth });
                Ok(ChildCheckpoint {
                    journal: checkpoint,
                    depth,
                })
            }
            Err(error) => {
                self.journal.revert(checkpoint)?;
                Err(error)
            }
        }
    }

    /// Commits one child journal while retaining transaction-global warmth.
    pub fn commit_child(
        &mut self,
        checkpoint: ChildCheckpoint<J::Checkpoint>,
    ) -> Result<(), HostCapabilityError> {
        self.journal.commit(checkpoint.journal)?;
        let _ = self.arena.leave_frame()?;
        self.inspector.observe(InspectorEvent::ChildCommitted {
            depth: checkpoint.depth,
        });
        Ok(())
    }

    /// Reverts one child journal while retaining transaction-global warmth.
    pub fn revert_child(
        &mut self,
        checkpoint: ChildCheckpoint<J::Checkpoint>,
    ) -> Result<(), HostCapabilityError> {
        self.journal.revert(checkpoint.journal)?;
        let _ = self.arena.leave_frame()?;
        self.inspector.observe(InspectorEvent::ChildReverted {
            depth: checkpoint.depth,
        });
        Ok(())
    }
}

/// Host capability or bounded arena failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostCapabilityError {
    /// Snapshot-pure state read failed.
    StateReadFailed,
    /// Journal lifecycle or mutation failed.
    JournalFailed,
    /// Warm-access tracking failed or reached capacity.
    AccessTrackingFailed,
    /// Cryptographic provider failed.
    CryptoFailed,
    /// Configured frame capacity is zero or exceeds the EVM limit.
    InvalidFrameCapacity,
    /// Iterative call depth reached configured capacity.
    CallDepthExceeded,
    /// No iterative call frame is active.
    CallFrameMissing,
    /// Requested memory exceeds the borrowed arena.
    MemoryCapacityExceeded,
}

impl HostCapabilityError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::StateReadFailed => "ETH_EVM_HOST_STATE_READ",
            Self::JournalFailed => "ETH_EVM_HOST_JOURNAL",
            Self::AccessTrackingFailed => "ETH_EVM_HOST_ACCESS_TRACKING",
            Self::CryptoFailed => "ETH_EVM_HOST_CRYPTO",
            Self::InvalidFrameCapacity => "ETH_EVM_HOST_INVALID_FRAME_CAPACITY",
            Self::CallDepthExceeded => "ETH_EVM_HOST_CALL_DEPTH",
            Self::CallFrameMissing => "ETH_EVM_HOST_CALL_FRAME_MISSING",
            Self::MemoryCapacityExceeded => "ETH_EVM_HOST_MEMORY_CAPACITY",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::StateReadFailed => "execution state view read failed",
            Self::JournalFailed => "execution state journal operation failed",
            Self::AccessTrackingFailed => "execution access tracker operation failed",
            Self::CryptoFailed => "execution cryptographic provider operation failed",
            Self::InvalidFrameCapacity => "iterative call-frame capacity is invalid",
            Self::CallDepthExceeded => "iterative EVM call depth exceeded",
            Self::CallFrameMissing => "iterative EVM call frame is missing",
            Self::MemoryCapacityExceeded => "transaction memory capacity exceeded",
        }
    }
}

impl fmt::Display for HostCapabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HostCapabilityError {}
