use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_primitives::{BlockNumber, Gas, Nonce, UnixTimestamp};
use std::vec::Vec;

use super::*;
use crate::{ForkActivation, ForkSpec};

#[path = "validity_test_fixtures.rs"]
mod fixtures;
use fixtures::*;

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 1024,
    max_list_items: 32,
    max_nesting_depth: 8,
    max_total_allocation: 1024,
    max_proof_nodes: b"node".len(),
    max_total_items: 128,
};

#[test]
fn set_code_validity_accepts_reviewed_context() {
    let authorizations = [authorization_tuple(
        &expected_chain_id_payload(),
        &starting_nonce_payload(),
    )];
    let transaction = decoded_transaction(&authorizations);
    assert!(transaction.is_ok(), "{transaction:?}");
    let Ok(transaction) = transaction else {
        return;
    };
    let authorities = [authority(0)];
    let accounts = [account(starting_nonce(), SetCodeAuthorityCode::Empty)];

    let result = validate_set_code_transaction_context(
        transaction,
        validity_context(None),
        &authorities[..],
        &accounts[..],
    );

    assert!(result.is_ok(), "{result:?}");
    if let Ok(valid) = result {
        assert_eq!(valid.authorization_count(), 1);
        assert_eq!(valid.applied_authorization_count(), 1);
        assert_eq!(valid.skipped_authorization_count(), 0);
        assert_eq!(valid.transaction().chain_id, expected_chain_id());
    }
}

#[test]
fn set_code_validity_rejects_empty_authorization_list() {
    assert_eq!(
        validate_single(&[]),
        Err(SetCodeTransactionValidityError::EmptyAuthorizationList)
    );
}

#[test]
fn set_code_validity_skips_wrong_authorization_chain() {
    let result = validate_single(&[
        authorization_tuple(&unexpected_chain_id_payload(), &starting_nonce_payload()),
        authorization_tuple(&expected_chain_id_payload(), &starting_nonce_payload()),
    ]);

    assert!(result.is_ok(), "{result:?}");
    if let Ok(valid) = result {
        assert_eq!(valid.authorization_count(), 2);
        assert_eq!(valid.applied_authorization_count(), 1);
        assert_eq!(valid.skipped_authorization_count(), 1);
    }
}

#[test]
fn set_code_validity_skips_max_authorization_nonce() {
    let result = validate_single(&[
        authorization_tuple(&expected_chain_id_payload(), &max_nonce_payload()),
        authorization_tuple(&expected_chain_id_payload(), &starting_nonce_payload()),
    ]);

    assert!(result.is_ok(), "{result:?}");
    if let Ok(valid) = result {
        assert_eq!(valid.authorization_count(), 2);
        assert_eq!(valid.applied_authorization_count(), 1);
        assert_eq!(valid.skipped_authorization_count(), 1);
    }
}

#[test]
fn set_code_validity_rejects_inactive_or_pre_prague_fork() {
    let authorizations = [authorization_tuple(
        &expected_chain_id_payload(),
        &starting_nonce_payload(),
    )];
    let transaction = decoded_transaction(&authorizations);
    assert!(transaction.is_ok(), "{transaction:?}");
    let Ok(transaction) = transaction else {
        return;
    };
    let authorities = [authority(0)];
    let accounts = [account(starting_nonce(), SetCodeAuthorityCode::Empty)];

    assert_eq!(
        validate_set_code_transaction_context(
            transaction,
            inactive_context(),
            &authorities[..],
            &accounts[..],
        ),
        Err(SetCodeTransactionValidityError::InactiveFork)
    );
    assert_eq!(
        validate_set_code_transaction_context(
            transaction,
            pre_prague_context(),
            &authorities[..],
            &accounts[..],
        ),
        Err(SetCodeTransactionValidityError::ForkBeforePrague)
    );
}

