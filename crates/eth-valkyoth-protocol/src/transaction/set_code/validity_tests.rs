use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_primitives::{Address, BlockNumber, ChainId, Gas, Nonce, UnixTimestamp};
use std::vec::Vec;

use super::*;
use crate::{ForkActivation, ForkSpec};

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 1024,
    max_list_items: 32,
    max_nesting_depth: 8,
    max_total_allocation: 1024,
    max_proof_nodes: 4,
    max_total_items: 128,
};

#[test]
fn set_code_validity_accepts_reviewed_context() {
    let authorizations = [authorization_tuple(&[1], &[4])];
    let transaction = decoded_transaction(&authorizations);
    assert!(transaction.is_ok(), "{transaction:?}");
    let Ok(transaction) = transaction else {
        return;
    };
    let authorities = [authority(0)];
    let accounts = [account(Nonce::new(4), SetCodeAuthorityCode::Empty)];

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
        assert_eq!(valid.transaction().chain_id, ChainId::new(1));
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
        authorization_tuple(&[2], &[4]),
        authorization_tuple(&[1], &[4]),
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
        authorization_tuple(&[1], &[0xff; 8]),
        authorization_tuple(&[1], &[4]),
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
    let authorizations = [authorization_tuple(&[1], &[4])];
    let transaction = decoded_transaction(&authorizations);
    assert!(transaction.is_ok(), "{transaction:?}");
    let Ok(transaction) = transaction else {
        return;
    };
    let authorities = [authority(0)];
    let accounts = [account(Nonce::new(4), SetCodeAuthorityCode::Empty)];

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
            decoded_transaction_with_fee_and_gas(&[3], &[2], &[0x52, 0x08]),
            validity_context(None),
            Nonce::new(4),
            SetCodeAuthorityCode::Empty,
        ),
        Err(SetCodeTransactionValidityError::PriorityFeeExceedsMaxFee)
    );
    assert_eq!(
        validate_transaction(
            decoded_transaction_with_fee_and_gas(&[1], &[2], &[0x52, 0x08]),
            validity_context(Some(Gas::new(21_001))),
            Nonce::new(4),
            SetCodeAuthorityCode::Empty,
        ),
        Err(SetCodeTransactionValidityError::GasLimitTooLow)
    );
}

#[test]
fn set_code_validity_skips_bad_account_state() {
    let stale_nonce = validate_transaction(
        decoded_transaction(&[
            authorization_tuple(&[1], &[4]),
            authorization_tuple(&[1], &[5]),
        ]),
        validity_context(None),
        Nonce::new(5),
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
            authorization_tuple(&[1], &[4]),
            authorization_tuple(&[1], &[5]),
        ]),
        validity_context(None),
        Nonce::new(5),
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
            authorization_tuple(&[1], &[4]),
            authorization_tuple(&[1], &[4]),
        ]),
        validity_context(None),
        Nonce::new(4),
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
            authorization_tuple(&[1], &[4]),
            authorization_tuple(&[1], &[5]),
        ]),
        validity_context(None),
        Nonce::new(4),
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
    let authorizations = [authorization_tuple(&[1], &[])];
    let transaction = decoded_transaction(&authorizations);
    assert!(transaction.is_ok(), "{transaction:?}");
    let Ok(transaction) = transaction else {
        return;
    };
    let authorities = [authority(0)];
    let accounts = [account(Nonce::new(0), SetCodeAuthorityCode::Empty)];

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
    let universal = authorization_tuple(&[], &[4]);
    let expected = authorization_tuple(&[1], &[4]);

    assert!(validate_single(&[universal]).is_ok());
    assert!(validate_single(&[expected]).is_ok());
}

fn validate_single(
    authorizations: &[Vec<u8>],
) -> Result<ValidSetCodeTransaction<'static>, SetCodeTransactionValidityError> {
    validate_transaction(
        decoded_transaction(authorizations),
        validity_context(None),
        Nonce::new(4),
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
    let tx = set_code_tx(&[1], &[3], &[4], &[0x52, 0x08], authorizations);
    let leaked = Vec::leak(tx);
    crate::decode_set_code_transaction(leaked, TEST_LIMITS)
}

