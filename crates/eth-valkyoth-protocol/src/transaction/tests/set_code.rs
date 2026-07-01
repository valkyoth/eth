use super::*;
use eth_valkyoth_codec::{DecodeError, DecodeLimits};
use eth_valkyoth_primitives::{Address, Gas, Nonce, Wei};
use std::vec::Vec;

#[test]
fn decodes_set_code_transaction_as_unvalidated() {
    let tx = set_code_tx(
        &[1],
        &[0x11; 20],
        &[authorization_tuple(&[], &[0x22; 20], 9, 1)],
        1,
    );
    let result = decode_set_code_transaction(&tx, TEST_LIMITS);
    assert!(result.is_ok());

    if let Ok(tx) = result {
        assert_eq!(tx.chain_id.get(), 1);
        assert_eq!(tx.nonce, Nonce::new(2));
        assert_eq!(tx.max_priority_fee_per_gas, Wei::from_u128(3));
        assert_eq!(tx.max_fee_per_gas, Wei::from_u128(4));
        assert_eq!(tx.gas_limit, Gas::new(21_000));
        assert_eq!(tx.to, Address::from_bytes([0x11; 20]));
        assert_eq!(tx.value, Wei::from_u128(5));
        assert_eq!(tx.input, &[]);
        assert_eq!(tx.access_list.address_count(), 0);
        assert_eq!(tx.authorization_list.len(), 1);
        assert_eq!(tx.y_parity, SignatureYParity::Odd);
        assert_eq!(last_byte(tx.r), Some(1));
        assert_eq!(last_byte(tx.s), Some(2));

        let mut authorizations = named_authorization_items(tx.authorization_list.authorizations());
        let first = authorizations.next();
        assert!(matches!(first, Some(Ok(_))));
        if let Some(Ok(auth)) = first {
            assert!(auth.chain_id.is_universal());
            assert_eq!(auth.address, Address::from_bytes([0x22; 20]));
            assert_eq!(auth.nonce, Nonce::new(9));
            assert_eq!(auth.y_parity, SignatureYParity::Odd);
            assert_eq!(last_byte(auth.r), Some(1));
            assert_eq!(last_byte(auth.s), Some(2));
        }
        assert_eq!(authorizations.next(), None);
    }
}

#[test]
fn set_code_decoder_defers_empty_authorization_list_validation() {
    let authorizations: [&[u8]; 0] = [];
    let tx = set_code_tx(&[1], &[0x11; 20], &authorizations, 1);
    let result = decode_set_code_transaction(&tx, TEST_LIMITS);
    assert!(result.is_ok());

    if let Ok(tx) = result {
        assert!(tx.authorization_list.is_empty());
    }
}

#[test]
fn round_trips_set_code_transaction_encoding() {
    let tx = set_code_tx(
        &[1],
        &[0x11; 20],
        &[authorization_tuple(&[1], &[0x22; 20], 9, 1)],
        1,
    );
    let decoded = decode_set_code_transaction(&tx, TEST_LIMITS);
    assert!(decoded.is_ok(), "{decoded:?}");

    if let Ok(transaction) = decoded {
        let mut output = [0_u8; 128];
        assert_eq!(encoded_set_code_transaction_len(&transaction), Ok(tx.len()));
        let written = encode_set_code_transaction(&transaction, &mut output);
        assert_eq!(written, Ok(tx.len()));
        assert_eq!(output.get(..tx.len()), Some(tx.as_slice()));

        let unified = UnvalidatedTransaction::SetCode(transaction);
        assert_eq!(encoded_transaction_len(unified), Ok(tx.len()));
        let written = encode_transaction(unified, &mut output);
        assert_eq!(written, Ok(tx.len()));
        assert_eq!(output.get(..tx.len()), Some(tx.as_slice()));
    }
}

#[test]
fn rejects_non_set_code_transaction_type() {
    assert_eq!(
        decode_set_code_transaction(&[0x03, 0xc0], TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::WrongTransactionType { type_byte: 3 })
    );
    assert_eq!(
        decode_set_code_transaction(CREATE_TX, TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::WrongTransactionType { type_byte: 0 })
    );
}

#[test]
fn rejects_wrong_set_code_field_count() {
    assert_eq!(
        decode_set_code_transaction(&[0x04, 0xc0], TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::WrongFieldCount {
            expected: SET_CODE_TRANSACTION_FIELD_COUNT,
            found: 0,
        })
    );
}

#[test]
fn rejects_malformed_set_code_payload_as_payload_field() {
    assert_eq!(
        decode_set_code_transaction(&[0x04, 0x80], TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::FieldDecode {
            field: SetCodeTransactionField::Payload,
            source: DecodeError::UnexpectedScalar,
        })
    );
}

