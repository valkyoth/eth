use eth_valkyoth_primitives::{Address, B256};

use crate::{
    ExecutionEnvironment, ExecutionRequest, SnapshotAccount, StateView,
    arena::{IterativeCallFrame, TransactionArena},
    host_error::{BeginChildError, ChildFinalizeAction, ChildLifecycleError, HostCapabilityError},
};

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

/// Finalization selected by one closure-scoped child execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildDecision<T> {
    /// Keep child journal changes.
    Commit(T),
    /// Roll back child journal changes.
    Revert(T),
}

/// Completed child output and immutable post-transition observation events.
#[derive(Debug, Eq, PartialEq)]
pub struct ChildExecution<T> {
    output: T,
    events: [InspectorEvent; 2],
}

impl<T> ChildExecution<T> {
    /// Result returned by the child execution closure.
    #[must_use]
    pub const fn output(&self) -> &T {
        &self.output
    }

    /// Events that may be dispatched after all critical transitions completed.
    #[must_use]
    pub const fn events(&self) -> &[InspectorEvent; 2] {
        &self.events
    }

    /// Consumes the evidence wrapper and returns the child output.
    #[must_use]
    pub fn into_output(self) -> T {
        self.output
    }
}

/// Request-bound bundle of powers available to an execution machine.
pub struct ExecutionHost<'host, 'transaction, J, A, C, R>
where
    J: StateJournal,
    A: AccessTracker,
    C: CryptoProvider,
    R: TransactionArena,
{
    request: &'host ExecutionRequest<'transaction, J::View>,
    journal: &'host mut J,
    access: &'host mut A,
    crypto: &'host mut C,
    arena: &'host mut R,
    poisoned: bool,
    transaction_started: bool,
}

