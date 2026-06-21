use crate::DecodeError;

/// Resource limits required by every untrusted decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    /// Maximum accepted input length in bytes.
    pub max_input_bytes: usize,
    /// Maximum items accepted in any single decoded list.
    pub max_list_items: usize,
    /// Maximum nested list depth.
    pub max_nesting_depth: usize,
    /// Maximum total allocation a decoder may request.
    pub max_total_allocation: usize,
    /// Maximum proof nodes accepted by a proof decoder.
    pub max_proof_nodes: usize,
    /// Maximum total decoded items visited by one decoder invocation.
    pub max_total_items: usize,
}

impl DecodeLimits {
    /// Limits for unit tests and conformance fixtures.
    ///
    /// This is not a production policy. Production decoders should choose
    /// deployment-specific limits or start from [`Self::PRODUCTION_RECOMMENDED`].
    pub const TEST_FIXTURE: Self = Self {
        max_input_bytes: 1 << 20,
        max_list_items: 4096,
        max_nesting_depth: 64,
        max_total_allocation: 1 << 20,
        max_proof_nodes: 1024,
        max_total_items: 8192,
    };

    /// Recommended starting point for production wire decoders.
    ///
    /// Review and tighten these values per deployment context before relying
    /// on them for externally reachable services.
    pub const PRODUCTION_RECOMMENDED: Self = Self {
        max_input_bytes: 2 << 20,
        max_list_items: 16_384,
        max_nesting_depth: 64,
        max_total_allocation: 4 << 20,
        max_proof_nodes: 4096,
        max_total_items: 65_536,
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
    pub fn check_single_allocation_limit(self, size: usize) -> Result<(), DecodeError> {
        if size > self.max_total_allocation {
            return Err(DecodeError::AllocationExceeded);
        }
        Ok(())
    }

    /// Validates one proof-node count against the proof-node budget.
    ///
    /// Decoders that traverse more than one proof segment must use
    /// [`DecodeAccumulator::account_proof_nodes`] for cumulative accounting.
    pub fn check_proof_node_count(self, count: usize) -> Result<(), DecodeError> {
        if count > self.max_proof_nodes {
            return Err(DecodeError::ProofTooLarge);
        }
        Ok(())
    }

    /// Validates one decoded item count against the cumulative item budget.
    ///
    /// This helper is for single-count checks only. Decoders that visit items
    /// incrementally must use [`DecodeAccumulator::account_items`].
    pub fn check_item_count(self, count: usize) -> Result<(), DecodeError> {
        if count > self.max_total_items {
            return Err(DecodeError::ItemCountExceeded);
        }
        Ok(())
    }

    /// Starts stateful budget accounting for a decoder invocation.
    #[must_use]
    pub const fn accumulator(self) -> DecodeAccumulator {
        DecodeAccumulator {
            limits: self,
            total_allocated: 0,
            total_items: 0,
            proof_nodes: 0,
        }
    }
}

/// Stateful budget accounting for one decoder invocation.
#[derive(Debug, Eq, PartialEq)]
pub struct DecodeAccumulator {
    limits: DecodeLimits,
    total_allocated: usize,
    total_items: usize,
    proof_nodes: usize,
}

impl DecodeAccumulator {
    /// Returns the active decode limits.
    #[must_use]
    pub const fn limits(&self) -> DecodeLimits {
        self.limits
    }

    /// Returns the cumulative allocation accounted so far.
    #[must_use]
    pub const fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// Returns the cumulative decoded items accounted so far.
    #[must_use]
    pub const fn total_items(&self) -> usize {
        self.total_items
    }

    /// Returns the cumulative proof nodes accounted so far.
    #[must_use]
    pub const fn proof_nodes(&self) -> usize {
        self.proof_nodes
    }

    /// Validates the input length before parsing starts.
    pub fn check_input_len(&self, len: usize) -> Result<(), DecodeError> {
        self.limits.check_input_len(len)
    }

    /// Validates a decoded list item count.
    pub fn check_list_count(&self, count: usize) -> Result<(), DecodeError> {
        self.limits.check_list_count(count)
    }

