use super::*;
use eth_valkyoth_codec::DecodeError;
use eth_valkyoth_primitives::{Address, B256, Gas, Nonce, Wei};
use std::vec::Vec;

#[test]
fn decodes_blob_transaction_as_unvalidated() {
    let hash = [0x01; 32];
    let tx = blob_tx(&[1], &[0x11; 20], &[hash.as_slice()], &[0xc0], 1);
    let result = decode_blob_transaction(&tx, TEST_LIMITS);
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
        assert_eq!(tx.max_fee_per_blob_gas, Wei::from_u128(6));
        assert_eq!(tx.blob_versioned_hashes.len(), 1);
        assert_eq!(tx.y_parity, SignatureYParity::Odd);
        assert_eq!(last_byte(tx.r), Some(1));
        assert_eq!(last_byte(tx.s), Some(2));

        let mut hashes = named_blob_hash_items(tx.blob_versioned_hashes.hashes());
        assert_eq!(hashes.next(), Some(Ok(B256::from_bytes(hash))));
        assert_eq!(hashes.next(), None);
    }
}

#[test]
fn blob_decoder_defers_blob_count_and_hash_version_validation() {
    let unexpected_version = [0x02; 32];
    let tx = blob_tx(
        &[1],
        &[0x11; 20],
        &[unexpected_version.as_slice()],
        &[0xc0],
        1,
    );

    let result = decode_blob_transaction(&tx, TEST_LIMITS);
    assert!(result.is_ok());
    if let Ok(tx) = result {
        let mut hashes = tx.blob_versioned_hashes.hashes();
        assert_eq!(
            hashes.next(),
            Some(Ok(B256::from_bytes(unexpected_version)))
        );
    }

    let tx = blob_tx(&[1], &[0x11; 20], &[], &[0xc0], 1);
    let result = decode_blob_transaction(&tx, TEST_LIMITS);
    assert!(result.is_ok());
    if let Ok(tx) = result {
        assert!(tx.blob_versioned_hashes.is_empty());
    }
}

#[test]
fn rejects_non_blob_transaction_type() {
    assert_eq!(
        decode_blob_transaction(&[0x02, 0xc0], TEST_LIMITS),
        Err(BlobTransactionDecodeError::WrongTransactionType { type_byte: 2 })
    );
    assert_eq!(
        decode_blob_transaction(CREATE_TX, TEST_LIMITS),
        Err(BlobTransactionDecodeError::WrongTransactionType { type_byte: 0 })
    );
}

#[test]
fn rejects_wrong_blob_field_count() {
    assert_eq!(
        decode_blob_transaction(&[0x03, 0xc0], TEST_LIMITS),
        Err(BlobTransactionDecodeError::WrongFieldCount {
            expected: BLOB_TRANSACTION_FIELD_COUNT,
            found: 0,
        })
    );
}

#[test]
fn rejects_malformed_blob_payload_as_payload_field() {
    assert_eq!(
        decode_blob_transaction(&[0x03, 0x80], TEST_LIMITS),
        Err(BlobTransactionDecodeError::FieldDecode {
            field: BlobTransactionField::Payload,
            source: DecodeError::UnexpectedScalar,
        })
    );
}

#[test]
fn rejects_reserved_blob_chain_id_zero() {
    let hash = [0x01; 32];
    let tx = blob_tx(&[], &[0x11; 20], &[hash.as_slice()], &[0xc0], 1);

    assert_eq!(
        decode_blob_transaction(&tx, TEST_LIMITS),
        Err(BlobTransactionDecodeError::FieldDecode {
            field: BlobTransactionField::ChainId,
            source: DecodeError::Malformed,
        })
    );
}

#[test]
fn rejects_blob_create_target() {
    let hash = [0x01; 32];
    let tx = blob_tx(&[1], &[], &[hash.as_slice()], &[0xc0], 1);

    assert_eq!(
        decode_blob_transaction(&tx, TEST_LIMITS),
        Err(BlobTransactionDecodeError::InvalidToLength { found: 0 })
    );
}

