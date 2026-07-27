use eth_valkyoth_codec::{DecodeLimits, DecodeSessionPolicy};
use eth_valkyoth_primitives::{
    Address, B256, BlockNumber, ChainId, Gas, Nonce, UnixTimestamp, Wei,
};
use eth_valkyoth_protocol::{ForkActivation, ForkSpec, Hardfork, ValidationContext};

use super::{
    BlockExecutionContext, CanonicalTransaction, ClassifiedEnvelope, EvmAdapterBoundary,
    ExecutionEnvironment, ExecutionEnvironmentError, ExecutionReadyTransaction, ExecutionRequest,
    GasEstimationError, GasEstimationPolicy, GasEstimationRequest, GasEstimationStatus,
    GasEstimationTermination, MAX_GAS_ESTIMATION_ATTEMPTS, SnapshotAccount, SnapshotError,
    StateSnapshot, revm_dependency_review,
};
use crate::test_fixtures::legacy_transaction;
use std::boxed::Box;

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
fn environment_rejects_inactive_fork() {
    let mut context = validation_context();
    context.block_number = BlockNumber::new(1);
    context.timestamp = UnixTimestamp::new(1);
    let mut block = block_context();
    block.block_number = BlockNumber::new(1);
    block.timestamp = UnixTimestamp::new(1);

    assert_eq!(
        ExecutionEnvironment::try_new(context, block),
        Err(ExecutionEnvironmentError::InactiveFork)
    );
}

#[test]
fn environment_rejects_chain_mismatch() {
    let mut block = block_context();
    block.chain_id = ChainId::new(2);

    assert_eq!(
        ExecutionEnvironment::try_new(validation_context(), block),
        Err(ExecutionEnvironmentError::ChainMismatch)
    );
}

