use eth_valkyoth_primitives::{
    Address, B256, BlockNumber, ChainId, Gas, Nonce, UnixTimestamp, Wei,
};
use eth_valkyoth_protocol::{ForkActivation, ForkSpec, Hardfork, ValidationContext};

use crate::{
    AccessStatus, AccessTracker, BeginChildError, BlockExecutionContext, BorrowedTransactionArena,
    CryptoProvider, ExecutionEnvironment, ExecutionHost, HostCapabilityError, Inspector,
    InspectorEvent, IterativeCallFrame, SnapshotAccount, SnapshotError, StateJournal,
    StateSnapshot, TransactionArena,
};

#[derive(Clone, Copy, Debug)]
struct JournalCheckpoint {
    value: B256,
}

struct TestJournal {
    value: B256,
    fail_revert: bool,
    revert_attempts: usize,
}

impl Default for TestJournal {
    fn default() -> Self {
        Self {
            value: zero_hash(),
            fail_revert: false,
            revert_attempts: 0,
        }
    }
}

impl StateJournal for TestJournal {
    type Checkpoint = JournalCheckpoint;

    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.value = zero_hash();
        Ok(())
    }

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, HostCapabilityError> {
        Ok(JournalCheckpoint { value: self.value })
    }

    fn commit(&mut self, _checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError> {
        Ok(())
    }

    fn revert(&mut self, checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError> {
        self.revert_attempts = self.revert_attempts.saturating_add(1);
        if self.fail_revert {
            return Err(HostCapabilityError::JournalFailed);
        }
        self.value = checkpoint.value;
        Ok(())
    }

    fn set_storage(
        &mut self,
        _address: Address,
        _slot: B256,
        value: B256,
    ) -> Result<(), HostCapabilityError> {
        self.value = value;
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

#[test]
fn child_revert_restores_state_but_preserves_transaction_warmth() {
    let address = Address::from_bytes(core::array::from_fn(|index| {
        u8::try_from(index).unwrap_or(u8::MAX)
    }));
    let slot = B256::from_bytes(core::array::from_fn(|index| {
        u8::try_from(index.saturating_add(1)).unwrap_or(u8::MAX)
    }));
    let changed = B256::from_bytes(core::array::from_fn(|index| {
        u8::try_from(index.saturating_add(2)).unwrap_or(u8::MAX)
    }));
    let mut memory = [0u8; 64];
    let arena = BorrowedTransactionArena::<4>::try_new(&mut memory);
    assert!(arena.is_ok(), "{arena:?}");
    let Some(mut arena) = arena.ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess::default();
    let mut inspector = TestInspector::default();
    let snapshot = TestSnapshot;
    let environment = environment();
    assert!(environment.is_some(), "{environment:?}");
    let Some(environment) = environment else {
        return;
    };
    let mut crypto = TestCrypto;

    {
        let mut host = ExecutionHost {
            state: &snapshot,
            journal: &mut journal,
            block: &environment,
            access: &mut access,
            crypto: &mut crypto,
            inspector: &mut inspector,
            arena: &mut arena,
        };
        assert_eq!(host.begin_transaction(), Ok(()));
        let checkpoint = host.begin_child(IterativeCallFrame {
            address,
            is_static: false,
        });
        assert!(checkpoint.is_ok(), "{checkpoint:?}");
        let Some(checkpoint) = checkpoint.ok() else {
            return;
        };
        assert_eq!(
            host.access.warm_storage(address, slot),
            Ok(AccessStatus::Cold)
        );
        assert_eq!(host.journal.set_storage(address, slot, changed), Ok(()));
        assert_eq!(host.revert_child(checkpoint), Ok(()));
    }

    assert_eq!(journal.value, zero_hash());
    assert_eq!(access.warm_storage(address, slot), Ok(AccessStatus::Warm));
    assert_eq!(
        inspector.last,
        Some(InspectorEvent::ChildReverted { depth: 1 })
    );
}

#[test]
fn iterative_frames_fail_with_evm_exception_at_capacity() {
    let mut memory = [0u8; 8];
    let arena = BorrowedTransactionArena::<2>::try_new(&mut memory);
    assert!(arena.is_ok(), "{arena:?}");
    let Some(mut arena) = arena.ok() else {
        return;
    };
    let frame = IterativeCallFrame {
        address: zero_address(),
        is_static: false,
    };

    assert_eq!(arena.enter_frame(frame), Ok(1));
    assert_eq!(arena.enter_frame(frame), Ok(2));
    assert_eq!(
        arena.enter_frame(frame),
        Err(HostCapabilityError::CallDepthExceeded)
    );
}

#[test]
fn memory_expansion_and_reset_are_bounded_and_destructive() {
    let mut memory = [0xff; 8];
    let arena = BorrowedTransactionArena::<1>::try_new(&mut memory);
    assert!(arena.is_ok(), "{arena:?}");
    let Some(mut arena) = arena.ok() else {
        return;
    };

    assert_eq!(arena.reserve_memory(8), Ok(()));
    assert_eq!(arena.memory(), &[0; 8]);
    assert_eq!(
        arena.reserve_memory(9),
        Err(HostCapabilityError::MemoryCapacityExceeded)
    );
    assert_eq!(arena.reset_transaction(), Ok(()));
    assert!(arena.memory().is_empty());
}

#[test]
fn invalid_frame_profiles_fail_closed() {
    let mut memory = [0u8; 1];
    assert_eq!(
        BorrowedTransactionArena::<0>::try_new(&mut memory),
        Err(HostCapabilityError::InvalidFrameCapacity)
    );
}

#[test]
fn rejected_child_frame_preserves_primary_error_after_successful_cleanup() {
    let result = begin_child_at_capacity(false);
    assert!(result.is_some(), "{result:?}");
    let Some((error, revert_attempts)) = result else {
        return;
    };
    assert_eq!(
        error,
        BeginChildError::FrameRejected {
            frame_error: HostCapabilityError::CallDepthExceeded,
        }
    );
    assert_eq!(revert_attempts, 1);
}

#[test]
fn rejected_child_frame_reports_both_frame_and_cleanup_failures() {
    let result = begin_child_at_capacity(true);
    assert!(result.is_some(), "{result:?}");
    let Some((error, revert_attempts)) = result else {
        return;
    };
    assert_eq!(
        error,
        BeginChildError::FrameRejectedAndJournalRevertFailed {
            frame_error: HostCapabilityError::CallDepthExceeded,
            revert_error: HostCapabilityError::JournalFailed,
        }
    );
    assert_eq!(
        error.code(),
        "ETH_EVM_HOST_FRAME_REJECTED_JOURNAL_REVERT_FAILED"
    );
    assert_eq!(revert_attempts, 1);
}

fn begin_child_at_capacity(fail_revert: bool) -> Option<(BeginChildError, usize)> {
    let mut memory = [0u8; 1];
    let mut arena = BorrowedTransactionArena::<1>::try_new(&mut memory).ok()?;
    let frame = IterativeCallFrame {
        address: zero_address(),
        is_static: false,
    };
    arena.enter_frame(frame).ok()?;
    let mut journal = TestJournal {
        fail_revert,
        ..TestJournal::default()
    };
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    let mut inspector = TestInspector::default();
    let snapshot = TestSnapshot;
    let environment = environment()?;
    let error = {
        let mut host = ExecutionHost {
            state: &snapshot,
            journal: &mut journal,
            block: &environment,
            access: &mut access,
            crypto: &mut crypto,
            inspector: &mut inspector,
            arena: &mut arena,
        };
        host.begin_child(frame).err()?
    };
    Some((error, journal.revert_attempts))
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
