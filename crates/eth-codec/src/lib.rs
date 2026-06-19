#![no_std]
#![forbid(unsafe_code)]
//! Bounded decoding policy for untrusted Ethereum wire inputs.

use core::cmp::Ordering;

/// Resource limits required by every untrusted decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    /// Maximum accepted input length in bytes.
    pub max_input_bytes: usize,
    /// Maximum items accepted in any decoded list.
    pub max_list_items: usize,
    /// Maximum nested list depth.
    pub max_nesting_depth: usize,
    /// Maximum total allocation a decoder may request.
    pub max_total_allocation: usize,
}

impl DecodeLimits {
    /// Conservative defaults for small unit and conformance fixtures.
    pub const STRICT: Self = Self {
        max_input_bytes: 1 << 20,
        max_list_items: 4096,
        max_nesting_depth: 64,
        max_total_allocation: 1 << 20,
    };

    /// Validates the input length before parsing starts.
    pub fn check_input_len(self, len: usize) -> Result<(), DecodeError> {
        if len > self.max_input_bytes {
            return Err(DecodeError::InputTooLarge);
        }
        Ok(())
    }
}

/// Shared decode failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// The byte input is larger than the active decode budget.
    InputTooLarge,
    /// The input contains trailing bytes after a decoded value.
    TrailingBytes,
    /// A decoder reported consuming more bytes than the input contains.
    DecoderOverread,
    /// The input is malformed for the selected wire format.
    Malformed,
}

/// Ensures a decoder consumed the whole input.
pub fn require_exact_consumption(consumed: usize, input_len: usize) -> Result<(), DecodeError> {
    match consumed.cmp(&input_len) {
        Ordering::Equal => Ok(()),
        Ordering::Less => Err(DecodeError::TrailingBytes),
        Ordering::Greater => Err(DecodeError::DecoderOverread),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_input() {
        let limits = DecodeLimits {
            max_input_bytes: 2,
            ..DecodeLimits::STRICT
        };
        assert_eq!(limits.check_input_len(3), Err(DecodeError::InputTooLarge));
    }

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
}