#[test]
fn environment_rejects_block_number_mismatch() {
    let mut block = block_context();
    block.block_number = BlockNumber::new(999);

    assert_eq!(
        ExecutionEnvironment::try_new(validation_context(), block),
        Err(ExecutionEnvironmentError::BlockNumberMismatch)
    );
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
fn transaction_admission_binds_raw_bytes_to_canonical_fields() {
    let environment = match ExecutionEnvironment::try_new(validation_context(), block_context()) {
        Ok(environment) => environment,
        Err(_) => return,
    };
    let Some(raw) = legacy_transaction() else {
        return;
    };
    let transaction = execution_ready(&raw, environment);
    assert!(transaction.is_some());
    let Some(transaction) = transaction else {
        return;
    };

    assert_eq!(transaction.raw(), raw);
    assert!(matches!(
        transaction.transaction(),
        CanonicalTransaction::Legacy(_)
    ));
    assert_eq!(transaction.transaction_type().get(), 0);
}

#[test]
fn request_report_binds_environment_transaction_and_snapshot() {
    let environment_result = ExecutionEnvironment::try_new(validation_context(), block_context());
    assert!(environment_result.is_ok());
    let environment = match environment_result {
        Ok(environment) => environment,
        Err(_) => return,
    };
    let Some(raw) = legacy_transaction() else {
        return;
    };
    let transaction = match execution_ready(&raw, environment) {
        Some(transaction) => transaction,
        None => return,
    };
    let snapshot = TestSnapshot {
        id: B256::from_bytes([0x22; 32]),
    };
    let transaction_hash = B256::from_bytes([0x99; 32]);

    let request = ExecutionRequest::new(transaction, &snapshot);
    let report = request.report(transaction_hash);

    assert_eq!(request.environment(), environment);
    assert_eq!(request.transaction().raw(), raw);
    assert_eq!(request.state().snapshot_id(), snapshot.id);
    assert_eq!(report.environment, environment);
    assert_eq!(report.transaction_type.get(), 0);
    assert_eq!(report.transaction_hash, transaction_hash);
    assert_eq!(report.snapshot_id, snapshot.id);
}

#[test]
fn gas_estimation_policy_rejects_unbounded_inputs() {
    assert_eq!(
        GasEstimationPolicy::try_new(
            0,
            Gas::new(21_000),
            GasEstimationTermination::BackendStepLimit {
                max_backend_steps: 1,
            },
        ),
        Err(GasEstimationError::ZeroMaxAttempts)
    );
    assert_eq!(
        GasEstimationPolicy::try_new(
            1,
            Gas::new(0),
            GasEstimationTermination::BackendStepLimit {
                max_backend_steps: 1,
            },
        ),
        Err(GasEstimationError::ZeroGasCap)
    );
    assert_eq!(
        GasEstimationPolicy::try_new(
            1,
            Gas::new(21_000),
            GasEstimationTermination::BackendStepLimit {
                max_backend_steps: 0,
            },
        ),
        Err(GasEstimationError::UnboundedTerminationPolicy)
    );
    assert_eq!(
        GasEstimationPolicy::try_new(
            1,
            Gas::new(21_000),
            GasEstimationTermination::WorkerTimeout { timeout_millis: 0 },
        ),
        Err(GasEstimationError::UnboundedTerminationPolicy)
    );
    assert_eq!(
        GasEstimationPolicy::try_new(
            1,
            Gas::new(21_000),
            GasEstimationTermination::WorkerIsolation { timeout_millis: 0 },
        ),
        Err(GasEstimationError::UnboundedTerminationPolicy)
    );
}

#[test]
fn gas_estimation_policy_rejects_limits_above_release_ceilings() {
    assert_eq!(
        GasEstimationPolicy::try_new(
            33,
            Gas::new(21_000),
            GasEstimationTermination::BackendStepLimit {
                max_backend_steps: 1,
            },
        ),
        Err(GasEstimationError::AttemptLimitTooHigh)
    );
    assert_eq!(
        GasEstimationPolicy::try_new(
            MAX_GAS_ESTIMATION_ATTEMPTS,
            Gas::new(1_000_000_001),
            GasEstimationTermination::BackendStepLimit {
                max_backend_steps: 1,
            },
        ),
        Err(GasEstimationError::GasCapTooHigh)
    );
    assert_eq!(
        GasEstimationPolicy::try_new(
            MAX_GAS_ESTIMATION_ATTEMPTS,
            Gas::new(21_000),
            GasEstimationTermination::BackendStepLimit {
                max_backend_steps: 10_000_001,
            },
        ),
        Err(GasEstimationError::TerminationLimitTooHigh)
    );
    assert_eq!(
        GasEstimationPolicy::try_new(
            MAX_GAS_ESTIMATION_ATTEMPTS,
            Gas::new(21_000),
            GasEstimationTermination::WorkerTimeout {
                timeout_millis: 30_001,
            },
        ),
        Err(GasEstimationError::TerminationLimitTooHigh)
    );
    assert_eq!(
        GasEstimationPolicy::try_new(
            MAX_GAS_ESTIMATION_ATTEMPTS,
            Gas::new(21_000),
            GasEstimationTermination::WorkerIsolation {
                timeout_millis: 30_001,
            },
        ),
        Err(GasEstimationError::TerminationLimitTooHigh)
    );
}

#[test]
fn gas_estimation_rejects_cap_above_block_limit() {
    let execution = execution_request();
    assert!(execution.is_some());
    let execution = match execution {
        Some(execution) => execution,
        None => return,
    };
    let policy = gas_policy(30_000_001);
    assert!(policy.is_some());
    let policy = match policy {
        Some(policy) => policy,
        None => return,
    };

    let rejected = GasEstimationRequest::try_new(execution, policy);
    assert!(rejected.is_err());
    if let Err(error) = rejected {
        assert_eq!(error, GasEstimationError::GasCapExceedsBlockLimit);
    }
}

#[test]
fn gas_estimation_report_enforces_attempt_and_gas_caps() {
    let execution = execution_request();
    assert!(execution.is_some());
    let execution = match execution {
        Some(execution) => execution,
        None => return,
    };
    let policy = gas_policy(50_000);
    assert!(policy.is_some());
    let policy = match policy {
        Some(policy) => policy,
        None => return,
    };
    let request_result = GasEstimationRequest::try_new(execution, policy);
    assert!(request_result.is_ok());
    let request = match request_result {
        Ok(request) => request,
        Err(_) => return,
    };
    let transaction_hash = B256::from_bytes([0x66; 32]);

    assert_eq!(
        request.report(
            transaction_hash,
            GasEstimationStatus::BackendUnavailable,
            5,
            None,
        ),
        Err(GasEstimationError::AttemptLimitExceeded)
    );
    assert_eq!(
        request.report(
            transaction_hash,
            GasEstimationStatus::Estimated,
            policy.max_attempts(),
            Some(Gas::new(50_001)),
        ),
        Err(GasEstimationError::EstimateExceedsGasCap)
    );
}

#[test]
fn gas_estimation_report_binds_policy_execution_and_result() {
    let execution = execution_request();
    assert!(execution.is_some());
    let execution = match execution {
        Some(execution) => execution,
        None => return,
    };
    let policy = GasEstimationPolicy::try_new(
        7,
        Gas::new(50_000),
        GasEstimationTermination::WorkerIsolation {
            timeout_millis: 500,
        },
    );
    assert!(policy.is_ok());
    let policy = match policy {
        Ok(policy) => policy,
        Err(_) => return,
    };
    let request_result = GasEstimationRequest::try_new(execution, policy);
    assert!(request_result.is_ok());
    let request = match request_result {
        Ok(request) => request,
        Err(_) => return,
    };
    let transaction_hash = B256::from_bytes([0x77; 32]);

    let report = request.report(
        transaction_hash,
        GasEstimationStatus::Estimated,
        3,
        Some(Gas::new(42_000)),
    );
    assert!(report.is_ok());
    let report = match report {
        Ok(report) => report,
        Err(_) => return,
    };

    assert_eq!(request.policy(), policy);
    assert!(!request.execution().transaction().raw().is_empty());
    assert_eq!(report.policy, policy);
    assert_eq!(report.status, GasEstimationStatus::Estimated);
    assert_eq!(report.attempts, 3);
    assert_eq!(report.estimated_gas, Some(Gas::new(42_000)));
    assert_eq!(report.execution.transaction_hash, transaction_hash);
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

fn execution_request() -> Option<ExecutionRequest<'static, TestSnapshot>> {
    let environment_result = ExecutionEnvironment::try_new(validation_context(), block_context());
    assert!(environment_result.is_ok());
    let environment = match environment_result {
        Ok(environment) => environment,
        Err(_) => return None,
    };
    let raw = Box::leak(legacy_transaction()?.into_boxed_slice());
    let transaction = execution_ready(raw, environment)?;
    static SNAPSHOT: TestSnapshot = TestSnapshot {
        id: B256::from_bytes([0x88; 32]),
    };

    Some(ExecutionRequest::new(transaction, &SNAPSHOT))
}

fn execution_ready(
    raw: &[u8],
    environment: ExecutionEnvironment,
) -> Option<ExecutionReadyTransaction<'_>> {
    let policy = DecodeSessionPolicy::compatibility_policy(LIMITS).ok()?;
    let classified = ClassifiedEnvelope::decode(raw, policy).ok()?;
    let canonical = classified.try_into_canonical().ok()?;
    let fork_validated = canonical.try_into_fork_validated(environment).ok()?;
    Some(fork_validated.into_execution_ready())
}

fn gas_policy(gas_cap: u64) -> Option<GasEstimationPolicy> {
    let policy = GasEstimationPolicy::try_new(
        4,
        Gas::new(gas_cap),
        GasEstimationTermination::BackendStepLimit {
            max_backend_steps: 128,
        },
    );
    assert!(policy.is_ok());
    policy.ok()
}
