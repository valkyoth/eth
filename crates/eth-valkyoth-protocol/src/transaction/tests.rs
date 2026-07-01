use super::*;
use eth_valkyoth_codec::{DecodeError, DecodeLimits};
use eth_valkyoth_primitives::{Address, B256, Gas, Nonce, Wei};

mod dynamic_fee;

extern crate std;
use std::string::ToString;

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 128,
    max_list_items: 16,
    max_nesting_depth: 8,
    max_total_allocation: 128,
    max_proof_nodes: 4,
    max_total_items: 32,
};

const CREATE_TX: &[u8] = &[
    0xcb, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0x1b, 0x01, 0x02,
];
const CALL_TX: &[u8] = &[
    0xdf, 0x01, 0x02, 0x82, 0x52, 0x08, 0x94, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x80, 0x80, 0x1b, 0x01, 0x02,
];
const ACCESS_LIST_TX: &[u8] = &[
    0x01, 0xf8, 0x46, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0xf8, 0x38, 0xf7, 0x94,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0xe1, 0xa0, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
    0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
    0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x01, 0x01, 0x02,
];
#[test]
fn decodes_create_legacy_transaction_as_unvalidated() {
    let result = decode_legacy_transaction(CREATE_TX, TEST_LIMITS);
    assert!(result.is_ok());

    if let Ok(tx) = result {
        assert_eq!(tx.nonce, Nonce::new(1));
        assert_eq!(tx.gas_price, Wei::from_u128(2));
        assert_eq!(tx.gas_limit, Gas::new(21_000));
        assert_eq!(tx.to, LegacyTransactionTo::Create);
        assert_eq!(tx.value, Wei::ZERO);
        assert_eq!(tx.input, &[]);
        assert_eq!(last_byte(tx.v), Some(27));
        assert_eq!(last_byte(tx.r), Some(1));
        assert_eq!(last_byte(tx.s), Some(2));
        assert_eq!(tx.eip155_chain_id(), None);
    }
}

#[test]
fn eip155_chain_id_recovers_without_subtraction_panic() {
    let tx = [
        0xcb, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0x25, 0x01, 0x02,
    ];
    let result = decode_legacy_transaction(&tx, TEST_LIMITS);
    assert!(result.is_ok());

    if let Ok(tx) = result {
        assert_eq!(tx.eip155_chain_id().map(|chain_id| chain_id.get()), Some(1));
    }
}

#[test]
fn eip155_chain_id_rejects_reserved_zero_chain_id() {
    for v in [35_u8, 36_u8] {
        let tx = [
            0xcb, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, v, 0x01, 0x02,
        ];
        let result = decode_legacy_transaction(&tx, TEST_LIMITS);
        assert!(result.is_ok());

        if let Ok(tx) = result {
            assert_eq!(tx.eip155_chain_id(), None);
        }
    }
}

#[test]
fn eip155_chain_id_ignores_oversized_v() {
    let tx = [
        0xd4, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0x89, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x23, 0x01, 0x02,
    ];
    let result = decode_legacy_transaction(&tx, TEST_LIMITS);
    assert!(result.is_ok());

    if let Ok(tx) = result {
        assert_eq!(tx.eip155_chain_id(), None);
    }
}

#[test]
fn decodes_call_legacy_transaction_address() {
    let result = decode_legacy_transaction(CALL_TX, TEST_LIMITS);
    assert!(result.is_ok());

    if let Ok(tx) = result {
        assert_eq!(
            tx.to,
            LegacyTransactionTo::Call(Address::from_bytes([0x11; 20]))
        );
    }
}

#[test]
fn rejects_typed_envelope_for_legacy_decoder() {
    assert_eq!(
        decode_legacy_transaction(&[0x02, 0xc0], TEST_LIMITS),
        Err(LegacyTransactionDecodeError::TypedEnvelope { type_byte: 2 })
    );
}

