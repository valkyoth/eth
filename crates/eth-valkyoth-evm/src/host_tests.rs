use eth_valkyoth_codec::{DecodeLimits, DecodeSessionPolicy};
use eth_valkyoth_primitives::{
    Address, B256, BlockNumber, ChainId, Gas, Nonce, UnixTimestamp, Wei,
};
use eth_valkyoth_protocol::{ForkActivation, ForkSpec, Hardfork, ValidationContext};

use crate::{
    AccessStatus, AccessTracker, BeginChildError, BlockExecutionContext, BorrowedTransactionArena,
    ChildDecision, ChildFinalizeAction, ChildLifecycleError, ClassifiedEnvelope, CryptoProvider,
    ExecutionEnvironment, ExecutionHost, ExecutionRequest, HostCapabilityError, Inspector,
    InspectorEvent, IterativeCallFrame, SnapshotAccount, SnapshotError, StateJournal,
    StateSnapshot, TransactionArena, test_fixtures::legacy_transaction,
};

#[derive(Clone, Copy, Debug)]
struct JournalCheckpoint {
    id: usize,
    value: Option<B256>,
}

#[derive(Default)]
struct TestJournal {
    value: Option<B256>,
    fail_commit: bool,
    fail_revert: bool,
    revert_attempts: usize,
    next_checkpoint: usize,
    finalized: [usize; 8],
    finalized_len: usize,
}

impl TestJournal {
    fn record_finalized(&mut self, id: usize) -> Result<(), HostCapabilityError> {
        let slot = self
            .finalized
            .get_mut(self.finalized_len)
            .ok_or(HostCapabilityError::JournalFailed)?;
        *slot = id;
        self.finalized_len = self.finalized_len.saturating_add(1);
        Ok(())
    }
}

impl StateJournal for TestJournal {
    type View = TestSnapshot;
    type Checkpoint = JournalCheckpoint;

    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.value = None;
        self.next_checkpoint = 0;
        self.finalized_len = 0;
        Ok(())
    }

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, HostCapabilityError> {
        self.next_checkpoint = self.next_checkpoint.saturating_add(1);
        Ok(JournalCheckpoint {
            id: self.next_checkpoint,
            value: self.value,
        })
    }

    fn commit(&mut self, checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError> {
        if self.fail_commit {
            return Err(HostCapabilityError::JournalFailed);
        }
        self.record_finalized(checkpoint.id)
    }

    fn revert(&mut self, checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError> {
        self.revert_attempts = self.revert_attempts.saturating_add(1);
        if self.fail_revert {
            return Err(HostCapabilityError::JournalFailed);
        }
        self.value = checkpoint.value;
        self.record_finalized(checkpoint.id)
    }

    fn current_storage(
        &self,
        base: &Self::View,
        address: Address,
        slot: B256,
    ) -> Result<B256, HostCapabilityError> {
        if let Some(value) = self.value {
            return Ok(value);
        }
        base.storage(address, slot)
            .map_err(|_| HostCapabilityError::StateReadFailed)
    }

    fn set_storage(
        &mut self,
        _address: Address,
        _slot: B256,
        value: B256,
    ) -> Result<(), HostCapabilityError> {
        self.value = Some(value);
        Ok(())
    }
}

#[derive(Default)]
struct TestAccess {
    address: Option<Address>,
    slot: Option<(Address, B256)>,
}

impl AccessTracker for TestAccess {
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.address = None;
        self.slot = None;
        Ok(())
    }

    fn warm_address(&mut self, address: Address) -> Result<AccessStatus, HostCapabilityError> {
        if self.address == Some(address) {
            return Ok(AccessStatus::Warm);
        }
        self.address = Some(address);
        Ok(AccessStatus::Cold)
    }

    fn warm_storage(
        &mut self,
        address: Address,
        slot: B256,
    ) -> Result<AccessStatus, HostCapabilityError> {
        if self.slot == Some((address, slot)) {
            return Ok(AccessStatus::Warm);
        }
        self.address = Some(address);
        self.slot = Some((address, slot));
        Ok(AccessStatus::Cold)
    }
}

#[derive(Default)]
struct TestInspector {
    last: Option<InspectorEvent>,
}

impl Inspector for TestInspector {
    fn observe(&mut self, event: InspectorEvent) {
        self.last = Some(event);
    }
}

struct TestSnapshot;

impl StateSnapshot for TestSnapshot {
    fn snapshot_id(&self) -> B256 {
        zero_hash()
    }

    fn account(&self, _address: Address) -> Result<Option<SnapshotAccount>, SnapshotError> {
        Ok(Some(SnapshotAccount {
            nonce: Nonce::new(0),
            balance: Wei::ZERO,
            code_hash: zero_hash(),
        }))
    }

