use crate::{
    DecodeAccumulator, DecodeError, DecodeLimits, checked_len_add, require_exact_consumption,
    require_range_in_bounds,
};

const SHORT_STRING_OFFSET: u8 = 0x80;
const LONG_STRING_OFFSET: u8 = 0xb7;
const SHORT_LIST_OFFSET: u8 = 0xc0;
const SHORT_STRING_LIMIT: usize = 55;
const LENGTH_RADIX: usize = 256;

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
    let scalar = decode_rlp_scalar_prefix(input, &mut accumulator)?;
    require_exact_consumption(scalar.encoded_len, input.len())?;
    Ok(scalar)
}

/// Decodes one canonical RLP scalar byte string from the start of `input`.
///
/// This helper does not reject trailing bytes. It is intended for future nested
/// decoders that need to consume one item while sharing cumulative budgets.
pub fn decode_rlp_scalar_prefix<'a>(
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

fn decode_short_string(input: &[u8], prefix: u8) -> Result<RlpScalar<'_>, DecodeError> {
    let payload_len = usize::from(
        prefix
            .checked_sub(SHORT_STRING_OFFSET)
            .ok_or(DecodeError::Malformed)?,
    );
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

fn decode_long_string(input: &[u8], prefix: u8) -> Result<RlpScalar<'_>, DecodeError> {
    let len_of_len = usize::from(
        prefix
            .checked_sub(LONG_STRING_OFFSET)
            .ok_or(DecodeError::Malformed)?,
    );
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

fn parse_payload_len(input: &[u8], offset: usize, len: usize) -> Result<usize, DecodeError> {
    let end = require_range_in_bounds(offset, len, input.len())?;
    let bytes = input
        .get(offset..end)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    if bytes.first().is_none_or(|byte| *byte == 0) {
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

fn payload(input: &[u8], offset: usize, len: usize) -> Result<&[u8], DecodeError> {
    let end = require_range_in_bounds(offset, len, input.len())?;
    input.get(offset..end).ok_or(DecodeError::OffsetOutOfBounds)
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::vec;

    #[test]
    fn decodes_single_byte_scalar() {
        assert_eq!(
            decode_rlp_scalar(&[0x7f], DecodeLimits::TEST_FIXTURE),
            Ok(RlpScalar {
                payload: &[0x7f],
                encoded_len: 1,
                header_len: 0,
                form: RlpScalarForm::SingleByte,
            })
        );
    }

    #[test]
    fn decodes_empty_string() {
        assert_eq!(
            decode_rlp_scalar(&[0x80], DecodeLimits::TEST_FIXTURE),
            Ok(RlpScalar {
                payload: &[],
                encoded_len: 1,
                header_len: 1,
                form: RlpScalarForm::ShortString,
            })
        );
    }

    #[test]
    fn decodes_short_string() {
        assert_eq!(
            decode_rlp_scalar(&[0x83, b'd', b'o', b'g'], DecodeLimits::TEST_FIXTURE),
            Ok(RlpScalar {
                payload: b"dog",
                encoded_len: 4,
                header_len: 1,
                form: RlpScalarForm::ShortString,
            })
        );
    }

    #[test]
    fn decodes_multi_byte_payload() {
        assert_eq!(
            decode_rlp_scalar(&[0x82, 0x04, 0x00], DecodeLimits::TEST_FIXTURE),
            Ok(RlpScalar {
                payload: &[0x04, 0x00],
                encoded_len: 3,
                header_len: 1,
                form: RlpScalarForm::ShortString,
            })
        );
    }

    #[test]
    fn decodes_long_string() {
        let mut input = vec![0xb8, 56];
        input.extend_from_slice(&[b'a'; 56]);

        assert!(matches!(
            decode_rlp_scalar(&input, DecodeLimits::TEST_FIXTURE),
            Ok(scalar)
                if scalar.payload().len() == 56
                    && scalar.encoded_len() == 58
                    && scalar.header_len() == 2
                    && scalar.form() == RlpScalarForm::LongString
        ));
    }

    #[test]
    fn prefix_decoder_leaves_trailing_bytes_to_caller() {
        let mut accumulator = DecodeLimits::TEST_FIXTURE.accumulator();

        assert!(matches!(
            decode_rlp_scalar_prefix(&[0x83, b'd', b'o', b'g', 0x80], &mut accumulator),
            Ok(scalar) if scalar.payload() == b"dog" && scalar.encoded_len() == 4
        ));
        assert_eq!(accumulator.total_items(), 1);
    }

    #[test]
    fn exact_decoder_rejects_trailing_bytes() {
        assert_eq!(
            decode_rlp_scalar(&[0x83, b'd', b'o', b'g', 0x80], DecodeLimits::TEST_FIXTURE),
            Err(DecodeError::TrailingBytes)
        );
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(
            decode_rlp_scalar(&[], DecodeLimits::TEST_FIXTURE),
            Err(DecodeError::Malformed)
        );
    }

    #[test]
    fn rejects_short_string_missing_payload() {
        assert_eq!(
            decode_rlp_scalar(&[0x82, 0x04], DecodeLimits::TEST_FIXTURE),
            Err(DecodeError::OffsetOutOfBounds)
        );
    }

    #[test]
    fn rejects_noncanonical_single_byte_string() {
        assert_eq!(
            decode_rlp_scalar(&[0x81, 0x7f], DecodeLimits::TEST_FIXTURE),
            Err(DecodeError::Malformed)
        );
    }

    #[test]
    fn rejects_long_string_missing_length_bytes() {
        assert_eq!(
            decode_rlp_scalar(&[0xb9, 0x01], DecodeLimits::TEST_FIXTURE),
            Err(DecodeError::OffsetOutOfBounds)
        );
    }

    #[test]
    fn rejects_long_string_missing_payload() {
        assert_eq!(
            decode_rlp_scalar(&[0xb8, 56, b'a'], DecodeLimits::TEST_FIXTURE),
            Err(DecodeError::OffsetOutOfBounds)
        );
    }

    #[test]
    fn rejects_long_string_with_leading_zero_length() {
        let mut input = vec![0xb9, 0, 56];
        input.extend_from_slice(&[b'a'; 56]);

        assert_eq!(
            decode_rlp_scalar(&input, DecodeLimits::TEST_FIXTURE),
            Err(DecodeError::Malformed)
        );
    }

    #[test]
    fn rejects_long_string_used_for_short_payload() {
        let mut input = vec![0xb8, 55];
        input.extend_from_slice(&[b'a'; 55]);

        assert_eq!(
            decode_rlp_scalar(&input, DecodeLimits::TEST_FIXTURE),
            Err(DecodeError::Malformed)
        );
    }

    #[test]
    fn rejects_list_prefix_for_scalar_decoder() {
        assert_eq!(
            decode_rlp_scalar(&[0xc0], DecodeLimits::TEST_FIXTURE),
            Err(DecodeError::UnexpectedList)
        );
    }

    #[test]
    fn enforces_input_budget() {
        let limits = DecodeLimits {
            max_input_bytes: 1,
            ..DecodeLimits::TEST_FIXTURE
        };

        assert_eq!(
            decode_rlp_scalar(&[0x81, 0x80], limits),
            Err(DecodeError::InputTooLarge)
        );
    }

    #[test]
    fn enforces_item_budget() {
        let limits = DecodeLimits {
            max_total_items: 0,
            ..DecodeLimits::TEST_FIXTURE
        };

        assert_eq!(
            decode_rlp_scalar(&[0x80], limits),
            Err(DecodeError::ItemCountExceeded)
        );
    }
}
