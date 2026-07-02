use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_primitives::{Address, B256, TransactionType};
extern crate std;
use std::{vec, vec::Vec};

use super::*;

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 2048,
    max_list_items: 32,
    max_nesting_depth: 8,
    max_total_allocation: 2048,
    max_proof_nodes: 4,
    max_total_items: 96,
};

#[test]
fn decodes_legacy_receipt_with_status_and_log() -> Result<(), &'static str> {
    let input = receipt_rlp(status_success(), logs_with_one_entry());
    let receipt = decode_ok(&input)?;
    let mut entries = receipt.logs.entries();
    let Some(log) = entries.next() else {
        return Err("receipt fixture must include one log");
    };
    let log = log.map_err(|_| "log must decode")?;

    assert_eq!(receipt.envelope, ReceiptKind::Legacy);
    assert_eq!(
        receipt.status_or_state_root,
        ReceiptStatusOrStateRoot::Success
    );
    assert_eq!(receipt.cumulative_gas_used.get(), 21_000);
    assert_eq!(receipt.logs_bloom.to_bytes(), bloom_bytes());
    assert_eq!(receipt.logs.len(), 1);
    assert_eq!(log.address, Address::from_bytes(bytes20(9)));
    assert_eq!(log.data, b"log-data");
    assert_eq!(log.topics.len(), 2);
    assert_eq!(log.topics.topics().count(), 2);
    assert!(entries.next().is_none());
    Ok(())
}

#[test]
fn decodes_typed_receipt_payload() -> Result<(), &'static str> {
    let payload = receipt_rlp(status_failure(), empty_logs());
    let mut input = Vec::new();
    input.push(0x02);
    input.extend_from_slice(&payload);
    let receipt = decode_ok(&input)?;

    assert_eq!(
        receipt.envelope,
        ReceiptKind::Typed(TransactionType::try_new_typed(0x02).map_err(|_| "type")?)
    );
    assert_eq!(
        receipt.status_or_state_root,
        ReceiptStatusOrStateRoot::Failure
    );
    assert_eq!(receipt.encoded_payload(), payload.as_slice());
    assert!(receipt.logs.is_empty());
    Ok(())
}

#[test]
fn decodes_pre_byzantium_state_root() -> Result<(), &'static str> {
    let input = receipt_rlp(scalar(&bytes32(3)), empty_logs());
    let receipt = decode_ok(&input)?;

    assert_eq!(
        receipt.status_or_state_root,
        ReceiptStatusOrStateRoot::StateRoot(B256::from_bytes(bytes32(3)))
    );
    Ok(())
}

#[test]
fn rejects_invalid_status_or_state_root() -> Result<(), &'static str> {
    let input = receipt_rlp(uint(2), empty_logs());
    let error = decode_err(&input)?;

    assert_eq!(
        error,
        ReceiptDecodeError::InvalidStatusOrStateRoot { found: 1 }
    );
    assert_eq!(error.code(), "ETH_RECEIPT_INVALID_STATUS_OR_ROOT");
    assert_eq!(error.category(), ReceiptDecodeErrorCategory::MalformedInput);
    Ok(())
}

#[test]
fn rejects_wrong_receipt_field_count() -> Result<(), &'static str> {
    let input = list(&[status_success(), uint(1)]);
    let error = decode_err(&input)?;

    assert_eq!(
        error,
        ReceiptDecodeError::WrongFieldCount {
            expected: RECEIPT_FIELD_COUNT,
            found: 2
        }
    );
    Ok(())
}

#[test]
fn rejects_malformed_log_shape() -> Result<(), &'static str> {
    let bad_log = list(&[scalar(&bytes20(9)), list(&[])]);
    let input = receipt_rlp(status_success(), list(&[bad_log]));
    let error = decode_err(&input)?;

    assert_eq!(error, ReceiptDecodeError::InvalidLogFieldCount { found: 2 });
    Ok(())
}

#[test]
fn rejects_invalid_log_topic_length() -> Result<(), &'static str> {
    let bad_topics = list(&[scalar(&bytes31(1))]);
    let bad_log = list(&[scalar(&bytes20(9)), bad_topics, scalar(b"")]);
    let input = receipt_rlp(status_success(), list(&[bad_log]));
    let error = decode_err(&input)?;

    assert_eq!(
        error,
        ReceiptDecodeError::InvalidLogTopicLength { found: 31 }
    );
    Ok(())
}

