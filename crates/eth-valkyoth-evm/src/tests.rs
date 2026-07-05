use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_primitives::{
    Address, B256, BlockNumber, ChainId, Gas, Nonce, UnixTimestamp, Wei,
};
use eth_valkyoth_protocol::{
    ForkActivation, ForkSpec, Hardfork, TransactionEnvelope, ValidationContext,
};

use super::{
    BlockExecutionContext, EvmAdapterBoundary, ExecutionEnvironment, ExecutionEnvironmentError,
    ExecutionRequest, ExecutionTransaction, SnapshotAccount, SnapshotError, StateSnapshot,
    revm_dependency_review,
};

const LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 64,
    max_list_items: 16,
    max_nesting_depth: 8,
    max_total_allocation: 64,
    max_proof_nodes: 4,
    max_total_items: 32,
};

#[derive(Clone, Copy, Debug)]
struct TestSnapshot {
    id: B256,
}

impl StateSnapshot for TestSnapshot {
    fn snapshot_id(&self) -> B256 {
        self.id
    }

    fn account(&self, _address: Address) -> Result<Option<SnapshotAccount>, SnapshotError> {
        Ok(Some(SnapshotAccount {
            nonce: Nonce::new(7),
            balance: Wei::from_u128(9),
            code_hash: B256::from_bytes([0x44; 32]),
        }))
    }

    fn storage(&self, _address: Address, _slot: B256) -> Result<B256, SnapshotError> {
        Ok(B256::from_bytes([0x55; 32]))
    }
}

#[test]
fn boundary_is_explicit() {
    assert_eq!(EvmAdapterBoundary, EvmAdapterBoundary);
}

#[test]
fn revm_dependency_is_not_admitted_until_policy_passes() {
    let review = revm_dependency_review();
    assert!(!review.admitted);
}

#[test]
fn environment_rejects_mismatched_block_context() {
    let mut block = block_context();
    block.timestamp = UnixTimestamp::new(13);

    assert_eq!(
        ExecutionEnvironment::try_new(validation_context(), block),
        Err(ExecutionEnvironmentError::TimestampMismatch)
    );
}

#[test]
fn transaction_decode_binds_raw_bytes_to_envelope() {
    let decoded = ExecutionTransaction::decode(&[0x02, 0xc0], LIMITS);
    assert!(decoded.is_ok(), "{decoded:?}");
    let transaction = match decoded {
        Ok(transaction) => transaction,
        Err(_) => return,
    };

    assert_eq!(transaction.raw(), &[0x02, 0xc0]);
    assert!(matches!(
        transaction.envelope(),
        TransactionEnvelope::Typed(_)
    ));
    assert_eq!(transaction.transaction_type().get(), 2);
}

#[test]
fn request_report_binds_environment_transaction_and_snapshot() {
    let environment_result = ExecutionEnvironment::try_new(validation_context(), block_context());
    assert!(environment_result.is_ok(), "{environment_result:?}");
    let environment = match environment_result {
        Ok(environment) => environment,
        Err(_) => return,
    };
    let decoded = ExecutionTransaction::decode(&[0xc0], LIMITS);
    assert!(decoded.is_ok(), "{decoded:?}");
    let transaction = match decoded {
        Ok(transaction) => transaction,
        Err(_) => return,
    };
    let snapshot = TestSnapshot {
        id: B256::from_bytes([0x22; 32]),
    };

    let request = ExecutionRequest::new(environment, transaction, &snapshot);
    let report = request.report();

    assert_eq!(request.environment(), environment);
    assert_eq!(request.transaction().raw(), &[0xc0]);
    assert_eq!(request.snapshot().snapshot_id(), snapshot.id);
    assert_eq!(report.environment, environment);
    assert_eq!(report.transaction_type.get(), 0);
    assert_eq!(report.snapshot_id, snapshot.id);
}

fn validation_context() -> ValidationContext {
    ValidationContext {
        fork: ForkSpec {
            chain_id: ChainId::new(1),
            hardfork: Hardfork::Prague,
            activation: ForkActivation::BlockAndTimestamp {
                activation_block: BlockNumber::new(10),
                activation_timestamp: UnixTimestamp::new(20),
            },
        },
        block_number: BlockNumber::new(12),
        timestamp: UnixTimestamp::new(22),
    }
}

fn block_context() -> BlockExecutionContext {
    BlockExecutionContext {
        chain_id: ChainId::new(1),
        block_number: BlockNumber::new(12),
        timestamp: UnixTimestamp::new(22),
        beneficiary: Address::from_bytes([0x11; 20]),
        gas_limit: Gas::new(30_000_000),
        base_fee_per_gas: Wei::from_u128(1_000_000_000),
        prev_randao: B256::from_bytes([0x33; 32]),
    }
}