impl<'host, 'transaction, J, A, C, R> ExecutionHost<'host, 'transaction, J, A, C, R>
where
    J: StateJournal,
    A: AccessTracker,
    C: CryptoProvider,
    R: TransactionArena,
{
    /// Binds all mutable host capabilities to one admitted execution request.
    #[must_use]
    pub fn new(
        request: &'host ExecutionRequest<'transaction, J::View>,
        journal: &'host mut J,
        access: &'host mut A,
        crypto: &'host mut C,
        arena: &'host mut R,
    ) -> Self {
        Self {
            request,
            journal,
            access,
            crypto,
            arena,
            poisoned: false,
            transaction_started: false,
        }
    }

    /// Exact admitted request that owns this host lifecycle.
    #[must_use]
    pub const fn request(&self) -> &ExecutionRequest<'transaction, J::View> {
        self.request
    }

    /// Immutable state view selected by the admitted request.
    #[must_use]
    pub const fn state(&self) -> &J::View {
        self.request.state()
    }

    /// Immutable environment selected during transaction admission.
    #[must_use]
    pub const fn environment(&self) -> ExecutionEnvironment {
        self.request.environment()
    }

    /// Whether an earlier partial transition made host consistency unknown.
    #[must_use]
    pub const fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    /// Destructively starts one transaction-local host lifecycle.
    ///
    /// The returned event is safe to dispatch only after this method returns.
    pub fn begin_transaction(&mut self) -> Result<InspectorEvent, HostCapabilityError> {
        self.ensure_usable()?;
        if self.arena.frame_depth() != 0 {
            return self.poison(HostCapabilityError::HostPoisoned);
        }
        if let Err(error) = self.journal.reset_transaction() {
            return self.poison(error);
        }
        if let Err(error) = self.access.reset_transaction() {
            return self.poison(error);
        }
        if let Err(error) = self.arena.reset_transaction() {
            return self.poison(error);
        }
        self.transaction_started = true;
        Ok(InspectorEvent::TransactionStarted)
    }

    /// Executes and finalizes one child without exposing a checkpoint token.
    ///
    /// Nested calls use this method recursively through the closure. The outer
    /// lifecycle cannot finalize until every inner lifecycle has completed.
    pub fn with_child<T, F>(
        &mut self,
        frame: IterativeCallFrame,
        execute: F,
    ) -> Result<ChildExecution<T>, ChildLifecycleError>
    where
        F: FnOnce(&mut Self) -> ChildDecision<T>,
    {
        if self.poisoned {
            return Err(ChildLifecycleError::HostPoisoned);
        }
        if !self.transaction_started {
            return Err(ChildLifecycleError::TransactionNotStarted);
        }
        let checkpoint = match self.journal.checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(error) => {
                self.poisoned = true;
                return Err(ChildLifecycleError::Begin(
                    BeginChildError::CheckpointFailed(error),
                ));
            }
        };
        let depth = match self.arena.enter_frame(frame) {
            Ok(depth) => depth,
            Err(frame_error) => {
                return match self.journal.revert(checkpoint) {
                    Ok(()) => Err(ChildLifecycleError::Begin(BeginChildError::FrameRejected {
                        frame_error,
                    })),
                    Err(revert_error) => {
                        self.poisoned = true;
                        Err(ChildLifecycleError::Begin(
                            BeginChildError::FrameRejectedAndJournalRevertFailed {
                                frame_error,
                                revert_error,
                            },
                        ))
                    }
                };
            }
        };

        let decision = execute(self);
        if self.poisoned {
            return Err(ChildLifecycleError::HostPoisoned);
        }
        let found_depth = self.arena.frame_depth();
        if found_depth != depth {
            self.poisoned = true;
            return Err(ChildLifecycleError::FrameDepthMismatch {
                expected: depth,
                found: found_depth,
            });
        }

        let (action, output) = match decision {
            ChildDecision::Commit(output) => (ChildFinalizeAction::Commit, output),
            ChildDecision::Revert(output) => (ChildFinalizeAction::Revert, output),
        };
        let journal_result = match action {
            ChildFinalizeAction::Commit => self.journal.commit(checkpoint),
            ChildFinalizeAction::Revert => self.journal.revert(checkpoint),
        };
        if let Err(error) = journal_result {
            self.poisoned = true;
            return Err(ChildLifecycleError::JournalConsistencyUnknown { action, error });
        }

        let removed = match self.arena.leave_frame() {
            Ok(removed) => removed,
            Err(error) => {
                self.poisoned = true;
                return Err(ChildLifecycleError::ArenaConsistencyUnknown { action, error });
            }
        };
        if removed != frame {
            self.poisoned = true;
            return Err(ChildLifecycleError::FrameMismatch { action });
        }
        let finished = match action {
            ChildFinalizeAction::Commit => InspectorEvent::ChildCommitted { depth },
            ChildFinalizeAction::Revert => InspectorEvent::ChildReverted { depth },
        };
        Ok(ChildExecution {
            output,
            events: [InspectorEvent::ChildStarted { depth }, finished],
        })
    }

    /// Reads one account from the request-bound immutable view.
    pub fn account(
        &self,
        address: Address,
    ) -> Result<Option<SnapshotAccount>, HostCapabilityError> {
        self.ensure_usable()?;
        self.state()
            .account(address)
            .map_err(|_| HostCapabilityError::StateReadFailed)
    }

    /// Reads transaction-start storage from the request-bound immutable view.
    pub fn original_storage(
        &self,
        address: Address,
        slot: B256,
    ) -> Result<B256, HostCapabilityError> {
        self.ensure_usable()?;
        self.state()
            .original_storage(address, slot)
            .map_err(|_| HostCapabilityError::StateReadFailed)
    }

    /// Reads current storage through the journal associated with this view.
    pub fn current_storage(
        &self,
        address: Address,
        slot: B256,
    ) -> Result<B256, HostCapabilityError> {
        self.ensure_started()?;
        self.journal.current_storage(self.state(), address, slot)
    }

    /// Writes current storage through the private request-bound journal.
    pub fn set_storage(
        &mut self,
        address: Address,
        slot: B256,
        value: B256,
    ) -> Result<(), HostCapabilityError> {
        self.ensure_started()?;
        self.journal.set_storage(address, slot, value)
    }

    /// Warms one address for the complete transaction.
    pub fn warm_address(&mut self, address: Address) -> Result<AccessStatus, HostCapabilityError> {
        self.ensure_started()?;
        self.access.warm_address(address)
    }

    /// Warms one address/slot pair for the complete transaction.
    pub fn warm_storage(
        &mut self,
        address: Address,
        slot: B256,
    ) -> Result<AccessStatus, HostCapabilityError> {
        self.ensure_started()?;
        self.access.warm_storage(address, slot)
    }

    /// Computes Keccak-256 through the reviewed request-bound provider.
    pub fn keccak256(&mut self, input: &[u8]) -> Result<B256, HostCapabilityError> {
        self.ensure_started()?;
        self.crypto.keccak256(input)
    }

    /// Recovers one address through the reviewed request-bound provider.
    pub fn recover_address(
        &mut self,
        digest: B256,
        signature: &[u8; 65],
    ) -> Result<Address, HostCapabilityError> {
        self.ensure_started()?;
        self.crypto.recover_address(digest, signature)
    }

    /// Expands zero-initialized transaction memory within the arena bound.
    pub fn reserve_memory(&mut self, required_len: usize) -> Result<(), HostCapabilityError> {
        self.ensure_started()?;
        self.arena.reserve_memory(required_len)
    }

    /// Current iterative frame depth for deterministic execution scheduling.
    #[must_use]
    pub fn frame_depth(&self) -> usize {
        self.arena.frame_depth()
    }

    fn ensure_usable(&self) -> Result<(), HostCapabilityError> {
        if self.poisoned {
            return Err(HostCapabilityError::HostPoisoned);
        }
        Ok(())
    }

    fn ensure_started(&self) -> Result<(), HostCapabilityError> {
        self.ensure_usable()?;
        if !self.transaction_started {
            return Err(HostCapabilityError::TransactionNotStarted);
        }
        Ok(())
    }

    fn poison<T>(&mut self, error: HostCapabilityError) -> Result<T, HostCapabilityError> {
        self.poisoned = true;
        Err(error)
    }
}
