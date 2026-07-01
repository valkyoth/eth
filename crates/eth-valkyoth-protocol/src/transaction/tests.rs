use super::*;
use eth_valkyoth_codec::{DecodeError, DecodeLimits};
use eth_valkyoth_primitives::{Address, Gas, Nonce, Wei};

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

fn last_byte(bytes: [u8; 32]) -> Option<u8> {
    bytes.last().copied()
}