#[test]
fn rejects_wrong_legacy_field_count() {
    assert_eq!(
        decode_legacy_transaction(&[0xc0], TEST_LIMITS),
        Err(LegacyTransactionDecodeError::WrongFieldCount {
            expected: LEGACY_TRANSACTION_FIELD_COUNT,
            found: 0,
        })
    );
}

#[test]
fn rejects_noncanonical_integer_field() {
    let tx = [
        0xcc, 0x81, 0x00, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0x1b, 0x01, 0x02,
    ];

    assert_eq!(
        decode_legacy_transaction(&tx, TEST_LIMITS),
        Err(LegacyTransactionDecodeError::Envelope(
            TransactionEnvelopeError::Decode(DecodeError::Malformed)
        ))
    );
}

#[test]
fn rejects_invalid_to_length() {
    let tx = [
        0xcb, 0x01, 0x02, 0x82, 0x52, 0x08, 0x01, 0x80, 0x80, 0x1b, 0x01, 0x02,
    ];

    assert_eq!(
        decode_legacy_transaction(&tx, TEST_LIMITS),
        Err(LegacyTransactionDecodeError::InvalidToLength { found: 1 })
    );
}

#[test]
fn rejects_nested_field_where_scalar_is_required() {
    let tx = [
        0xcb, 0xc0, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0x1b, 0x01, 0x02,
    ];

    assert_eq!(
        decode_legacy_transaction(&tx, TEST_LIMITS),
        Err(LegacyTransactionDecodeError::FieldDecode {
            field: LegacyTransactionField::Nonce,
            source: DecodeError::UnexpectedList,
        })
    );
}

#[test]
fn bounds_input_field_by_allocation_limit() {
    let tx = [
        0xcb, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x01, 0x1b, 0x01, 0x02,
    ];
    let limits = DecodeLimits {
        max_total_allocation: 0,
        ..TEST_LIMITS
    };

    assert_eq!(
        decode_legacy_transaction(&tx, limits),
        Err(LegacyTransactionDecodeError::FieldDecode {
            field: LegacyTransactionField::Input,
            source: DecodeError::AllocationExceeded,
        })
    );
}

#[test]
fn rejects_oversized_signature_field() {
    let tx = [
        0xec, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0xa1, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02,
    ];

    assert_eq!(
        decode_legacy_transaction(&tx, TEST_LIMITS),
        Err(LegacyTransactionDecodeError::FieldDecode {
            field: LegacyTransactionField::V,
            source: DecodeError::LengthOverflow,
        })
    );
}

#[test]
fn legacy_errors_have_stable_codes_and_messages() {
    let error = LegacyTransactionDecodeError::TypedEnvelope { type_byte: 2 };

    assert_eq!(error.code(), "ETH_LEGACY_TX_TYPED_ENVELOPE");
    assert_eq!(
        error.message(),
        "legacy transaction decoder received a typed envelope"
    );
    assert_eq!(
        error.category(),
        LegacyTransactionDecodeErrorCategory::WrongType
    );
    assert_eq!(
        error.to_string(),
        "legacy transaction decoder received a typed envelope"
    );
}

#[test]
fn decodes_access_list_transaction_as_unvalidated() {
    let result = decode_access_list_transaction(ACCESS_LIST_TX, TEST_LIMITS);
    assert!(result.is_ok());

    if let Ok(tx) = result {
        assert_eq!(tx.chain_id.get(), 1);
        assert_eq!(tx.nonce, Nonce::new(2));
        assert_eq!(tx.gas_price, Wei::from_u128(3));
        assert_eq!(tx.gas_limit, Gas::new(21_000));
        assert_eq!(tx.to, AccessListTransactionTo::Create);
        assert_eq!(tx.value, Wei::ZERO);
        assert_eq!(tx.input, &[]);
        assert_eq!(tx.access_list.address_count(), 1);
        assert_eq!(tx.access_list.storage_key_count(), 1);
        assert_eq!(tx.y_parity, SignatureYParity::Odd);
        assert_eq!(last_byte(tx.r), Some(1));
        assert_eq!(last_byte(tx.s), Some(2));

        let mut entries = named_access_list_entries(tx.access_list.entries());
        let first = entries.next();
        assert!(matches!(first, Some(Ok(_))));
        if let Some(Ok(entry)) = first {
            assert_eq!(entry.address, Address::from_bytes([0x11; 20]));
            assert_eq!(entry.storage_keys.len(), 1);

            let mut keys = named_storage_key_items(entry.storage_keys.keys());
            assert_eq!(keys.next(), Some(Ok(B256::from_bytes([0x22; 32]))));
            assert_eq!(keys.next(), None);
        }
        assert_eq!(entries.next(), None);
    }
}

