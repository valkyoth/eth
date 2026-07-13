use eth_valkyoth_primitives::{Address, B256, ChainId};
use std::vec::Vec;

use super::super::typed_helpers::encode_numeric_or_fixed_bytes;
use super::dom::Json;
use super::{Eip712JsonError, Eip712JsonLimits};
use crate::{Eip712EncodeError, Eip712ValueKind};

const WORD_BYTES: usize = 32;

pub(super) fn encode_json_numeric_or_bytes(
    type_name: &str,
    value: &Json,
    out: &mut [u8; WORD_BYTES],
    _limits: Eip712JsonLimits,
) -> Result<(), Eip712JsonError> {
    if type_name.starts_with("uint") {
        encode_numeric_or_fixed_bytes(
            type_name,
            &Eip712ValueKind::Uint256(parse_u256(value)?),
            out,
        )?;
    } else if type_name.starts_with("int") {
        encode_numeric_or_fixed_bytes(
            type_name,
            &Eip712ValueKind::Int256(parse_i256(value)?),
            out,
        )?;
    } else if type_name.starts_with("bytes") {
        let bytes = parse_hex(value.as_str()?, WORD_BYTES)?;
        encode_numeric_or_fixed_bytes(type_name, &Eip712ValueKind::FixedBytes(&bytes), out)?;
    } else {
        return Err(Eip712JsonError::Encode(Eip712EncodeError::InvalidType));
    }
    Ok(())
}

pub(super) fn parse_chain_id(value: &Json) -> Result<ChainId, Eip712JsonError> {
    let chain_id = parse_u64(value)?;
    if chain_id == 0 {
        return Err(Eip712JsonError::Integer);
    }
    Ok(ChainId::new(chain_id))
}

pub(super) fn parse_address(input: &str) -> Result<Address, Eip712JsonError> {
    let bytes = parse_hex(input, 20)?;
    let bytes = <[u8; 20]>::try_from(bytes.as_slice()).map_err(|_| Eip712JsonError::Hex)?;
    Ok(Address::from_bytes(bytes))
}

pub(super) fn parse_b256(input: &str) -> Result<B256, Eip712JsonError> {
    let bytes = parse_hex(input, WORD_BYTES)?;
    let bytes = <[u8; WORD_BYTES]>::try_from(bytes.as_slice()).map_err(|_| Eip712JsonError::Hex)?;
    Ok(B256::from_bytes(bytes))
}

fn parse_u64(value: &Json) -> Result<u64, Eip712JsonError> {
    match value {
        Json::Number(number) => number.as_u64().ok_or(Eip712JsonError::Integer),
        Json::String(text) => parse_decimal_u64(text),
        _ => Err(Eip712JsonError::Shape),
    }
}

fn parse_u256(value: &Json) -> Result<[u8; WORD_BYTES], Eip712JsonError> {
    match value {
        Json::Number(number) => Ok(u64_to_word(
            number.as_u64().ok_or(Eip712JsonError::Integer)?,
        )),
        Json::String(text) if text.starts_with("0x") => parse_hex_word(text),
        Json::String(text) => parse_decimal_u256(text),
        _ => Err(Eip712JsonError::Shape),
    }
}

fn parse_i256(value: &Json) -> Result<[u8; WORD_BYTES], Eip712JsonError> {
    match value {
        Json::Number(number) => {
            if let Some(signed) = number.as_i64() {
                Ok(i64_to_word(signed))
            } else {
                number
                    .as_u64()
                    .map(u64_to_word)
                    .ok_or(Eip712JsonError::Integer)
            }
        }
        Json::String(text) if text.starts_with("0x") => parse_hex_word(text),
        Json::String(text) => parse_decimal_i256(text),
        _ => Err(Eip712JsonError::Shape),
    }
}

fn parse_decimal_u64(input: &str) -> Result<u64, Eip712JsonError> {
    reject_empty_decimal(input)?;
    let mut value = 0_u64;
    for byte in input.bytes() {
        let digit = decimal_digit(byte)?;
        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(u64::from(digit)))
            .ok_or(Eip712JsonError::Integer)?;
    }
    Ok(value)
}

fn parse_decimal_i256(input: &str) -> Result<[u8; WORD_BYTES], Eip712JsonError> {
    let negative = input.starts_with('-');
    let digits = input.strip_prefix('-').unwrap_or(input);
    let magnitude = parse_decimal_u256(digits)?;
    if !negative {
        if magnitude.first().copied().ok_or(Eip712JsonError::Integer)? & 0x80 == 0 {
            return Ok(magnitude);
        }
        return Err(Eip712JsonError::Integer);
    }
    if magnitude > i256_min_magnitude() {
        return Err(Eip712JsonError::Integer);
    }
    negate_twos_complement(magnitude)
}

