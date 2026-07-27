use eth_valkyoth_primitives::{Address, B256};

use crate::{HostCapabilityError, StateView};

/// Warm/cold result from a transaction-global access tracker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessStatus {
    /// Address or slot was already warm.
    Warm,
    /// Address or slot was cold and is now warm.
    Cold,
}

/// Journaled state mutations with explicit child checkpoints.
pub trait StateJournal {
    /// Exact immutable state-view type compatible with this journal.
    type View: StateView + ?Sized;
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
    /// Reads one slot from the current journal overlay or immutable base.
    fn current_storage(
        &self,
        base: &Self::View,
        address: Address,
        slot: B256,
    ) -> Result<B256, HostCapabilityError>;
    /// Writes one storage slot into the current journal view.
    fn set_storage(
        &mut self,
        address: Address,
        slot: B256,
        value: B256,
    ) -> Result<(), HostCapabilityError>;
}

/// Transaction-wide EIP-2929 warmth with nested LIFO scope checkpoints.
pub trait AccessTracker {
    /// Non-forgeable implementation-defined checkpoint token.
    type Checkpoint;

    /// Clears all transaction warmth and invalidates old checkpoints.
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError>;
    /// Starts one child-scope checkpoint.
    fn checkpoint(&mut self) -> Result<Self::Checkpoint, HostCapabilityError>;
    /// Commits the active child-scope checkpoint.
    fn commit(&mut self, checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError>;
    /// Restores warmth to the active child-scope checkpoint.
    fn revert(&mut self, checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError>;
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
        /// Active frame count after entry; the first nested frame reports one.
        depth: usize,
    },
    /// A child frame committed.
    ChildCommitted {
        /// Active frame count before exit; the first nested frame reports one.
        depth: usize,
    },
    /// A child frame reverted.
    ChildReverted {
        /// Active frame count before exit; the first nested frame reports one.
        depth: usize,
    },
}

/// Observation-only execution hook with no consensus decision authority.
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
