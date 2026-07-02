use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_primitives::Address;
extern crate std;
use std::{vec, vec::Vec};

use super::*;

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 512,
    max_list_items: 16,
    max_nesting_depth: 4,
    max_total_allocation: 512,
    max_proof_nodes: 4,
    max_total_items: 32,
};

#[test]
fn decodes_withdrawals_list() -> Result<(), &'static str> {
    let input = list(&[withdrawal(7, 11, address_bytes(), 13)]);
    let withdrawals = decode_ok(&input)?;
    let mut entries = withdrawals.entries();
    let Some(entry) = entries.next() else {
        return Err("withdrawal fixture must include one entry");
    };
    let entry = entry.map_err(|_| "withdrawal entry must decode")?;

    assert_eq!(withdrawals.encoded_rlp(), input.as_slice());
    assert_eq!(withdrawals.len(), 1);
    assert_eq!(entry.index.get(), 7);
    assert_eq!(entry.validator_index.get(), 11);
    assert_eq!(entry.address, Address::from_bytes(address_bytes()));
    assert_eq!(entry.amount.get(), 13);
    assert!(entries.next().is_none());
    Ok(())
}

#[test]
fn decodes_empty_withdrawals_list() -> Result<(), &'static str> {
    let input = list(&[]);
    let withdrawals = decode_ok(&input)?;

    assert!(withdrawals.is_empty());
    assert_eq!(withdrawals.entries().count(), 0);
    Ok(())
}

#[test]
fn rejects_wrong_withdrawal_field_count() -> Result<(), &'static str> {
    let input = list(&[list(&[uint(1), uint(2), scalar(&address_bytes())])]);
    let error = decode_err(&input)?;

    assert_eq!(
        error,
        WithdrawalDecodeError::WrongFieldCount {
            expected: WITHDRAWAL_FIELD_COUNT,
            found: 3
        }
    );
    assert_eq!(error.code(), "ETH_WITHDRAWAL_WRONG_FIELD_COUNT");
    assert_eq!(
        error.category(),
        WithdrawalDecodeErrorCategory::MalformedInput
    );
    Ok(())
}

#[test]
fn rejects_invalid_address_length() -> Result<(), &'static str> {
    let input = list(&[withdrawal(1, 2, [0x11; 19], 3)]);
    let error = decode_err(&input)?;

    assert_eq!(
        error,
        WithdrawalDecodeError::InvalidAddressLength { found: 19 }
    );
    Ok(())
}

#[test]
fn rejects_zero_amount() -> Result<(), &'static str> {
    let input = list(&[withdrawal(1, 2, address_bytes(), 0)]);
    let error = decode_err(&input)?;

    assert_eq!(error, WithdrawalDecodeError::ZeroAmount);
    assert_eq!(error.code(), "ETH_WITHDRAWAL_ZERO_AMOUNT");
    Ok(())
}

#[test]
fn rejects_noncanonical_integer_field() -> Result<(), &'static str> {
    let bad_index = scalar(&[0, 1]);
    let input = list(&[list(&[
        bad_index,
        uint(2),
        scalar(&address_bytes()),
        uint(3),
    ])]);
    let error = decode_err(&input)?;

    assert!(matches!(
        error,
        WithdrawalDecodeError::FieldDecode {
            field: WithdrawalField::Index,
            ..
        }
    ));
    Ok(())
}

#[test]
fn rejects_oversized_withdrawal_input_as_resource_exhaustion() -> Result<(), &'static str> {
    let input = list(&[withdrawal(1, 2, address_bytes(), 3)]);
    let limits = DecodeLimits {
        max_input_bytes: 4,
        ..TEST_LIMITS
    };
    let error = decode_err_with_limits(&input, limits)?;

    assert_eq!(
        error.category(),
        WithdrawalDecodeErrorCategory::ResourceExhaustion
    );
    Ok(())
}

fn withdrawal<const N: usize>(
    index: u64,
    validator_index: u64,
    address: [u8; N],
    amount: u64,
) -> Vec<u8> {
    list(&[
        uint(index),
        uint(validator_index),
        scalar(&address),
        uint(amount),
    ])
}

fn address_bytes() -> [u8; 20] {
    core::array::from_fn(|index| 0x30_u8.wrapping_add(usize_to_u8(index)))
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

fn decode_ok(input: &[u8]) -> Result<UnvalidatedWithdrawals<'_>, &'static str> {
    match decode_withdrawals(input, TEST_LIMITS) {
        Ok(withdrawals) => Ok(withdrawals),
        Err(_) => Err("withdrawal fixture should decode"),
    }
}

fn decode_err(input: &[u8]) -> Result<WithdrawalDecodeError, &'static str> {
    decode_err_with_limits(input, TEST_LIMITS)
}

fn decode_err_with_limits(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<WithdrawalDecodeError, &'static str> {
    match decode_withdrawals(input, limits) {
        Ok(_) => Err("withdrawal fixture should fail"),
        Err(error) => Ok(error),
    }
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}
