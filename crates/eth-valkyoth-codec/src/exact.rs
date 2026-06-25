use core::cmp::Ordering;

use crate::DecodeError;

/// Ensures a decoder consumed the whole input.
pub fn require_exact_consumption(consumed: usize, input_len: usize) -> Result<(), DecodeError> {
    match consumed.cmp(&input_len) {
        Ordering::Equal => Ok(()),
        Ordering::Less => Err(DecodeError::TrailingBytes),
        Ordering::Greater => Err(DecodeError::DecoderOverread),
    }
}

/// Adds two decoded lengths and rejects integer overflow.
pub fn checked_len_add(left: usize, right: usize) -> Result<usize, DecodeError> {
    left.checked_add(right).ok_or(DecodeError::LengthOverflow)
}

/// Computes the end offset for a decoded range and rejects overflow.
pub fn checked_range_end(offset: usize, len: usize) -> Result<usize, DecodeError> {
    checked_len_add(offset, len)
}

/// Ensures a decoded range is inside an input buffer and returns its end.
pub fn require_range_in_bounds(
    offset: usize,
    len: usize,
    input_len: usize,
) -> Result<usize, DecodeError> {
    let end = checked_range_end(offset, len)?;
    if end > input_len {
        return Err(DecodeError::OffsetOutOfBounds);
    }
    Ok(end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_trailing_bytes() {
        assert_eq!(
            require_exact_consumption(1, 2),
            Err(DecodeError::TrailingBytes)
        );
    }

    #[test]
    fn detects_decoder_overread() {
        assert_eq!(
            require_exact_consumption(3, 2),
            Err(DecodeError::DecoderOverread)
        );
    }

    #[test]
    fn checked_len_add_rejects_overflow() {
        assert_eq!(
            checked_len_add(usize::MAX, 1),
            Err(DecodeError::LengthOverflow)
        );
    }

    #[test]
    fn checked_range_end_rejects_overflow() {
        assert_eq!(
            checked_range_end(usize::MAX, 1),
            Err(DecodeError::LengthOverflow)
        );
    }

    #[test]
    fn require_range_in_bounds_accepts_valid_range() {
        assert_eq!(require_range_in_bounds(2, 3, 5), Ok(5));
    }

    #[test]
    fn require_range_in_bounds_rejects_end_beyond_input() {
        assert_eq!(
            require_range_in_bounds(2, 4, 5),
            Err(DecodeError::OffsetOutOfBounds)
        );
    }

    #[test]
    fn require_range_in_bounds_rejects_offset_beyond_input() {
        assert_eq!(
            require_range_in_bounds(6, 0, 5),
            Err(DecodeError::OffsetOutOfBounds)
        );
    }
}