    fn storage(&self, _address: Address, _slot: B256) -> Result<B256, SnapshotError> {
        Ok(zero_hash())
    }
}

struct TestCrypto;

impl CryptoProvider for TestCrypto {
    fn keccak256(&mut self, _input: &[u8]) -> Result<B256, HostCapabilityError> {
        Ok(zero_hash())
    }

    fn recover_address(
        &mut self,
        _digest: B256,
        _signature: &[u8; 65],
    ) -> Result<Address, HostCapabilityError> {
        Ok(zero_address())
    }
}

macro_rules! request_fixture {
    ($raw:ident, $snapshot:ident, $request:ident) => {
        let Some($raw) = legacy_transaction() else {
            return;
        };
        let $snapshot = TestSnapshot;
        let Some($request) = request_for(&$raw, &$snapshot) else {
            return;
        };
    };
}

#[test]
fn child_revert_restores_state_but_preserves_transaction_warmth() {
    request_fixture!(raw, snapshot, request);
    let address = patterned_address();
    let slot = patterned_hash(1);
    let changed = patterned_hash(2);
    let mut memory = [0u8; 64];
    let Some(mut arena) = BorrowedTransactionArena::<4>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    let mut inspector = TestInspector::default();

    {
        let mut host =
            ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
        let started = host.begin_transaction();
        assert_eq!(started, Ok(InspectorEvent::TransactionStarted));
        inspector.observe(started.unwrap_or(InspectorEvent::TransactionStarted));
        let child = host.with_child(
            IterativeCallFrame {
                address,
                is_static: false,
            },
            |host| {
                assert_eq!(host.warm_storage(address, slot), Ok(AccessStatus::Cold));
                assert_eq!(host.set_storage(address, slot, changed), Ok(()));
                ChildDecision::Revert(())
            },
        );
        assert!(child.is_ok(), "{child:?}");
        if let Ok(child) = child {
            for event in child.events() {
                inspector.observe(*event);
            }
        }
    }

    assert_eq!(journal.value, None);
    assert_eq!(access.warm_storage(address, slot), Ok(AccessStatus::Warm));
    assert_eq!(
        inspector.last,
        Some(InspectorEvent::ChildReverted { depth: 1 })
    );
    assert_eq!(snapshot.snapshot_id(), request.snapshot().snapshot_id());
}

#[test]
fn nested_child_lifecycles_finalize_in_lifo_order() {
    request_fixture!(raw, snapshot, request);
    let mut memory = [0u8; 8];
    let Some(mut arena) = BorrowedTransactionArena::<2>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    {
        let mut host =
            ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
        assert!(host.begin_transaction().is_ok());

        let outer = host.with_child(frame(patterned_address()), |host| {
            let inner = host.with_child(frame(zero_address()), |_| ChildDecision::Commit(()));
            assert!(inner.is_ok(), "{inner:?}");
            ChildDecision::Commit(())
        });
        assert!(outer.is_ok(), "{outer:?}");
    }
    assert_eq!(
        journal.finalized.get(..journal.finalized_len),
        Some(&[2, 1][..])
    );
}

#[test]
fn current_storage_is_authoritative_after_journal_write() {
    request_fixture!(raw, snapshot, request);
    let mut memory = [0u8; 8];
    let Some(mut arena) = BorrowedTransactionArena::<1>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    let mut host = ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
    assert!(host.begin_transaction().is_ok());
    let changed = patterned_hash(7);
    assert_eq!(
        host.set_storage(zero_address(), zero_hash(), changed),
        Ok(())
    );
    assert_eq!(
        host.current_storage(zero_address(), zero_hash()),
        Ok(changed)
    );
}

#[test]
fn journal_finalization_failure_poisons_host_without_popping_frame() {
    request_fixture!(raw, snapshot, request);
    let mut memory = [0u8; 8];
    let Some(mut arena) = BorrowedTransactionArena::<1>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal {
        fail_commit: true,
        ..TestJournal::default()
    };
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    let mut host = ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
    assert!(host.begin_transaction().is_ok());
    let result = host.with_child(frame(zero_address()), |_| ChildDecision::Commit(()));
    assert_eq!(
        result,
        Err(ChildLifecycleError::JournalConsistencyUnknown {
            action: ChildFinalizeAction::Commit,
            error: HostCapabilityError::JournalFailed,
        })
    );
    assert!(host.is_poisoned());
    assert_eq!(host.frame_depth(), 1);
    assert_eq!(
        host.warm_address(zero_address()),
        Err(HostCapabilityError::HostPoisoned)
    );
}