#[test]
fn set_code_validity_rejects_bad_fee_and_gas() {
    assert_eq!(
        validate_transaction(
            decoded_transaction_with_fee_and_gas(
                &priority_fee_too_high_payload(),
                &max_fee_payload(),
                &gas_limit_payload(),
            ),
            validity_context(None),
            starting_nonce(),
            SetCodeAuthorityCode::Empty,
        ),
        Err(SetCodeTransactionValidityError::PriorityFeeExceedsMaxFee)
    );
    assert_eq!(
        validate_transaction(
            decoded_transaction_with_fee_and_gas(
                &low_priority_fee_payload(),
                &max_fee_payload(),
                &gas_limit_payload(),
            ),
            validity_context(Some(Gas::new(21_001))),
            starting_nonce(),
            SetCodeAuthorityCode::Empty,
        ),
        Err(SetCodeTransactionValidityError::GasLimitTooLow)
    );
}

#[test]
fn set_code_validity_skips_bad_account_state() {
    let stale_nonce = validate_transaction(
        decoded_transaction(&[
            authorization_tuple(&expected_chain_id_payload(), &starting_nonce_payload()),
            authorization_tuple(&expected_chain_id_payload(), &next_nonce_payload()),
        ]),
        validity_context(None),
        next_nonce(),
        SetCodeAuthorityCode::Empty,
    );
    assert!(stale_nonce.is_ok(), "{stale_nonce:?}");
    if let Ok(valid) = stale_nonce {
        assert_eq!(valid.authorization_count(), 2);
        assert_eq!(valid.applied_authorization_count(), 1);
        assert_eq!(valid.skipped_authorization_count(), 1);
    }

    let invalid_code = validate_transaction(
        decoded_transaction(&[
            authorization_tuple(&expected_chain_id_payload(), &starting_nonce_payload()),
            authorization_tuple(&expected_chain_id_payload(), &next_nonce_payload()),
        ]),
        validity_context(None),
        next_nonce(),
        SetCodeAuthorityCode::Other,
    );
    assert!(invalid_code.is_ok(), "{invalid_code:?}");
    if let Ok(valid) = invalid_code {
        assert_eq!(valid.authorization_count(), 2);
        assert_eq!(valid.applied_authorization_count(), 0);
        assert_eq!(valid.skipped_authorization_count(), 2);
    }
}

#[test]
fn set_code_validity_tracks_repeated_authority_nonce_in_order() {
    let duplicate_nonce = validate_transaction(
        decoded_transaction(&[
            authorization_tuple(&expected_chain_id_payload(), &starting_nonce_payload()),
            authorization_tuple(&expected_chain_id_payload(), &starting_nonce_payload()),
        ]),
        validity_context(None),
        starting_nonce(),
        SetCodeAuthorityCode::Empty,
    );
    assert!(duplicate_nonce.is_ok(), "{duplicate_nonce:?}");
    if let Ok(valid) = duplicate_nonce {
        assert_eq!(valid.authorization_count(), 2);
        assert_eq!(valid.applied_authorization_count(), 1);
        assert_eq!(valid.skipped_authorization_count(), 1);
    }

    let sequential_nonce = validate_transaction(
        decoded_transaction(&[
            authorization_tuple(&expected_chain_id_payload(), &starting_nonce_payload()),
            authorization_tuple(&expected_chain_id_payload(), &next_nonce_payload()),
        ]),
        validity_context(None),
        starting_nonce(),
        SetCodeAuthorityCode::Empty,
    );
    assert!(sequential_nonce.is_ok(), "{sequential_nonce:?}");
    if let Ok(valid) = sequential_nonce {
        assert_eq!(valid.authorization_count(), 2);
        assert_eq!(valid.applied_authorization_count(), 2);
        assert_eq!(valid.skipped_authorization_count(), 0);
    }
}

#[test]
fn set_code_validity_accepts_synthesized_absent_account_state() {
    let authorizations = [authorization_tuple(
        &expected_chain_id_payload(),
        &uninitialized_nonce_payload(),
    )];
    let transaction = decoded_transaction(&authorizations);
    assert!(transaction.is_ok(), "{transaction:?}");
    let Ok(transaction) = transaction else {
        return;
    };
    let authorities = [authority(0)];
    let accounts = [account(uninitialized_nonce(), SetCodeAuthorityCode::Empty)];

    let result = validate_set_code_transaction_context(
        transaction,
        validity_context(None),
        &authorities[..],
        &accounts[..],
    );

    assert!(result.is_ok(), "{result:?}");
    if let Ok(valid) = result {
        assert_eq!(valid.applied_authorization_count(), 1);
        assert_eq!(valid.skipped_authorization_count(), 0);
    }
}