#[test]
fn rejects_invalid_blob_to_length() {
    let hash = [0x01; 32];
    let tx = blob_tx(&[1], &[1], &[hash.as_slice()], &[0xc0], 1);

    assert_eq!(
        decode_blob_transaction(&tx, TEST_LIMITS),
        Err(BlobTransactionDecodeError::InvalidToLength { found: 1 })
    );
}

#[test]
fn rejects_invalid_blob_access_list_entry_field_count() {
    let hash = [0x01; 32];
    let tx = blob_tx(&[1], &[0x11; 20], &[hash.as_slice()], &[0xc1, 0xc0], 1);

    assert_eq!(
        decode_blob_transaction(&tx, TEST_LIMITS),
        Err(BlobTransactionDecodeError::InvalidAccessListEntryFieldCount { found: 0 })
    );
}

#[test]
fn rejects_invalid_blob_versioned_hash_length() {
    let tx = blob_tx(&[1], &[0x11; 20], &[&[1]], &[0xc0], 1);

    assert_eq!(
        decode_blob_transaction(&tx, TEST_LIMITS),
        Err(BlobTransactionDecodeError::InvalidBlobVersionedHashLength { found: 1 })
    );
}

#[test]
fn rejects_oversized_blob_versioned_hash_list() {
    let hash = [0x01; 32];
    let hashes = [&hash[..]; 17];
    let tx = blob_tx(&[1], &[0x11; 20], &hashes, &[0xc0], 1);
    let limits = DecodeLimits {
        max_input_bytes: 1024,
        max_list_items: 16,
        max_nesting_depth: 8,
        max_total_allocation: 1024,
        max_proof_nodes: 4,
        max_total_items: 128,
    };

    assert_eq!(
        decode_blob_transaction(&tx, limits),
        Err(BlobTransactionDecodeError::FieldDecode {
            field: BlobTransactionField::Payload,
            source: DecodeError::ListTooLong,
        })
    );
}

#[test]
fn rejects_invalid_blob_y_parity() {
    let hash = [0x01; 32];
    let tx = blob_tx(&[1], &[0x11; 20], &[hash.as_slice()], &[0xc0], 2);

    assert_eq!(
        decode_blob_transaction(&tx, TEST_LIMITS),
        Err(BlobTransactionDecodeError::InvalidYParity { value: 2 })
    );
}

fn blob_tx(
    chain_id: &[u8],
    to: &[u8],
    blob_hashes: &[&[u8]],
    access_list_rlp: &[u8],
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
    fields.extend_from_slice(access_list_rlp);
    push_scalar(&mut fields, &[6]);

    let mut hashes = Vec::new();
    for hash in blob_hashes {
        push_scalar(&mut hashes, hash);
    }
    push_list(&mut fields, &hashes);

    push_scalar(&mut fields, &[y_parity]);
    push_scalar(&mut fields, &[1]);
    push_scalar(&mut fields, &[2]);

    let mut tx = Vec::new();
    tx.push(0x03);
    push_list(&mut tx, &fields);
    tx
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
    } else if payload.len() <= 0xff {
        let Some(prefix) = short_base.checked_add(56) else {
            return;
        };
        let Ok(length) = u8::try_from(payload.len()) else {
            return;
        };
        out.push(prefix);
        out.push(length);
    } else {
        let Some(prefix) = short_base.checked_add(57) else {
            return;
        };
        let Ok(length) = u16::try_from(payload.len()) else {
            return;
        };
        out.push(prefix);
        out.extend_from_slice(&length.to_be_bytes());
    }
    out.extend_from_slice(payload);
}

fn named_blob_hash_items<'a>(
    items: crate::BlobVersionedHashItems<'a>,
) -> crate::BlobVersionedHashItems<'a> {
    items
}