fn negate_twos_complement(word: [u8; WORD_BYTES]) -> Result<[u8; WORD_BYTES], Eip712JsonError> {
    let mut out = [0_u8; WORD_BYTES];
    let mut carry = 1_u16;
    for (target, byte) in out.iter_mut().rev().zip(word.iter().rev()) {
        let sum = u16::from(!byte)
            .checked_add(carry)
            .ok_or(Eip712JsonError::Integer)?;
        *target = u8::try_from(sum & 0xff).map_err(|_| Eip712JsonError::Integer)?;
        carry = sum >> 8;
    }
    Ok(out)
}

fn i256_min_magnitude() -> [u8; WORD_BYTES] {
    let mut word = [0_u8; WORD_BYTES];
    if let Some(first) = word.first_mut() {
        *first = 0x80;
    }
    word
}

fn reject_empty_decimal(input: &str) -> Result<(), Eip712JsonError> {
    if input.is_empty() {
        Err(Eip712JsonError::Integer)
    } else {
        Ok(())
    }
}

fn parse_decimal_u256(input: &str) -> Result<[u8; WORD_BYTES], Eip712JsonError> {
    reject_empty_decimal(input)?;
    let mut word = [0_u8; WORD_BYTES];
    for byte in input.bytes() {
        let digit = decimal_digit(byte)?;
        mul_add_word(&mut word, digit)?;
    }
    Ok(word)
}

fn mul_add_word(word: &mut [u8; WORD_BYTES], add: u8) -> Result<(), Eip712JsonError> {
    let mut carry = u16::from(add);
    for byte in word.iter_mut().rev() {
        let value = u16::from(*byte)
            .checked_mul(10)
            .and_then(|current| current.checked_add(carry))
            .ok_or(Eip712JsonError::Integer)?;
        *byte = u8::try_from(value & 0xff).map_err(|_| Eip712JsonError::Integer)?;
        carry = value >> 8;
    }
    if carry == 0 {
        Ok(())
    } else {
        Err(Eip712JsonError::Integer)
    }
}

fn decimal_digit(byte: u8) -> Result<u8, Eip712JsonError> {
    if !byte.is_ascii_digit() {
        return Err(Eip712JsonError::Integer);
    }
    byte.checked_sub(b'0').ok_or(Eip712JsonError::Integer)
}

fn parse_hex_word(input: &str) -> Result<[u8; WORD_BYTES], Eip712JsonError> {
    let bytes = parse_hex(input, WORD_BYTES)?;
    let mut word = [0_u8; WORD_BYTES];
    let start = WORD_BYTES
        .checked_sub(bytes.len())
        .ok_or(Eip712JsonError::Hex)?;
    word.get_mut(start..)
        .ok_or(Eip712JsonError::Hex)?
        .copy_from_slice(&bytes);
    Ok(word)
}

fn u64_to_word(value: u64) -> [u8; WORD_BYTES] {
    let mut word = [0_u8; WORD_BYTES];
    if let Some(target) = word.get_mut(24..) {
        target.copy_from_slice(&value.to_be_bytes());
    }
    word
}

fn i64_to_word(value: i64) -> [u8; WORD_BYTES] {
    let fill = if value < 0 { u8::MAX } else { 0 };
    let mut word = [fill; WORD_BYTES];
    if let Some(target) = word.get_mut(24..) {
        target.copy_from_slice(&value.to_be_bytes());
    }
    word
}

pub(super) fn parse_hex(input: &str, max: usize) -> Result<Vec<u8>, Eip712JsonError> {
    let hex = input.strip_prefix("0x").ok_or(Eip712JsonError::Hex)?;
    if !hex.len().is_multiple_of(2) {
        return Err(Eip712JsonError::Hex);
    }
    let byte_len = hex.len().checked_div(2).ok_or(Eip712JsonError::Hex)?;
    check_len(byte_len, max)?;
    let mut bytes = Vec::with_capacity(byte_len);
    for chunk in hex.as_bytes().chunks_exact(2) {
        let high = hex_nibble(chunk.first().copied().ok_or(Eip712JsonError::Hex)?)?;
        let low = hex_nibble(chunk.get(1).copied().ok_or(Eip712JsonError::Hex)?)?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_nibble(byte: u8) -> Result<u8, Eip712JsonError> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0').ok_or(Eip712JsonError::Hex),
        b'a'..=b'f' => byte
            .checked_sub(b'a')
            .and_then(|value| value.checked_add(10))
            .ok_or(Eip712JsonError::Hex),
        b'A'..=b'F' => byte
            .checked_sub(b'A')
            .and_then(|value| value.checked_add(10))
            .ok_or(Eip712JsonError::Hex),
        _ => Err(Eip712JsonError::Hex),
    }
}

pub(super) fn check_len(len: usize, max: usize) -> Result<(), Eip712JsonError> {
    if len <= max {
        Ok(())
    } else {
        Err(Eip712JsonError::Limit)
    }
}

pub(super) fn check_depth(depth: usize, limits: Eip712JsonLimits) -> Result<(), Eip712JsonError> {
    let max = limits.max_depth.min(super::super::MAX_TYPE_DEPTH);
    if depth >= max {
        Err(Eip712JsonError::Encode(Eip712EncodeError::RecursionLimit))
    } else {
        Ok(())
    }
}
