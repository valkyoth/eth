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

    /// Validates a decoded list item count.
    pub fn check_list_count(self, count: usize) -> Result<(), DecodeError> {
        if count > self.max_list_items {
            return Err(DecodeError::ListTooLong);
        }
        Ok(())
    }

    /// Validates the current nesting depth.
    pub fn check_nesting_depth(self, depth: usize) -> Result<(), DecodeError> {
        if depth > self.max_nesting_depth {
            return Err(DecodeError::NestingTooDeep);
        }
        Ok(())
    }

    /// Validates one requested allocation against the allocation budget.
    ///
    /// This helper is for single-allocation checks only. Decoders that can make
    /// more than one allocation must use [`DecodeAccumulator`] to enforce the
    /// cumulative budget.
    pub fn check_allocation(self, size: usize) -> Result<(), DecodeError> {
        if size > self.max_total_allocation {
            return Err(DecodeError::AllocationExceeded);
        }
        Ok(())
    }

    /// Starts stateful budget accounting for a decoder invocation.
    #[must_use]
    pub const fn accumulator(self) -> DecodeAccumulator {
        DecodeAccumulator {
            limits: self,
            total_allocated: 0,
        }
    }
}

/// Stateful budget accounting for one decoder invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeAccumulator {
    limits: DecodeLimits,
    total_allocated: usize,
}

impl DecodeAccumulator {
    /// Returns the active decode limits.
    #[must_use]
    pub const fn limits(self) -> DecodeLimits {
        self.limits
    }

    /// Returns the cumulative allocation accounted so far.
    #[must_use]
    pub const fn total_allocated(self) -> usize {
        self.total_allocated
    }

    /// Validates the input length before parsing starts.
    pub fn check_input_len(self, len: usize) -> Result<(), DecodeError> {
        self.limits.check_input_len(len)
    }

    /// Validates a decoded list item count.
    pub fn check_list_count(self, count: usize) -> Result<(), DecodeError> {
        self.limits.check_list_count(count)
    }

    /// Validates the current nesting depth.
    pub fn check_nesting_depth(self, depth: usize) -> Result<(), DecodeError> {
        self.limits.check_nesting_depth(depth)
    }

    /// Accounts for one allocation against the cumulative allocation budget.
    pub fn check_allocation(&mut self, size: usize) -> Result<(), DecodeError> {
        let new_total = self
            .total_allocated
            .checked_add(size)
            .ok_or(DecodeError::AllocationExceeded)?;
        if new_total > self.limits.max_total_allocation {
            return Err(DecodeError::AllocationExceeded);
        }
        self.total_allocated = new_total;
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
    /// A decoded list contains more items than the active decode budget.
    ListTooLong,
    /// Decoding exceeded the active nesting-depth budget.
    NestingTooDeep,
    /// A decoder requested allocation beyond the active allocation budget.
    AllocationExceeded,
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
    fn rejects_oversized_list() {
        let limits = DecodeLimits {
            max_list_items: 2,
            ..DecodeLimits::STRICT
        };
        assert_eq!(limits.check_list_count(3), Err(DecodeError::ListTooLong));
    }

    #[test]
    fn rejects_excessive_nesting_depth() {
        let limits = DecodeLimits {
            max_nesting_depth: 2,
            ..DecodeLimits::STRICT
        };
        assert_eq!(
            limits.check_nesting_depth(3),
            Err(DecodeError::NestingTooDeep)
        );
    }

    #[test]
    fn rejects_excessive_allocation() {
        let limits = DecodeLimits {
            max_total_allocation: 2,
            ..DecodeLimits::STRICT
        };
        assert_eq!(
            limits.check_allocation(3),
            Err(DecodeError::AllocationExceeded)
        );
    }

    #[test]
    fn accumulator_rejects_cumulative_allocation_over_budget() {
        let limits = DecodeLimits {
            max_total_allocation: 4,
            ..DecodeLimits::STRICT
        };
        let mut accumulator = limits.accumulator();

        assert_eq!(accumulator.check_allocation(3), Ok(()));
        assert_eq!(accumulator.total_allocated(), 3);
        assert_eq!(
            accumulator.check_allocation(2),
            Err(DecodeError::AllocationExceeded)
        );
        assert_eq!(accumulator.total_allocated(), 3);
    }

    #[test]
    fn accumulator_rejects_allocation_counter_overflow() {
        let limits = DecodeLimits {
            max_total_allocation: usize::MAX,
            ..DecodeLimits::STRICT
        };
        let mut accumulator = limits.accumulator();

        assert_eq!(accumulator.check_allocation(usize::MAX), Ok(()));
        assert_eq!(
            accumulator.check_allocation(1),
            Err(DecodeError::AllocationExceeded)
        );
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
