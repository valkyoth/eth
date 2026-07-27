use eth_valkyoth_primitives::{Address, B256};

use crate::{
    AccessStatus, AccessTracker, CryptoProvider, ExecutionEnvironment, ExecutionRequest,
    InspectorEvent, SnapshotAccount, StateJournal, StateView,
    arena::{IterativeCallFrame, TransactionArena},
    host_error::{BeginChildError, ChildFinalizeAction, ChildLifecycleError, HostCapabilityError},
    host_scope::{PoisonScope, PoisonScopeHost},
};

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
///
/// Any mutable backend error or unwind poisons the host because partial
/// backend effects cannot be proven absent through these trait contracts.
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
    /// The host remains poisoned after any incomplete reset. The returned
    /// event is safe to dispatch only after this method returns.
    pub fn begin_transaction(&mut self) -> Result<InspectorEvent, HostCapabilityError> {
        self.ensure_usable()?;
        self.poisoned = true;
        self.transaction_started = false;
        if self.arena.frame_depth() != 0 {
            return Err(HostCapabilityError::HostPoisoned);
        }
        self.journal.reset_transaction()?;
        self.access.reset_transaction()?;
        self.arena.reset_transaction()?;
        self.transaction_started = true;
        self.poisoned = false;
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
        let mut scope = PoisonScope::new(self);
        let checkpoint = match scope.host().journal.checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(error) => {
                return Err(ChildLifecycleError::Begin(
                    BeginChildError::CheckpointFailed(error),
                ));
            }
        };
        let access_checkpoint = match scope.host().access.checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(error) => {
                let journal_revert_error = scope.host().journal.revert(checkpoint).err();
                return Err(ChildLifecycleError::Begin(
                    BeginChildError::AccessCheckpointFailed {
                        error,
                        journal_revert_error,
                    },
                ));
            }
        };
        let depth = match scope.host().arena.enter_frame(frame) {
            Ok(depth) => depth,
            Err(frame_error) => {
                let access_revert_error = scope.host().access.revert(access_checkpoint).err();
                let journal_revert_error = scope.host().journal.revert(checkpoint).err();
                if access_revert_error.is_none() && journal_revert_error.is_none() {
                    scope.finish();
                    return Err(ChildLifecycleError::Begin(BeginChildError::FrameRejected {
                        frame_error,
                    }));
                }
                return Err(ChildLifecycleError::Begin(
                    BeginChildError::FrameRejectedAndCleanupFailed {
                        frame_error,
                        access_revert_error,
                        journal_revert_error,
                    },
                ));
            }
        };

        let decision = execute(scope.host());
        if scope.host().poisoned {
            return Err(ChildLifecycleError::HostPoisoned);
        }
        let found_depth = scope.host().arena.frame_depth();
        if found_depth != depth {
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
            ChildFinalizeAction::Commit => scope.host().journal.commit(checkpoint),
            ChildFinalizeAction::Revert => scope.host().journal.revert(checkpoint),
        };
        let access_result = match action {
            ChildFinalizeAction::Commit => scope.host().access.commit(access_checkpoint),
            ChildFinalizeAction::Revert => scope.host().access.revert(access_checkpoint),
        };
        if journal_result.is_err() || access_result.is_err() {
            return Err(ChildLifecycleError::CapabilityConsistencyUnknown {
                action,
                journal_error: journal_result.err(),
                access_error: access_result.err(),
            });
        }

        let removed = match scope.host().arena.leave_frame() {
            Ok(removed) => removed,
            Err(error) => {
                return Err(ChildLifecycleError::ArenaConsistencyUnknown { action, error });
            }
        };
        if removed != frame {
            return Err(ChildLifecycleError::FrameMismatch { action });
        }
        let finished = match action {
            ChildFinalizeAction::Commit => InspectorEvent::ChildCommitted { depth },
            ChildFinalizeAction::Revert => InspectorEvent::ChildReverted { depth },
        };
        scope.finish();
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
        self.with_mutation(|host| host.journal.set_storage(address, slot, value))
    }

    /// Warms one address for the complete transaction.
    pub fn warm_address(&mut self, address: Address) -> Result<AccessStatus, HostCapabilityError> {
        self.with_mutation(|host| host.access.warm_address(address))
    }

    /// Warms one address/slot pair for the complete transaction.
    pub fn warm_storage(
        &mut self,
        address: Address,
        slot: B256,
    ) -> Result<AccessStatus, HostCapabilityError> {
        self.with_mutation(|host| host.access.warm_storage(address, slot))
    }

    /// Computes Keccak-256 through the reviewed request-bound provider.
    pub fn keccak256(&mut self, input: &[u8]) -> Result<B256, HostCapabilityError> {
        self.with_mutation(|host| host.crypto.keccak256(input))
    }

    /// Recovers one address through the reviewed request-bound provider.
    pub fn recover_address(
        &mut self,
        digest: B256,
        signature: &[u8; 65],
    ) -> Result<Address, HostCapabilityError> {
        self.with_mutation(|host| host.crypto.recover_address(digest, signature))
    }

    /// Expands zero-initialized transaction memory within the arena bound.
    pub fn reserve_memory(&mut self, required_len: usize) -> Result<(), HostCapabilityError> {
        self.with_mutation(|host| host.arena.reserve_memory(required_len))
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

    fn with_mutation<T>(
        &mut self,
        mutate: impl FnOnce(&mut Self) -> Result<T, HostCapabilityError>,
    ) -> Result<T, HostCapabilityError> {
        self.ensure_started()?;
        let mut scope = PoisonScope::new(self);
        let output = mutate(scope.host())?;
        scope.finish();
        Ok(output)
    }
}

impl<J, A, C, R> PoisonScopeHost for ExecutionHost<'_, '_, J, A, C, R>
where
    J: StateJournal,
    A: AccessTracker,
    C: CryptoProvider,
    R: TransactionArena,
{
    fn poison_scope(&mut self) {
        self.poisoned = true;
        self.transaction_started = false;
    }
}
