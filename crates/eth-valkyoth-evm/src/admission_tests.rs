use eth_valkyoth_codec::{DecodeLimits, DecodeSessionPolicy};
use eth_valkyoth_primitives::{Address, B256, BlockNumber, ChainId, Gas, UnixTimestamp, Wei};
use eth_valkyoth_protocol::{ForkActivation, ForkSpec, Hardfork, ValidationContext};

use crate::{
    BlockExecutionContext, ClassifiedEnvelope, ExecutionAdmissionError, ExecutionEnvironment,
    test_fixtures::{
        access_list_transaction, blob_transaction, dynamic_fee_transaction, legacy_transaction,
        set_code_transaction,
    },
};

#[test]
fn rejects_empty_and_unknown_typed_envelopes_before_canonical_decode() {
    let empty = ClassifiedEnvelope::decode(&[0x02], policy());
    assert!(matches!(
        empty,
        Err(ExecutionAdmissionError::EmptyTypedPayload { type_byte: 2 })
    ));

    let unknown = ClassifiedEnvelope::decode(&[0x05, 0xc0], policy());
    assert!(matches!(
        unknown,
        Err(ExecutionAdmissionError::UnsupportedTransactionType { type_byte: 5 })
    ));
}

#[test]
fn malformed_payload_cannot_promote_and_returns_candidate() {
    let classified = ClassifiedEnvelope::decode(&[0x02, 0xc0], policy());
    assert!(classified.is_ok(), "{classified:?}");
    let Some(classified) = classified.ok() else {
        return;
    };
    let promoted = classified.try_into_canonical();
    assert!(promoted.is_err(), "{promoted:?}");
    let Some(failure) = promoted.err() else {
        return;
    };
    assert_eq!(
        failure.error(),
        ExecutionAdmissionError::CanonicalDecode {
            type_byte: 2,
            source_code: "ETH_DYNAMIC_FEE_TX_WRONG_FIELD_COUNT",
        }
    );
    let (candidate, _) = failure.into_parts();
    assert_eq!(candidate.raw(), &[0x02, 0xc0]);
}

#[test]
fn legacy_classification_and_canonical_reparse_share_one_ledger() {
    let Some(transaction) = legacy_transaction() else {
        return;
    };
    let classified = ClassifiedEnvelope::decode(&transaction, policy());
    assert!(classified.is_ok(), "{classified:?}");
    let Some(classified) = classified.ok() else {
        return;
    };
    let classification_work = classified.decode_session().encoded_bytes();
    assert_eq!(classification_work, transaction.len());

    let canonical = classified.try_into_canonical();
    assert!(canonical.is_ok(), "{canonical:?}");
    let Some(canonical) = canonical.ok() else {
        return;
    };
    assert!(canonical.decode_session().encoded_bytes() > classification_work);
}

#[test]
fn fork_type_matrix_fails_closed() {
    let Some(legacy) = legacy_transaction() else {
        return;
    };
    let Some(access_list) = access_list_transaction() else {
        return;
    };
    let Some(dynamic_fee) = dynamic_fee_transaction() else {
        return;
    };
    let Some(blob) = blob_transaction() else {
        return;
    };
    let Some(set_code) = set_code_transaction() else {
        return;
    };
    assert!(admit(&legacy, Hardfork::Frontier).is_ok());
    assert_type_inactive(&access_list, Hardfork::Byzantium, Hardfork::London);
    assert!(admit(&access_list, Hardfork::London).is_ok());
    assert_type_inactive(&dynamic_fee, Hardfork::Byzantium, Hardfork::London);
    assert!(admit(&dynamic_fee, Hardfork::London).is_ok());
    assert_type_inactive(&blob, Hardfork::Shanghai, Hardfork::Cancun);
    assert!(admit(&blob, Hardfork::Cancun).is_ok());
    assert_type_inactive(&set_code, Hardfork::Cancun, Hardfork::Prague);
    assert!(admit(&set_code, Hardfork::Prague).is_ok());
}

#[test]
fn signed_chain_must_match_execution_environment() {
    let Some(transaction) = dynamic_fee_transaction() else {
        return;
    };
    let canonical = canonical(&transaction);
    assert!(canonical.is_some(), "{canonical:?}");
    let Some(canonical) = canonical else {
        return;
    };
    let environment = environment(Hardfork::London, ChainId::new(2));
    assert!(environment.is_some(), "{environment:?}");
    let Some(environment) = environment else {
        return;
    };
    let validated = canonical.try_into_fork_validated(environment);
    assert!(matches!(
        validated,
        Err(ref failure)
            if failure.error()
                == ExecutionAdmissionError::ChainMismatch {
                    expected: ChainId::new(2),
                    found: ChainId::new(1),
                }
    ));
}

fn assert_type_inactive(raw: &[u8], active: Hardfork, required: Hardfork) {
    let result = admit(raw, active);
    assert!(matches!(
        result,
        Err(ExecutionAdmissionError::TransactionTypeNotActive {
            required: found,
            active: selected,
            ..
        }) if found == required && selected == active
    ));
}

fn admit(raw: &[u8], hardfork: Hardfork) -> Result<(), ExecutionAdmissionError> {
    let canonical = canonical(raw).ok_or(ExecutionAdmissionError::CanonicalDecode {
        type_byte: 0,
        source_code: "test canonical admission failed",
    })?;
    let environment =
        environment(hardfork, ChainId::new(1)).ok_or(ExecutionAdmissionError::CanonicalDecode {
            type_byte: 0,
            source_code: "test environment admission failed",
        })?;
    canonical
        .try_into_fork_validated(environment)
        .map(|validated| {
            let _ = validated.into_execution_ready();
        })
        .map_err(|failure| failure.error())
}

fn canonical(raw: &[u8]) -> Option<crate::CanonicallyDecodedTransaction<'_>> {
    ClassifiedEnvelope::decode(raw, policy())
        .ok()?
        .try_into_canonical()
        .ok()
}

fn policy() -> DecodeSessionPolicy {
    let limits = DecodeLimits::reviewed_policy(256, 64, 8, 256, 4, 256);
    DecodeSessionPolicy::compatibility_policy(limits)
        .unwrap_or(DecodeSessionPolicy::DEPLOYMENT_STARTING_POINT)
}

fn environment(hardfork: Hardfork, chain_id: ChainId) -> Option<ExecutionEnvironment> {
    let block_number = BlockNumber::new(10);
    let timestamp = UnixTimestamp::new(20);
    let context = ValidationContext {
        fork: ForkSpec {
            chain_id,
            hardfork,
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
        beneficiary: Address::from_bytes(core::array::from_fn(|_| u8::default())),
        gas_limit: Gas::new(30_000_000),
        base_fee_per_gas: Wei::ZERO,
        prev_randao: B256::from_bytes(core::array::from_fn(|_| u8::default())),
    };
    ExecutionEnvironment::try_new(context, block).ok()
}