fn decoded_transaction_with_fee_and_gas(
    priority_fee: &[u8],
    max_fee: &[u8],
    gas_limit: &[u8],
) -> Result<UnvalidatedSetCodeTransaction<'static>, SetCodeTransactionDecodeError> {
    let auth = authorization_tuple(&[1], &[4]);
    let tx = set_code_tx(&[1], priority_fee, max_fee, gas_limit, &[auth]);
    let leaked = Vec::leak(tx);
    crate::decode_set_code_transaction(leaked, TEST_LIMITS)
}

fn validity_context(minimum_gas_limit: Option<Gas>) -> SetCodeTransactionValidationContext {
    SetCodeTransactionValidationContext {
        fork: ValidationContext {
            fork: ForkSpec {
                chain_id: ChainId::new(1),
                hardfork: Hardfork::Prague,
                activation: ForkActivation::BlockAndTimestamp {
                    activation_block: BlockNumber::new(10),
                    activation_timestamp: UnixTimestamp::new(20),
                },
            },
            block_number: BlockNumber::new(10),
            timestamp: UnixTimestamp::new(20),
        },
        minimum_gas_limit,
    }
}

fn inactive_context() -> SetCodeTransactionValidationContext {
    let mut context = validity_context(None);
    context.fork.block_number = BlockNumber::new(9);
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

fn authority_address() -> Address {
    Address::from_bytes(test_address_from_label(b"authority"))
}

fn set_code_tx(
    chain_id: &[u8],
    priority_fee: &[u8],
    max_fee: &[u8],
    gas_limit: &[u8],
    authorizations: &[Vec<u8>],
) -> Vec<u8> {
    let mut fields = Vec::new();
    push_scalar(&mut fields, chain_id);
    push_scalar(&mut fields, &[2]);
    push_scalar(&mut fields, priority_fee);
    push_scalar(&mut fields, max_fee);
    push_scalar(&mut fields, gas_limit);
    push_scalar(&mut fields, &test_address_from_label(b"set-code-to"));
    push_scalar(&mut fields, &[5]);
    push_scalar(&mut fields, &[]);
    push_list(&mut fields, &[]);

    let mut auth_list = Vec::new();
    for authorization in authorizations {
        auth_list.extend_from_slice(authorization);
    }
    push_list(&mut fields, &auth_list);

    push_scalar(&mut fields, &[1]);
    push_scalar(&mut fields, &[1]);
    push_scalar(&mut fields, &[2]);

    let mut tx = Vec::new();
    tx.push(crate::SET_CODE_TRANSACTION_TYPE);
    push_list(&mut tx, &fields);
    tx
}

fn authorization_tuple(chain_id: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut fields = Vec::new();
    push_scalar(&mut fields, chain_id);
    push_scalar(&mut fields, &test_address_from_label(b"set-code-auth"));
    push_scalar(&mut fields, nonce);
    push_scalar(&mut fields, &[1]);
    push_scalar(&mut fields, &[1]);
    push_scalar(&mut fields, &[2]);

    let mut tuple = Vec::new();
    push_list(&mut tuple, &fields);
    tuple
}

fn test_address_from_label(label: &[u8]) -> [u8; 20] {
    let mut bytes = [0_u8; 20];
    if label.is_empty() {
        return bytes;
    }
    for (index, byte) in bytes.iter_mut().enumerate() {
        let Some(label_index) = index.checked_rem(label.len()) else {
            return bytes;
        };
        let label_byte = label.get(label_index).copied().unwrap_or_default();
        *byte = label_byte.wrapping_add(u8::try_from(index).unwrap_or_default());
    }
    bytes
}

fn push_scalar(out: &mut Vec<u8>, payload: &[u8]) {
    if let Some(byte) = payload
        .first()
        .copied()
        .filter(|byte| payload.len() == 1 && *byte < 0x80)
    {
        out.push(byte);
    } else {
        push_prefixed(out, 0x80, payload);
    }
}

fn push_list(out: &mut Vec<u8>, payload: &[u8]) {
    push_prefixed(out, 0xc0, payload);
}

fn push_prefixed(out: &mut Vec<u8>, short_base: u8, payload: &[u8]) {
    if payload.len() <= 55 {
        let Ok(length) = u8::try_from(payload.len()) else {
            return;
        };
        let Some(prefix) = short_base.checked_add(length) else {
            return;
        };
        out.push(prefix);
    } else {
        let Some(prefix) = short_base.checked_add(56) else {
            return;
        };
        let Ok(length) = u8::try_from(payload.len()) else {
            return;
        };
        out.push(prefix);
        out.push(length);
    }
    out.extend_from_slice(payload);
}