#[test]
fn rejects_non_access_list_transaction_type() {
    assert_eq!(
        decode_access_list_transaction(&[0x02, 0xc0], TEST_LIMITS),
        Err(AccessListTransactionDecodeError::WrongTransactionType { type_byte: 2 })
    );
    assert_eq!(
        decode_access_list_transaction(CREATE_TX, TEST_LIMITS),
        Err(AccessListTransactionDecodeError::WrongTransactionType { type_byte: 0 })
    );
}

#[test]
fn rejects_wrong_access_list_field_count() {
    assert_eq!(
        decode_access_list_transaction(&[0x01, 0xc0], TEST_LIMITS),
        Err(AccessListTransactionDecodeError::WrongFieldCount {
            expected: ACCESS_LIST_TRANSACTION_FIELD_COUNT,
            found: 0,
        })
    );
}

#[test]
fn rejects_malformed_access_list_payload_as_payload_field() {
    assert_eq!(
        decode_access_list_transaction(&[0x01, 0x80], TEST_LIMITS),
        Err(AccessListTransactionDecodeError::FieldDecode {
            field: AccessListTransactionField::Payload,
            source: DecodeError::UnexpectedScalar,
        })
    );
}

#[test]
fn rejects_reserved_access_list_chain_id_zero() {
    let tx = [
        0x01, 0xcd, 0x80, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0xc0, 0x01, 0x01, 0x02,
    ];

    assert_eq!(
        decode_access_list_transaction(&tx, TEST_LIMITS),
        Err(AccessListTransactionDecodeError::FieldDecode {
            field: AccessListTransactionField::ChainId,
            source: DecodeError::Malformed,
        })
    );
}

#[test]
fn rejects_invalid_access_list_y_parity() {
    let tx = [
        0x01, 0xcd, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0xc0, 0x02, 0x01, 0x02,
    ];

    assert_eq!(
        decode_access_list_transaction(&tx, TEST_LIMITS),
        Err(AccessListTransactionDecodeError::InvalidYParity { value: 2 })
    );
}

#[test]
fn rejects_invalid_access_list_address_length() {
    let tx = [
        0x01, 0xd0, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0xc3, 0xc2, 0x01, 0xc0,
        0x01, 0x01, 0x02,
    ];

    assert_eq!(
        decode_access_list_transaction(&tx, TEST_LIMITS),
        Err(AccessListTransactionDecodeError::InvalidAccessListAddressLength { found: 1 })
    );
}

#[test]
fn rejects_invalid_access_list_storage_key_length() {
    let tx = [
        0x01, 0xe5, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0xd8, 0xd7, 0x94, 0x11,
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x11, 0x11, 0xc1, 0x01, 0x01, 0x01, 0x02,
    ];

    assert_eq!(
        decode_access_list_transaction(&tx, TEST_LIMITS),
        Err(AccessListTransactionDecodeError::InvalidStorageKeyLength { found: 1 })
    );
}

fn last_byte(bytes: [u8; 32]) -> Option<u8> {
    bytes.last().copied()
}

fn named_access_list_entries<'a>(
    entries: crate::AccessListEntries<'a>,
) -> crate::AccessListEntries<'a> {
    entries
}

fn named_storage_key_items<'a>(
    keys: crate::AccessListStorageKeyItems<'a>,
) -> crate::AccessListStorageKeyItems<'a> {
    keys
}