#[test]
fn rejected_frame_preserves_cleanup_failure_and_poisons_host() {
    request_fixture!(raw, snapshot, request);
    let mut arena = RejectingArena;
    let mut journal = TestJournal {
        fail_revert: true,
        ..TestJournal::default()
    };
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    let mut host = ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
    assert!(host.begin_transaction().is_ok());
    let result = host.with_child(frame(zero_address()), |_| ChildDecision::Commit(()));
    assert_eq!(
        result,
        Err(ChildLifecycleError::Begin(
            BeginChildError::FrameRejectedAndJournalRevertFailed {
                frame_error: HostCapabilityError::CallDepthExceeded,
                revert_error: HostCapabilityError::JournalFailed,
            }
        ))
    );
    assert!(host.is_poisoned());
}

#[test]
fn inspector_dispatch_occurs_only_after_child_finalization() {
    request_fixture!(raw, snapshot, request);
    let mut memory = [0u8; 8];
    let Some(mut arena) = BorrowedTransactionArena::<1>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    let mut inspector = TestInspector::default();
    let mut host = ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
    assert!(host.begin_transaction().is_ok());
    let child = host.with_child(frame(zero_address()), |_| ChildDecision::Commit(()));
    assert!(child.is_ok(), "{child:?}");
    assert_eq!(inspector.last, None);
    if let Ok(child) = child {
        assert_eq!(host.frame_depth(), 0);
        for event in child.events() {
            inspector.observe(*event);
        }
    }
    assert_eq!(
        inspector.last,
        Some(InspectorEvent::ChildCommitted { depth: 1 })
    );
}

struct RejectingArena;

impl TransactionArena for RejectingArena {
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        Ok(())
    }

    fn reserve_memory(&mut self, _required_len: usize) -> Result<(), HostCapabilityError> {
        Ok(())
    }

    fn enter_frame(&mut self, _frame: IterativeCallFrame) -> Result<usize, HostCapabilityError> {
        Err(HostCapabilityError::CallDepthExceeded)
    }

    fn leave_frame(&mut self) -> Result<IterativeCallFrame, HostCapabilityError> {
        Err(HostCapabilityError::CallFrameMissing)
    }

    fn frame_depth(&self) -> usize {
        0
    }
}

fn request_for<'a>(
    raw: &'a [u8],
    snapshot: &'a TestSnapshot,
) -> Option<ExecutionRequest<'a, TestSnapshot>> {
    let ready = ClassifiedEnvelope::decode(raw, policy())
        .ok()?
        .try_into_canonical()
        .ok()?
        .try_into_fork_validated(environment()?)
        .ok()?
        .into_execution_ready();
    Some(ExecutionRequest::new(ready, snapshot))
}

fn policy() -> DecodeSessionPolicy {
    let limits = DecodeLimits::reviewed_policy(256, 64, 8, 256, 4, 256);
    DecodeSessionPolicy::compatibility_policy(limits)
        .unwrap_or(DecodeSessionPolicy::DEPLOYMENT_STARTING_POINT)
}

fn frame(address: Address) -> IterativeCallFrame {
    IterativeCallFrame {
        address,
        is_static: false,
    }
}

fn patterned_address() -> Address {
    Address::from_bytes(core::array::from_fn(|index| {
        u8::try_from(index).unwrap_or(u8::MAX)
    }))
}

fn patterned_hash(offset: usize) -> B256 {
    B256::from_bytes(core::array::from_fn(|index| {
        u8::try_from(index.saturating_add(offset)).unwrap_or(u8::MAX)
    }))
}

fn zero_address() -> Address {
    Address::from_bytes(core::array::from_fn(|_| u8::default()))
}

fn zero_hash() -> B256 {
    B256::from_bytes(core::array::from_fn(|_| u8::default()))
}

fn environment() -> Option<ExecutionEnvironment> {
    let chain_id = ChainId::new(1);
    let block_number = BlockNumber::new(1);
    let timestamp = UnixTimestamp::new(1);
    let fork = ValidationContext {
        fork: ForkSpec {
            chain_id,
            hardfork: Hardfork::London,
            activation: ForkActivation::BlockAndTimestamp {
                activation_block: block_number,
                activation_timestamp: timestamp,
            },
        },
        block_number,
        timestamp,
    };
    let block = BlockExecutionContext {
        chain_id,
        block_number,
        timestamp,
        beneficiary: zero_address(),
        gas_limit: Gas::new(1),
        base_fee_per_gas: Wei::ZERO,
        prev_randao: zero_hash(),
    };
    ExecutionEnvironment::try_new(fork, block).ok()
}
