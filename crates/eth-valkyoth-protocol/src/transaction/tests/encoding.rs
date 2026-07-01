use super::*;
use eth_valkyoth_codec::DecodeError;

const BLOB_TX: &[u8] = &[
    0x03, 0xf8, 0x45, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x94, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x05,
    0x80, 0xc0, 0x06, 0xe1, 0xa0, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02,
];

#[test]
fn round_trips_legacy_transaction_encoding() {
    let decoded = decode_legacy_transaction(CALL_TX, TEST_LIMITS);
    assert!(decoded.is_ok());
    if let Ok(transaction) = decoded {
        let mut output = [0_u8; 128];
        assert_eq!(
            encoded_legacy_transaction_len(&transaction),
            Ok(CALL_TX.len())
        );
        let written = encode_legacy_transaction(&transaction, &mut output);
        assert_eq!(written, Ok(CALL_TX.len()));
        assert_eq!(output.get(..CALL_TX.len()), Some(CALL_TX));
    }
}

#[test]
fn round_trips_access_list_transaction_encoding() {
    let decoded = decode_access_list_transaction(ACCESS_LIST_TX, TEST_LIMITS);
    assert!(decoded.is_ok());
    if let Ok(transaction) = decoded {
        let mut output = [0_u8; 128];
        assert_eq!(
            encoded_access_list_transaction_len(&transaction),
            Ok(ACCESS_LIST_TX.len())
        );
        let written = encode_access_list_transaction(&transaction, &mut output);
        assert_eq!(written, Ok(ACCESS_LIST_TX.len()));
        assert_eq!(output.get(..ACCESS_LIST_TX.len()), Some(ACCESS_LIST_TX));
    }
}

#[test]
fn round_trips_dynamic_fee_transaction_encoding() {
    let decoded = decode_dynamic_fee_transaction(dynamic_fee::DYNAMIC_FEE_TX, TEST_LIMITS);
    assert!(decoded.is_ok());
    if let Ok(transaction) = decoded {
        let mut output = [0_u8; 128];
        assert_eq!(
            encoded_dynamic_fee_transaction_len(&transaction),
            Ok(dynamic_fee::DYNAMIC_FEE_TX.len())
        );
        let written = encode_dynamic_fee_transaction(&transaction, &mut output);
        assert_eq!(written, Ok(dynamic_fee::DYNAMIC_FEE_TX.len()));
        assert_eq!(
            output.get(..dynamic_fee::DYNAMIC_FEE_TX.len()),
            Some(dynamic_fee::DYNAMIC_FEE_TX)
        );
    }
}

#[test]
fn round_trips_blob_transaction_encoding() {
    let decoded = decode_blob_transaction(BLOB_TX, TEST_LIMITS);
    assert!(decoded.is_ok());
    if let Ok(transaction) = decoded {
        let mut output = [0_u8; 128];
        assert_eq!(
            encoded_blob_transaction_len(&transaction),
            Ok(BLOB_TX.len())
        );
        let written = encode_blob_transaction(&transaction, &mut output);
        assert_eq!(written, Ok(BLOB_TX.len()));
        assert_eq!(output.get(..BLOB_TX.len()), Some(BLOB_TX));
    }
}

#[test]
fn unified_encoder_round_trips_admitted_transaction_domain() {
    let decoded = decode_dynamic_fee_transaction(dynamic_fee::DYNAMIC_FEE_TX, TEST_LIMITS);
    assert!(decoded.is_ok());
    if let Ok(transaction) = decoded {
        let transaction = UnvalidatedTransaction::DynamicFee(transaction);
        let mut output = [0_u8; 128];
        assert_eq!(
            encoded_transaction_len(transaction),
            Ok(dynamic_fee::DYNAMIC_FEE_TX.len())
        );
        let written = encode_transaction(transaction, &mut output);
        assert_eq!(written, Ok(dynamic_fee::DYNAMIC_FEE_TX.len()));
        assert_eq!(
            output.get(..dynamic_fee::DYNAMIC_FEE_TX.len()),
            Some(dynamic_fee::DYNAMIC_FEE_TX)
        );
    }
}

#[test]
fn transaction_encoder_rejects_short_output_without_modifying_it() {
    let decoded = decode_dynamic_fee_transaction(dynamic_fee::DYNAMIC_FEE_TX, TEST_LIMITS);
    assert!(decoded.is_ok());
    if let Ok(transaction) = decoded {
        let mut output = [0xaa_u8; 1];
        assert_eq!(
            encode_dynamic_fee_transaction(&transaction, &mut output),
            Err(TransactionEncodeError::Codec(
                DecodeError::OffsetOutOfBounds
            ))
        );
        assert_eq!(output, [0xaa]);
    }
}

#[test]
fn transaction_encode_errors_have_stable_codes_and_messages() {
    let error = TransactionEncodeError::Codec(DecodeError::OffsetOutOfBounds);
    assert_eq!(error.code(), "ETH_CODEC_OFFSET_OUT_OF_BOUNDS");
    assert_eq!(error.message(), "offset or range is outside the input");
    assert_eq!(
        error.category(),
        TransactionEncodeErrorCategory::MalformedInput
    );
}
