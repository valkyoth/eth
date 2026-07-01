use super::*;
use eth_valkyoth_codec::DecodeError;
use eth_valkyoth_primitives::{Gas, Nonce, Wei};

pub(super) const DYNAMIC_FEE_TX: &[u8] = &[
    0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01, 0x01, 0x02,
];

#[test]
fn decodes_dynamic_fee_transaction_as_unvalidated() {
    let result = decode_dynamic_fee_transaction(DYNAMIC_FEE_TX, TEST_LIMITS);
    assert!(result.is_ok());

    if let Ok(tx) = result {
        assert_eq!(tx.chain_id.get(), 1);
        assert_eq!(tx.nonce, Nonce::new(2));
        assert_eq!(tx.max_priority_fee_per_gas, Wei::from_u128(3));
        assert_eq!(tx.max_fee_per_gas, Wei::from_u128(4));
        assert_eq!(tx.gas_limit, Gas::new(21_000));
        assert_eq!(tx.to, DynamicFeeTransactionTo::Create);
        assert_eq!(tx.value, Wei::from_u128(5));
        assert_eq!(tx.input, &[]);
        assert_eq!(tx.access_list.address_count(), 0);
        assert_eq!(tx.access_list.storage_key_count(), 0);
        assert_eq!(tx.y_parity, SignatureYParity::Odd);
        assert_eq!(last_byte(tx.r), Some(1));
        assert_eq!(last_byte(tx.s), Some(2));
    }
}

#[test]
fn dynamic_fee_decoder_defers_fee_order_validation() {
    let tx = [
        0x02, 0xce, 0x01, 0x02, 0x04, 0x03, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01, 0x01,
        0x02,
    ];

    let result = decode_dynamic_fee_transaction(&tx, TEST_LIMITS);
    assert!(result.is_ok());
    if let Ok(tx) = result {
        assert_eq!(tx.max_priority_fee_per_gas, Wei::from_u128(4));
        assert_eq!(tx.max_fee_per_gas, Wei::from_u128(3));
    }
}

#[test]
fn rejects_non_dynamic_fee_transaction_type() {
    assert_eq!(
        decode_dynamic_fee_transaction(&[0x01, 0xc0], TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::WrongTransactionType { type_byte: 1 })
    );
    assert_eq!(
        decode_dynamic_fee_transaction(CREATE_TX, TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::WrongTransactionType { type_byte: 0 })
    );
}

#[test]
fn rejects_wrong_dynamic_fee_field_count() {
    assert_eq!(
        decode_dynamic_fee_transaction(&[0x02, 0xc0], TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::WrongFieldCount {
            expected: DYNAMIC_FEE_TRANSACTION_FIELD_COUNT,
            found: 0,
        })
    );
}

#[test]
fn rejects_malformed_dynamic_fee_payload_as_payload_field() {
    assert_eq!(
        decode_dynamic_fee_transaction(&[0x02, 0x80], TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::FieldDecode {
            field: DynamicFeeTransactionField::Payload,
            source: DecodeError::UnexpectedScalar,
        })
    );
}

#[test]
fn rejects_reserved_dynamic_fee_chain_id_zero() {
    let tx = [
        0x02, 0xce, 0x80, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01, 0x01,
        0x02,
    ];

    assert_eq!(
        decode_dynamic_fee_transaction(&tx, TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::FieldDecode {
            field: DynamicFeeTransactionField::ChainId,
            source: DecodeError::Malformed,
        })
    );
}

#[test]
fn rejects_invalid_dynamic_fee_to_length() {
    let tx = [
        0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x01, 0x05, 0x80, 0xc0, 0x01, 0x01,
        0x02,
    ];

    assert_eq!(
        decode_dynamic_fee_transaction(&tx, TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::InvalidToLength { found: 1 })
    );
}

#[test]
fn rejects_invalid_dynamic_fee_access_list_entry_field_count() {
    let tx = [
        0x02, 0xcf, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc1, 0xc0, 0x01,
        0x01, 0x02,
    ];

    assert_eq!(
        decode_dynamic_fee_transaction(&tx, TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::InvalidAccessListEntryFieldCount { found: 0 })
    );
}

#[test]
fn rejects_invalid_dynamic_fee_y_parity() {
    let tx = [
        0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x02, 0x01,
        0x02,
    ];

    assert_eq!(
        decode_dynamic_fee_transaction(&tx, TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::InvalidYParity { value: 2 })
    );
}

#[test]
fn rejects_invalid_dynamic_fee_storage_key_length() {
    let tx = [
        0x02, 0xe6, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xd8, 0xd7, 0x94,
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x11, 0x11, 0x11, 0xc1, 0x01, 0x01, 0x01, 0x02,
    ];

    assert_eq!(
        decode_dynamic_fee_transaction(&tx, TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::InvalidStorageKeyLength { found: 1 })
    );
}

#[test]
fn rejects_invalid_dynamic_fee_access_list_address_length() {
    let tx = [
        0x02, 0xd1, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc3, 0xc2, 0x01,
        0xc0, 0x01, 0x01, 0x02,
    ];

    assert_eq!(
        decode_dynamic_fee_transaction(&tx, TEST_LIMITS),
        Err(DynamicFeeTransactionDecodeError::InvalidAccessListAddressLength { found: 1 })
    );
}