#[test]
fn rejects_reserved_set_code_chain_id_zero() {
    let tx = set_code_tx(
        &[],
        &[0x11; 20],
        &[authorization_tuple(&[], &[0x22; 20], 9, 1)],
        1,
    );

    assert_eq!(
        decode_set_code_transaction(&tx, TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::FieldDecode {
            field: SetCodeTransactionField::ChainId,
            source: DecodeError::Malformed,
        })
    );
}

#[test]
fn rejects_set_code_create_target() {
    let tx = set_code_tx(&[1], &[], &[authorization_tuple(&[], &[0x22; 20], 9, 1)], 1);

    assert_eq!(
        decode_set_code_transaction(&tx, TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::InvalidToLength { found: 0 })
    );
}

#[test]
fn rejects_invalid_authorization_tuple_shape() {
    let malformed_auth = [0xc0];
    let tx = set_code_tx(&[1], &[0x11; 20], &[malformed_auth.as_slice()], 1);

    assert_eq!(
        decode_set_code_transaction(&tx, TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::InvalidAuthorizationFieldCount { found: 0 })
    );
}

#[test]
fn reports_invalid_authorization_subfield() {
    let tx = set_code_tx(
        &[1],
        &[0x11; 20],
        &[authorization_tuple(&[0, 1], &[0x22; 20], 9, 1)],
        1,
    );

    assert_eq!(
        decode_set_code_transaction(&tx, TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::AuthorizationFieldDecode {
            field: SetCodeAuthorizationField::ChainId,
            source: DecodeError::Malformed,
        })
    );
}

#[test]
fn rejects_invalid_authorization_address_length() {
    let tx = set_code_tx(
        &[1],
        &[0x11; 20],
        &[authorization_tuple(&[], &[1], 9, 1)],
        1,
    );

    assert_eq!(
        decode_set_code_transaction(&tx, TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::InvalidAuthorizationAddressLength { found: 1 })
    );
}

#[test]
fn rejects_invalid_authorization_y_parity() {
    let tx = set_code_tx(
        &[1],
        &[0x11; 20],
        &[authorization_tuple(&[], &[0x22; 20], 9, 2)],
        1,
    );

    assert_eq!(
        decode_set_code_transaction(&tx, TEST_LIMITS),
        Err(SetCodeTransactionDecodeError::InvalidAuthorizationYParity { value: 2 })
    );
}

#[test]
fn rejects_oversized_authorization_list() {
    let auth = authorization_tuple(&[], &[0x22; 20], 9, 1);
    let authorizations = [auth.as_slice(); 17];
    let tx = set_code_tx(&[1], &[0x11; 20], &authorizations, 1);
    let limits = DecodeLimits {
        max_input_bytes: 1024,
        max_list_items: 13,
        max_nesting_depth: 8,
        max_total_allocation: 1024,
        max_proof_nodes: 4,
        max_total_items: 128,
    };

    assert!(decode_set_code_transaction(&tx, limits).is_err());
}

fn set_code_tx<T: AsRef<[u8]>>(
    chain_id: &[u8],
    to: &[u8],
    authorizations: &[T],
    y_parity: u8,
) -> Vec<u8> {
    let mut fields = Vec::new();
    push_scalar(&mut fields, chain_id);
    push_scalar(&mut fields, &[2]);
    push_scalar(&mut fields, &[3]);
    push_scalar(&mut fields, &[4]);
    push_scalar(&mut fields, &[0x52, 0x08]);
    push_scalar(&mut fields, to);
    push_scalar(&mut fields, &[5]);
    push_scalar(&mut fields, &[]);
    push_list(&mut fields, &[]);

    let mut auth_list = Vec::new();
    for authorization in authorizations {
        auth_list.extend_from_slice(authorization.as_ref());
    }
    push_list(&mut fields, &auth_list);

    push_scalar(&mut fields, &[y_parity]);
    push_scalar(&mut fields, &[1]);
    push_scalar(&mut fields, &[2]);

    let mut tx = Vec::new();
    tx.push(SET_CODE_TRANSACTION_TYPE);
    push_list(&mut tx, &fields);
    tx
}

fn authorization_tuple(chain_id: &[u8], address: &[u8], nonce: u8, y_parity: u8) -> Vec<u8> {
    let mut fields = Vec::new();
    push_scalar(&mut fields, chain_id);
    push_scalar(&mut fields, address);
    push_scalar(&mut fields, &[nonce]);
    push_scalar(&mut fields, &[y_parity]);
    push_scalar(&mut fields, &[1]);
    push_scalar(&mut fields, &[2]);

    let mut tuple = Vec::new();
    push_list(&mut tuple, &fields);
    tuple
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

fn named_authorization_items<'a>(
    items: crate::SetCodeAuthorizationItems<'a>,
) -> crate::SetCodeAuthorizationItems<'a> {
    items
}