#[test]
fn rejects_oversized_receipt_input_as_resource_exhaustion() -> Result<(), &'static str> {
    let input = receipt_rlp(status_success(), empty_logs());
    let limits = DecodeLimits {
        max_input_bytes: 4,
        ..TEST_LIMITS
    };
    let error = decode_err_with_limits(&input, limits)?;

    assert_eq!(
        error.category(),
        ReceiptDecodeErrorCategory::ResourceExhaustion
    );
    Ok(())
}

fn receipt_rlp(status_or_root: Vec<u8>, logs: Vec<u8>) -> Vec<u8> {
    list(&[status_or_root, uint(21_000), scalar(&bloom_bytes()), logs])
}

fn logs_with_one_entry() -> Vec<u8> {
    let topics = list(&[scalar(&bytes32(1)), scalar(&bytes32(2))]);
    let log = list(&[scalar(&bytes20(9)), topics, scalar(b"log-data")]);
    list(&[log])
}

fn empty_logs() -> Vec<u8> {
    list(&[])
}

fn status_success() -> Vec<u8> {
    uint(1)
}

fn status_failure() -> Vec<u8> {
    scalar(&[])
}

fn bytes20(seed: u8) -> [u8; 20] {
    core::array::from_fn(|index| seed.wrapping_add(usize_to_u8(index)))
}

fn bytes31(seed: u8) -> [u8; 31] {
    core::array::from_fn(|index| seed.wrapping_add(usize_to_u8(index)))
}

fn bytes32(seed: u8) -> [u8; 32] {
    core::array::from_fn(|index| seed.wrapping_add(usize_to_u8(index)))
}

fn bloom_bytes() -> [u8; 256] {
    core::array::from_fn(usize_to_u8)
}

fn uint(value: u64) -> Vec<u8> {
    if value == 0 {
        return scalar(&[]);
    }
    let bytes = value.to_be_bytes();
    let Some(first) = bytes.iter().position(|byte| *byte != 0) else {
        return scalar(&[]);
    };
    let Some(payload) = bytes.get(first..) else {
        return scalar(&[]);
    };
    scalar(payload)
}

fn scalar(payload: &[u8]) -> Vec<u8> {
    if let [byte] = payload
        && *byte < 0x80
    {
        return vec![*byte];
    }
    let mut output = Vec::new();
    append_header(0x80, payload.len(), &mut output);
    output.extend_from_slice(payload);
    output
}

fn list(items: &[Vec<u8>]) -> Vec<u8> {
    let payload_len = items.iter().map(Vec::len).sum();
    let mut output = Vec::new();
    append_header(0xc0, payload_len, &mut output);
    for item in items {
        output.extend_from_slice(item);
    }
    output
}

fn append_header(offset: u8, payload_len: usize, output: &mut Vec<u8>) {
    if payload_len < 56 {
        output.push(offset.saturating_add(usize_to_u8(payload_len)));
        return;
    }
    let len_bytes = payload_len.to_be_bytes();
    let Some(first) = len_bytes.iter().position(|byte| *byte != 0) else {
        return;
    };
    let len_of_len = usize_to_u8(len_bytes.len().saturating_sub(first));
    output.push(offset.saturating_add(55).saturating_add(len_of_len));
    let Some(encoded_len) = len_bytes.get(first..) else {
        return;
    };
    output.extend_from_slice(encoded_len);
}

fn decode_ok(input: &[u8]) -> Result<UnvalidatedReceipt<'_>, &'static str> {
    match decode_receipt(input, TEST_LIMITS) {
        Ok(receipt) => Ok(receipt),
        Err(_) => Err("receipt fixture should decode"),
    }
}

fn decode_err(input: &[u8]) -> Result<ReceiptDecodeError, &'static str> {
    decode_err_with_limits(input, TEST_LIMITS)
}

fn decode_err_with_limits(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<ReceiptDecodeError, &'static str> {
    match decode_receipt(input, limits) {
        Ok(_) => Err("receipt fixture should fail"),
        Err(error) => Ok(error),
    }
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}