#[test]
fn authorization_chain_id_matches_universal_or_expected_chain() {
    let universal = authorization_tuple(&universal_chain_id_payload(), &starting_nonce_payload());
    let expected = authorization_tuple(&expected_chain_id_payload(), &starting_nonce_payload());

    assert!(validate_single(&[universal]).is_ok());
    assert!(validate_single(&[expected]).is_ok());
}

fn validate_single(
    authorizations: &[Vec<u8>],
) -> Result<ValidSetCodeTransaction<'static>, SetCodeTransactionValidityError> {
    validate_transaction(
        decoded_transaction(authorizations),
        validity_context(None),
        starting_nonce(),
        SetCodeAuthorityCode::Empty,
    )
}

fn validate_transaction(
    transaction: Result<UnvalidatedSetCodeTransaction<'static>, SetCodeTransactionDecodeError>,
    context: SetCodeTransactionValidationContext,
    nonce: Nonce,
    code: SetCodeAuthorityCode,
) -> Result<ValidSetCodeTransaction<'static>, SetCodeTransactionValidityError> {
    assert!(transaction.is_ok(), "{transaction:?}");
    let Ok(transaction) = transaction else {
        return Err(SetCodeTransactionValidityError::EmptyAuthorizationList);
    };
    let authorities = authorities_for(transaction);
    let accounts = [account(nonce, code)];
    validate_set_code_transaction_context(transaction, context, &authorities[..], &accounts[..])
}

fn authorities_for(
    transaction: UnvalidatedSetCodeTransaction<'_>,
) -> Vec<SetCodeAuthorizationAuthority> {
    (0..transaction.authorization_list.len())
        .map(authority)
        .collect()
}

fn decoded_transaction(
    authorizations: &[Vec<u8>],
) -> Result<UnvalidatedSetCodeTransaction<'static>, SetCodeTransactionDecodeError> {
    let tx = set_code_tx(
        &expected_chain_id_payload(),
        &default_priority_fee_payload(),
        &default_max_fee_payload(),
        &gas_limit_payload(),
        authorizations,
    );
    let leaked = Vec::leak(tx);
    crate::decode_set_code_transaction(leaked, TEST_LIMITS)
}

fn decoded_transaction_with_fee_and_gas(
    priority_fee: &[u8],
    max_fee: &[u8],
    gas_limit: &[u8],
) -> Result<UnvalidatedSetCodeTransaction<'static>, SetCodeTransactionDecodeError> {
    let auth = authorization_tuple(&expected_chain_id_payload(), &starting_nonce_payload());
    let tx = set_code_tx(
        &expected_chain_id_payload(),
        priority_fee,
        max_fee,
        gas_limit,
        &[auth],
    );
    let leaked = Vec::leak(tx);
    crate::decode_set_code_transaction(leaked, TEST_LIMITS)
}

fn validity_context(minimum_gas_limit: Option<Gas>) -> SetCodeTransactionValidationContext {
    SetCodeTransactionValidationContext {
        fork: ValidationContext {
            fork: ForkSpec {
                chain_id: expected_chain_id(),
                hardfork: Hardfork::Prague,
                activation: ForkActivation::BlockAndTimestamp {
                    activation_block: BlockNumber::new(active_block_number()),
                    activation_timestamp: UnixTimestamp::new(active_timestamp()),
                },
            },
            block_number: BlockNumber::new(active_block_number()),
            timestamp: UnixTimestamp::new(active_timestamp()),
        },
        minimum_gas_limit,
    }
}

fn inactive_context() -> SetCodeTransactionValidationContext {
    let mut context = validity_context(None);
    context.fork.block_number = BlockNumber::new(inactive_block_number());
    context
}

fn pre_prague_context() -> SetCodeTransactionValidationContext {
    let mut context = validity_context(None);
    context.fork.fork.hardfork = Hardfork::Cancun;
    context
}

fn authority(authorization_index: usize) -> SetCodeAuthorizationAuthority {
    SetCodeAuthorizationAuthority {
        authorization_index,
        authority: authority_address(),
    }
}

fn account(nonce: Nonce, code: SetCodeAuthorityCode) -> SetCodeAuthorityAccount {
    SetCodeAuthorityAccount {
        authority: authority_address(),
        nonce,
        code,
    }
}
