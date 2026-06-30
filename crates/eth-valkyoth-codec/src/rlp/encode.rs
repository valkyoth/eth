use core::convert::TryFrom;

use crate::{DecodeAccumulator, DecodeError, DecodeLimits, checked_len_add};

use super::{
    LENGTH_RADIX, LONG_LIST_OFFSET, LONG_STRING_OFFSET, RlpInteger, RlpItem, RlpList, RlpScalar,
    SHORT_LIST_OFFSET, SHORT_STRING_LIMIT, SHORT_STRING_OFFSET,
    integer::validate_rlp_integer_payload, list::validate_list_payload,
};

/// Returns the encoded RLP byte length for a scalar byte-string payload.
pub fn encoded_rlp_scalar_len(payload: &[u8]) -> Result<usize, DecodeError> {
    encoded_payload_len(payload, ScalarSingleByte::Enabled)
}

/// Returns the encoded RLP byte length for a canonical integer payload.
///
/// The empty payload represents zero. Non-empty payloads must be shortest-form
/// unsigned big-endian bytes and therefore cannot start with `0x00`.
pub fn encoded_rlp_integer_len(payload: &[u8]) -> Result<usize, DecodeError> {
    validate_rlp_integer_payload(payload)?;
    encoded_rlp_scalar_len(payload)
}

/// Returns the encoded RLP byte length for a list payload.
///
/// The payload must be the concatenated encoded child items of the list.
pub fn encoded_rlp_list_len(payload: &[u8], limits: DecodeLimits) -> Result<usize, DecodeError> {
    validate_encoded_list_payload(payload, limits)?;
    encoded_payload_len(payload, ScalarSingleByte::Disabled)
}

/// Canonically encodes a scalar byte-string payload into `output`.
///
/// `output` is not modified unless this function returns `Ok`.
/// Returns the number of bytes written.
pub fn encode_rlp_scalar(payload: &[u8], output: &mut [u8]) -> Result<usize, DecodeError> {
    encode_payload(
        payload,
        output,
        SHORT_STRING_OFFSET,
        LONG_STRING_OFFSET,
        ScalarSingleByte::Enabled,
    )
}

/// Canonically encodes an Ethereum integer payload into `output`.
///
/// `output` is not modified unless this function returns `Ok`.
/// Returns the number of bytes written.
pub fn encode_rlp_integer(payload: &[u8], output: &mut [u8]) -> Result<usize, DecodeError> {
    validate_rlp_integer_payload(payload)?;
    encode_rlp_scalar(payload, output)
}

/// Canonically encodes a list payload into `output`.
///
/// The payload must already contain the concatenated encoded child items.
/// `output` is not modified unless this function returns `Ok`.
/// Returns the number of bytes written.
pub fn encode_rlp_list_payload(
    payload: &[u8],
    limits: DecodeLimits,
    output: &mut [u8],
) -> Result<usize, DecodeError> {
    validate_encoded_list_payload(payload, limits)?;
    encode_rlp_list_payload_unchecked(payload, output)
}

fn encode_rlp_list_payload_unchecked(
    payload: &[u8],
    output: &mut [u8],
) -> Result<usize, DecodeError> {
    encode_payload(
        payload,
        output,
        SHORT_LIST_OFFSET,
        LONG_LIST_OFFSET,
        ScalarSingleByte::Disabled,
    )
}

/// Re-encodes a decoded canonical scalar into `output`.
///
/// `output` is not modified unless this function returns `Ok`.
/// Returns the number of bytes written.
pub fn encode_decoded_scalar(
    scalar: RlpScalar<'_>,
    output: &mut [u8],
) -> Result<usize, DecodeError> {
    encode_rlp_scalar(scalar.payload(), output)
}

/// Re-encodes a decoded canonical integer into `output`.
///
/// `output` is not modified unless this function returns `Ok`.
/// Returns the number of bytes written.
pub fn encode_decoded_integer(
    integer: RlpInteger<'_>,
    output: &mut [u8],
) -> Result<usize, DecodeError> {
    encode_rlp_integer(integer.payload(), output)
}

/// Re-encodes a decoded canonical list into `output`.
///
/// `output` is not modified unless this function returns `Ok`.
/// Returns the number of bytes written.
pub fn encode_decoded_list(list: RlpList<'_>, output: &mut [u8]) -> Result<usize, DecodeError> {
    encode_rlp_list_payload_unchecked(list.payload(), output)
}

/// Re-encodes a decoded canonical RLP item into `output`.
///
/// `output` is not modified unless this function returns `Ok`.
/// Returns the number of bytes written.
pub fn encode_decoded_item(item: RlpItem<'_>, output: &mut [u8]) -> Result<usize, DecodeError> {
    match item {
        RlpItem::Scalar(scalar) => encode_decoded_scalar(scalar, output),
        RlpItem::List(list) => encode_decoded_list(list, output),
    }
}