    /// Validates the current nesting depth.
    pub fn check_nesting_depth(&self, depth: usize) -> Result<(), DecodeError> {
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

    /// Accounts for decoded items against the cumulative item budget.
    pub fn account_items(&mut self, count: usize) -> Result<(), DecodeError> {
        let new_total = self
            .total_items
            .checked_add(count)
            .ok_or(DecodeError::ItemCountExceeded)?;
        if new_total > self.limits.max_total_items {
            return Err(DecodeError::ItemCountExceeded);
        }
        self.total_items = new_total;
        Ok(())
    }

    /// Accounts for proof nodes against the cumulative proof-node budget.
    pub fn account_proof_nodes(&mut self, count: usize) -> Result<(), DecodeError> {
        let new_total = self
            .proof_nodes
            .checked_add(count)
            .ok_or(DecodeError::ProofTooLarge)?;
        if new_total > self.limits.max_proof_nodes {
            return Err(DecodeError::ProofTooLarge);
        }
        self.proof_nodes = new_total;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_input() {
        let limits = DecodeLimits {
            max_input_bytes: 2,
            ..DecodeLimits::TEST_FIXTURE
        };
        assert_eq!(limits.check_input_len(3), Err(DecodeError::InputTooLarge));
    }

    #[test]
    fn rejects_oversized_list() {
        let limits = DecodeLimits {
            max_list_items: 2,
            ..DecodeLimits::TEST_FIXTURE
        };
        assert_eq!(limits.check_list_count(3), Err(DecodeError::ListTooLong));
    }

    #[test]
    fn rejects_excessive_nesting_depth() {
        let limits = DecodeLimits {
            max_nesting_depth: 2,
            ..DecodeLimits::TEST_FIXTURE
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
            ..DecodeLimits::TEST_FIXTURE
        };
        assert_eq!(
            limits.check_single_allocation_limit(3),
            Err(DecodeError::AllocationExceeded)
        );
    }

    #[test]
    fn rejects_excessive_proof_nodes() {
        let limits = DecodeLimits {
            max_proof_nodes: 2,
            ..DecodeLimits::TEST_FIXTURE
        };
        assert_eq!(
            limits.check_proof_node_count(3),
            Err(DecodeError::ProofTooLarge)
        );
    }

    #[test]
    fn rejects_excessive_total_items() {
        let limits = DecodeLimits {
            max_total_items: 2,
            ..DecodeLimits::TEST_FIXTURE
        };
        assert_eq!(
            limits.check_item_count(3),
            Err(DecodeError::ItemCountExceeded)
        );
    }

    #[test]
    fn fixture_and_production_limits_are_distinct() {
        let production = DecodeLimits::PRODUCTION_RECOMMENDED;
        let fixture = DecodeLimits::TEST_FIXTURE;

        assert!(production.max_input_bytes > fixture.max_input_bytes);
        assert!(production.max_total_allocation > fixture.max_total_allocation);
        assert!(production.max_proof_nodes > fixture.max_proof_nodes);
        assert!(production.max_total_items > fixture.max_total_items);
    }

    #[test]
    fn accumulator_rejects_cumulative_allocation_over_budget() {
        let limits = DecodeLimits {
            max_total_allocation: 4,
            ..DecodeLimits::TEST_FIXTURE
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
            ..DecodeLimits::TEST_FIXTURE
        };
        let mut accumulator = limits.accumulator();

        assert_eq!(accumulator.check_allocation(usize::MAX), Ok(()));
        assert_eq!(
            accumulator.check_allocation(1),
            Err(DecodeError::AllocationExceeded)
        );
    }

    #[test]
    fn accumulator_rejects_cumulative_items_over_budget() {
        let limits = DecodeLimits {
            max_total_items: 4,
            ..DecodeLimits::TEST_FIXTURE
        };
        let mut accumulator = limits.accumulator();

        assert_eq!(accumulator.account_items(3), Ok(()));
        assert_eq!(accumulator.total_items(), 3);
        assert_eq!(
            accumulator.account_items(2),
            Err(DecodeError::ItemCountExceeded)
        );
        assert_eq!(accumulator.total_items(), 3);
    }

    #[test]
    fn accumulator_rejects_cumulative_proof_nodes_over_budget() {
        let limits = DecodeLimits {
            max_proof_nodes: 4,
            ..DecodeLimits::TEST_FIXTURE
        };
        let mut accumulator = limits.accumulator();

        assert_eq!(accumulator.account_proof_nodes(3), Ok(()));
        assert_eq!(accumulator.proof_nodes(), 3);
        assert_eq!(
            accumulator.account_proof_nodes(2),
            Err(DecodeError::ProofTooLarge)
        );
        assert_eq!(accumulator.proof_nodes(), 3);
    }
}
