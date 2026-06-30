use core::convert::TryFrom;

use crate::{DecodeAccumulator, DecodeError, DecodeLimits, require_exact_consumption};

use super::{RlpScalar, RlpScalarForm, decode_rlp_scalar_partial};

/// Ethereum U256 maximum byte width in bytes.
///
/// Mirrors the primitive crate's internal `MAX_U256_BYTES`; changes to
/// Ethereum integer canonicality rules must be applied to both crates.
pub const MAX_RLP_U256_BYTES: usize = 32;

const MAX_U64_BYTES: usize = 8;
const MAX_U128_BYTES: usize = 16;
// Mirrors the primitive crate's internal integer radix. Keep both in sync.
const INTEGER_RADIX_U64: u64 = 256;
const INTEGER_RADIX_U128: u128 = 256;

/// Borrowed canonical RLP integer.
///
/// Ethereum integers are encoded as shortest-form unsigned big-endian bytes.
/// The empty payload represents zero. A non-empty payload whose first byte is
/// zero is rejected.
///
/// Fields are private so downstream crates cannot construct unvalidated
/// decoded values and feed them into trusted re-encoding paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RlpInteger<'a> {
    scalar: RlpScalar<'a>,
}

impl<'a> RlpInteger<'a> {
    /// Validates a decoded RLP scalar as a canonical integer.
    pub fn try_from_scalar(scalar: RlpScalar<'a>) -> Result<Self, DecodeError> {
        validate_integer_payload(scalar.payload())?;
        Ok(Self { scalar })
    }

    /// Returns the underlying scalar byte string.
    #[must_use]
    pub const fn scalar(self) -> RlpScalar<'a> {
        self.scalar
    }

    /// Returns the canonical integer payload bytes.
    #[must_use]
    pub const fn payload(self) -> &'a [u8] {
        self.scalar.payload()
    }

    /// Returns the total encoded item length in bytes.
    #[must_use]
    pub const fn encoded_len(self) -> usize {
        self.scalar.encoded_len()
    }

    /// Returns the RLP header length in bytes.
    #[must_use]
    pub const fn header_len(self) -> usize {
        self.scalar.header_len()
    }

    /// Returns the canonical scalar form used by the integer encoding.
    #[must_use]
    pub const fn form(self) -> RlpScalarForm {
        self.scalar.form()
    }

    /// Returns true when the integer value is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.scalar.payload().is_empty()
    }

    /// Converts this integer to `u64` after checking the byte width.
    pub fn to_u64(self) -> Result<u64, DecodeError> {
        fold_u64(self.payload())
    }

    /// Converts this integer to `u128` after checking the byte width.
    pub fn to_u128(self) -> Result<u128, DecodeError> {
        fold_u128(self.payload())
    }

    /// Converts this integer to right-aligned unsigned 256-bit bytes.
    pub fn to_be_bytes32(self) -> Result<[u8; 32], DecodeError> {
        to_be_bytes32(self.payload())
    }
}

impl<'a> TryFrom<RlpScalar<'a>> for RlpInteger<'a> {
    type Error = DecodeError;

    fn try_from(value: RlpScalar<'a>) -> Result<Self, Self::Error> {
        Self::try_from_scalar(value)
    }
}

/// Decodes exactly one canonical RLP integer.
pub fn decode_rlp_integer<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<RlpInteger<'a>, DecodeError> {
    let mut accumulator = limits.accumulator();
    let integer = decode_rlp_integer_partial(input, &mut accumulator)?;
    require_exact_consumption(integer.encoded_len(), input.len())?;
    Ok(integer)
}

/// Decodes one canonical RLP integer from the start of `input`.
///
/// Warning: this intentionally accepts trailing bytes. Use
/// [`decode_rlp_integer`] when the full input must be consumed.
///
/// The input-length budget check applies to the full `input` slice, not only
/// the consumed integer bytes. Callers that decode from a larger outer buffer
/// must pre-slice before calling this helper.
pub fn decode_rlp_integer_partial<'a>(
    input: &'a [u8],
    accumulator: &mut DecodeAccumulator,
) -> Result<RlpInteger<'a>, DecodeError> {
    let scalar = decode_rlp_scalar_partial(input, accumulator)?;
    RlpInteger::try_from_scalar(scalar)
}

/// Decodes exactly one canonical RLP integer and converts it to `u64`.
pub fn decode_rlp_u64(input: &[u8], limits: DecodeLimits) -> Result<u64, DecodeError> {
    decode_rlp_integer(input, limits)?.to_u64()
}

/// Decodes exactly one canonical RLP integer and converts it to `u128`.
pub fn decode_rlp_u128(input: &[u8], limits: DecodeLimits) -> Result<u128, DecodeError> {
    decode_rlp_integer(input, limits)?.to_u128()
}

/// Decodes exactly one canonical RLP integer as unsigned 256-bit bytes.
pub fn decode_rlp_u256_bytes(input: &[u8], limits: DecodeLimits) -> Result<[u8; 32], DecodeError> {
    decode_rlp_integer(input, limits)?.to_be_bytes32()
}

pub(super) fn validate_integer_payload(payload: &[u8]) -> Result<(), DecodeError> {
    if payload.first().is_some_and(|byte| *byte == 0) {
        return Err(DecodeError::Malformed);
    }
    Ok(())
}

fn fold_u64(payload: &[u8]) -> Result<u64, DecodeError> {
    // Pre-condition: validate_integer_payload already rejected leading zeros.
    // Length is the only integer-canonicality guard needed here.
    if payload.len() > MAX_U64_BYTES {
        return Err(DecodeError::LengthOverflow);
    }

    let mut value = 0_u64;
    for byte in payload {
        value = value
            .checked_mul(INTEGER_RADIX_U64)
            .ok_or(DecodeError::LengthOverflow)?;
        value = value
            .checked_add(u64::from(*byte))
            .ok_or(DecodeError::LengthOverflow)?;
    }
    Ok(value)
}

fn fold_u128(payload: &[u8]) -> Result<u128, DecodeError> {
    // Pre-condition: validate_integer_payload already rejected leading zeros.
    // Length is the only integer-canonicality guard needed here.
    if payload.len() > MAX_U128_BYTES {
        return Err(DecodeError::LengthOverflow);
    }

    let mut value = 0_u128;
    for byte in payload {
        value = value
            .checked_mul(INTEGER_RADIX_U128)
            .ok_or(DecodeError::LengthOverflow)?;
        value = value
            .checked_add(u128::from(*byte))
            .ok_or(DecodeError::LengthOverflow)?;
    }
    Ok(value)
}

fn to_be_bytes32(payload: &[u8]) -> Result<[u8; 32], DecodeError> {
    // Pre-condition: validate_integer_payload already rejected leading zeros.
    // Length is the only integer-canonicality guard needed here.
    if payload.len() > MAX_RLP_U256_BYTES {
        return Err(DecodeError::LengthOverflow);
    }

    let mut output = [0_u8; 32];
    let start = MAX_RLP_U256_BYTES
        .checked_sub(payload.len())
        .ok_or(DecodeError::LengthOverflow)?;
    // start is 32 - payload.len(), with payload.len() <= 32, so the range is
    // always inside output. Keep the Result form to satisfy indexing policy.
    let target = output
        .get_mut(start..)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    target.copy_from_slice(payload);
    Ok(output)
}
