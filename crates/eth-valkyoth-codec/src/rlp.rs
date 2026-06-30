mod encode;
mod integer;
mod list;

#[cfg(test)]
mod tests;

use crate::{
    DecodeAccumulator, DecodeError, DecodeLimits, checked_len_add, require_exact_consumption,
    require_range_in_bounds,
};

pub use encode::{
    encode_decoded_integer, encode_decoded_item, encode_decoded_list, encode_decoded_scalar,
    encode_rlp_integer, encode_rlp_list_payload, encode_rlp_scalar, encoded_rlp_integer_len,
    encoded_rlp_list_len, encoded_rlp_scalar_len,
};
pub use integer::{
    MAX_RLP_U256_BYTES, RlpInteger, decode_rlp_integer, decode_rlp_integer_partial, decode_rlp_u64,
    decode_rlp_u128, decode_rlp_u256_bytes,
};
pub use list::{
    MAX_RLP_LIST_TRAVERSAL_DEPTH, RlpItem, RlpList, RlpListForm, RlpListItems, decode_rlp_list,
    decode_rlp_list_partial,
};

pub(super) const SHORT_STRING_OFFSET: u8 = 0x80;
/// Base for computing `len_of_len` in long-string encoding.
///
/// This is the last short-string prefix, not the first long-string prefix.
pub(super) const LONG_STRING_OFFSET: u8 = 0xb7;
pub(super) const SHORT_LIST_OFFSET: u8 = 0xc0;
/// Base for computing `len_of_len` in long-list encoding.
///
/// This is the last short-list prefix, not the first long-list prefix.
pub(super) const LONG_LIST_OFFSET: u8 = 0xf7;
pub(super) const SHORT_STRING_LIMIT: usize = 55;
pub(super) const LENGTH_RADIX: usize = 256;

/// Canonical RLP scalar form used by the decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RlpScalarForm {
    /// A single byte in `0x00..=0x7f`, encoded as itself.
    SingleByte,
    /// A byte string with a one-byte RLP header.
    ShortString,
    /// A byte string with a length-of-length RLP header.
    LongString,
}

/// Borrowed RLP scalar byte string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RlpScalar<'a> {
    payload: &'a [u8],
    encoded_len: usize,
    header_len: usize,
    form: RlpScalarForm,
}

impl<'a> RlpScalar<'a> {
    /// Returns the decoded scalar payload bytes.
    #[must_use]
    pub const fn payload(self) -> &'a [u8] {
        self.payload
    }

    /// Returns the total encoded item length in bytes.
    #[must_use]
    pub const fn encoded_len(self) -> usize {
        self.encoded_len
    }

    /// Returns the RLP header length in bytes.
    #[must_use]
    pub const fn header_len(self) -> usize {
        self.header_len
    }

    /// Returns the canonical scalar form that was decoded.
    #[must_use]
    pub const fn form(self) -> RlpScalarForm {
        self.form
    }

    /// Returns true when the decoded payload is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.payload.is_empty()
    }
}

/// Decodes exactly one canonical RLP scalar byte string.
pub fn decode_rlp_scalar<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<RlpScalar<'a>, DecodeError> {
    let mut accumulator = limits.accumulator();
    let scalar = decode_rlp_scalar_partial(input, &mut accumulator)?;
    require_exact_consumption(scalar.encoded_len, input.len())?;
    Ok(scalar)
}

/// Decodes one canonical RLP scalar byte string from the start of `input`.
///
/// Warning: this intentionally accepts trailing bytes. Use
/// [`decode_rlp_scalar`] when the full input must be consumed.
///
/// The input-length budget check applies to the full `input` slice, not only
/// the consumed scalar bytes. Callers that decode from a larger outer buffer
/// must pre-slice before calling this helper.
///
/// This helper does not reject trailing bytes. It is intended for nested
/// decoders that need to consume one item while sharing cumulative budgets.
pub fn decode_rlp_scalar_partial<'a>(
    input: &'a [u8],
    accumulator: &mut DecodeAccumulator,
) -> Result<RlpScalar<'a>, DecodeError> {
    accumulator.check_input_len(input.len())?;
    accumulator.account_items(1)?;

    let prefix = *input.first().ok_or(DecodeError::Malformed)?;
    match prefix {
        0x00..=0x7f => decode_single_byte(input),
        SHORT_STRING_OFFSET..=LONG_STRING_OFFSET => decode_short_string(input, prefix),
        0xb8..=0xbf => decode_long_string(input, prefix),
        SHORT_LIST_OFFSET..=0xff => Err(DecodeError::UnexpectedList),
    }
}

fn decode_single_byte(input: &[u8]) -> Result<RlpScalar<'_>, DecodeError> {
    let payload = input.get(..1).ok_or(DecodeError::OffsetOutOfBounds)?;
    Ok(RlpScalar {
        payload,
        encoded_len: 1,
        header_len: 0,
        form: RlpScalarForm::SingleByte,
    })
}

pub(super) fn decode_short_string(input: &[u8], prefix: u8) -> Result<RlpScalar<'_>, DecodeError> {
    let payload_len = usize::from(prefix.saturating_sub(SHORT_STRING_OFFSET));
    let payload = payload(input, 1, payload_len)?;
    if payload_len == 1 && payload.first().is_some_and(|byte| *byte <= 0x7f) {
        return Err(DecodeError::Malformed);
    }
    Ok(RlpScalar {
        payload,
        encoded_len: checked_len_add(1, payload_len)?,
        header_len: 1,
        form: RlpScalarForm::ShortString,
    })
}

pub(super) fn decode_long_string(input: &[u8], prefix: u8) -> Result<RlpScalar<'_>, DecodeError> {
    let len_of_len = usize::from(prefix.saturating_sub(LONG_STRING_OFFSET));
    let payload_len = parse_payload_len(input, 1, len_of_len)?;
    if payload_len <= SHORT_STRING_LIMIT {
        return Err(DecodeError::Malformed);
    }
    let header_len = checked_len_add(1, len_of_len)?;
    let payload = payload(input, header_len, payload_len)?;
    Ok(RlpScalar {
        payload,
        encoded_len: checked_len_add(header_len, payload_len)?,
        header_len,
        form: RlpScalarForm::LongString,
    })
}

pub(super) fn parse_payload_len(
    input: &[u8],
    offset: usize,
    len: usize,
) -> Result<usize, DecodeError> {
    let end = require_range_in_bounds(offset, len, input.len())?;
    let bytes = input
        .get(offset..end)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    if bytes.first().is_some_and(|byte| *byte == 0) {
        return Err(DecodeError::Malformed);
    }

    let mut value = 0usize;
    for byte in bytes {
        value = value
            .checked_mul(LENGTH_RADIX)
            .ok_or(DecodeError::LengthOverflow)?;
        value = value
            .checked_add(usize::from(*byte))
            .ok_or(DecodeError::LengthOverflow)?;
    }
    Ok(value)
}

pub(super) fn payload(input: &[u8], offset: usize, len: usize) -> Result<&[u8], DecodeError> {
    let end = require_range_in_bounds(offset, len, input.len())?;
    input.get(offset..end).ok_or(DecodeError::OffsetOutOfBounds)
}