#[derive(Clone, Copy)]
enum ScalarSingleByte {
    Enabled,
    Disabled,
}

fn encoded_payload_len(
    payload: &[u8],
    scalar_single_byte: ScalarSingleByte,
) -> Result<usize, DecodeError> {
    let payload_len = payload.len();
    if matches!(scalar_single_byte, ScalarSingleByte::Enabled)
        && payload_len == 1
        && payload.first().is_some_and(|byte| *byte <= 0x7f)
    {
        return Ok(1);
    }
    if payload_len <= SHORT_STRING_LIMIT {
        return checked_len_add(1, payload_len);
    }
    let len_of_len = length_of_length(payload_len)?;
    checked_len_add(checked_len_add(1, len_of_len)?, payload_len)
}

fn encode_payload(
    payload: &[u8],
    output: &mut [u8],
    short_offset: u8,
    long_offset: u8,
    scalar_single_byte: ScalarSingleByte,
) -> Result<usize, DecodeError> {
    let required_len = encoded_payload_len(payload, scalar_single_byte)?;
    if output.len() < required_len {
        return Err(DecodeError::OffsetOutOfBounds);
    }

    if matches!(scalar_single_byte, ScalarSingleByte::Enabled)
        && payload.len() == 1
        && payload.first().is_some_and(|byte| *byte <= 0x7f)
    {
        let byte = *payload.first().ok_or(DecodeError::OffsetOutOfBounds)?;
        write_byte(output, byte)?;
        return Ok(1);
    }

    let header_len = if payload.len() <= SHORT_STRING_LIMIT {
        write_short_header(output, short_offset, payload.len())?
    } else {
        write_long_header(output, long_offset, payload.len())?
    };
    write_payload(output, header_len, payload)?;
    checked_len_add(header_len, payload.len())
}

fn write_short_header(
    output: &mut [u8],
    short_offset: u8,
    payload_len: usize,
) -> Result<usize, DecodeError> {
    let payload_len = u8::try_from(payload_len).map_err(|_| DecodeError::LengthOverflow)?;
    let prefix = short_offset
        .checked_add(payload_len)
        .ok_or(DecodeError::LengthOverflow)?;
    write_byte(output, prefix)?;
    Ok(1)
}

fn write_long_header(
    output: &mut [u8],
    long_offset: u8,
    payload_len: usize,
) -> Result<usize, DecodeError> {
    let len_of_len = length_of_length(payload_len)?;
    let len_of_len_byte = u8::try_from(len_of_len).map_err(|_| DecodeError::LengthOverflow)?;
    let prefix = long_offset
        .checked_add(len_of_len_byte)
        .ok_or(DecodeError::LengthOverflow)?;
    let header_len = checked_len_add(1, len_of_len)?;
    write_byte(output, prefix)?;

    let len_bytes = payload_len.to_be_bytes();
    let start = len_bytes
        .len()
        .checked_sub(len_of_len)
        .ok_or(DecodeError::LengthOverflow)?;
    let source = len_bytes
        .get(start..)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    let target = output
        .get_mut(1..header_len)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    target.copy_from_slice(source);
    Ok(header_len)
}

fn write_byte(output: &mut [u8], byte: u8) -> Result<(), DecodeError> {
    let target = output.first_mut().ok_or(DecodeError::OffsetOutOfBounds)?;
    *target = byte;
    Ok(())
}

fn write_payload(output: &mut [u8], header_len: usize, payload: &[u8]) -> Result<(), DecodeError> {
    let end = checked_len_add(header_len, payload.len())?;
    let target = output
        .get_mut(header_len..end)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    target.copy_from_slice(payload);
    Ok(())
}

fn length_of_length(payload_len: usize) -> Result<usize, DecodeError> {
    if payload_len <= SHORT_STRING_LIMIT {
        return Err(DecodeError::Malformed);
    }

    let mut value = payload_len;
    let mut len = 0usize;
    while value > 0 {
        len = checked_len_add(len, 1)?;
        value = value
            .checked_div(LENGTH_RADIX)
            // LENGTH_RADIX is the nonzero constant 256, so this branch is
            // unreachable unless the constant changes.
            .ok_or(DecodeError::LengthOverflow)?;
    }
    Ok(len)
}

fn validate_encoded_list_payload(
    payload: &[u8],
    limits: DecodeLimits,
) -> Result<usize, DecodeError> {
    let mut accumulator: DecodeAccumulator = limits.accumulator();
    accumulator.check_input_len(payload.len())?;
    validate_list_payload(payload, &mut accumulator)
}
