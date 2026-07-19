use crate::{DecodeError, DecodeLimits};

/// Complete work policy for one untrusted decode operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeSessionPolicy {
    limits: DecodeLimits,
    max_encoded_bytes: usize,
    max_rlp_headers: usize,
    max_hashes: usize,
    max_hash_bytes: usize,
    max_nibbles: usize,
    max_value_bytes: usize,
    max_total_work: usize,
}

impl DecodeSessionPolicy {
    /// Policy for unit tests, conformance fixtures, and fuzz targets.
    #[cfg(any(test, feature = "testing"))]
    pub const TEST_FIXTURE: Self = Self {
        limits: DecodeLimits::TEST_FIXTURE,
        max_encoded_bytes: 1 << 20,
        max_rlp_headers: 8192,
        max_hashes: 1024,
        max_hash_bytes: 1 << 20,
        max_nibbles: 2 << 20,
        max_value_bytes: 1 << 20,
        max_total_work: 4 << 20,
    };

    /// Starting point for externally reachable decode sessions.
    ///
    /// This policy must be copied, reviewed field by field, and tightened for
    /// the deployment before use. [`Self::validate_deployment_policy`] rejects
    /// the unchanged value.
    pub const DEPLOYMENT_STARTING_POINT: Self = Self {
        limits: DecodeLimits::DEPLOYMENT_STARTING_POINT,
        max_encoded_bytes: 4 << 20,
        max_rlp_headers: 65_536,
        max_hashes: 4096,
        max_hash_bytes: 4 << 20,
        max_nibbles: 8 << 20,
        max_value_bytes: 4 << 20,
        max_total_work: 16 << 20,
    };

    /// Constructs an explicit policy and validates all cross-limit relations.
    #[allow(clippy::too_many_arguments)]
    pub const fn reviewed_policy(
        limits: DecodeLimits,
        max_encoded_bytes: usize,
        max_rlp_headers: usize,
        max_hashes: usize,
        max_hash_bytes: usize,
        max_nibbles: usize,
        max_value_bytes: usize,
        max_total_work: usize,
    ) -> Result<Self, DecodeError> {
        let policy = Self {
            limits,
            max_encoded_bytes,
            max_rlp_headers,
            max_hashes,
            max_hash_bytes,
            max_nibbles,
            max_value_bytes,
            max_total_work,
        };
        match policy.validate_relationships() {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
        Ok(policy)
    }

    /// Derives a compatibility session for APIs that still accept only
    /// [`DecodeLimits`].
    ///
    /// New composite untrusted operations should use [`Self::reviewed_policy`]
    /// and choose every work ceiling explicitly.
    pub fn compatibility_policy(limits: DecodeLimits) -> Result<Self, DecodeError> {
        let proof_bytes = limits.max_input_bytes.max(limits.max_total_allocation);
        // Proof preflight and traversal intentionally account repeated parsing.
        let max_encoded_bytes = proof_bytes
            .checked_mul(4)
            .ok_or(DecodeError::InvalidSessionPolicy)?;
        let max_hashes = limits
            .max_proof_nodes
            .checked_add(1)
            .ok_or(DecodeError::InvalidSessionPolicy)?;
        let max_hash_bytes = proof_bytes
            .checked_add(32)
            .ok_or(DecodeError::InvalidSessionPolicy)?;
        let max_nibbles = proof_bytes
            .checked_mul(6)
            .ok_or(DecodeError::InvalidSessionPolicy)?;
        let max_value_bytes = proof_bytes
            .checked_mul(6)
            .ok_or(DecodeError::InvalidSessionPolicy)?;
        let components = [
            max_encoded_bytes,
            limits.max_total_items,
            limits.max_nesting_depth,
            limits.max_total_allocation,
            limits.max_proof_nodes,
            max_hashes,
            max_hash_bytes,
            max_nibbles,
            max_value_bytes,
        ];
        let max_total_work = components.into_iter().try_fold(0usize, |sum, value| {
            sum.checked_add(value)
                .ok_or(DecodeError::InvalidSessionPolicy)
        })?;
        Self::reviewed_policy(
            limits,
            max_encoded_bytes,
            limits.max_total_items,
            max_hashes,
            max_hash_bytes,
            max_nibbles,
            max_value_bytes,
            max_total_work,
        )
    }

    /// Returns the structural decode limits.
    #[must_use]
    pub const fn limits(self) -> DecodeLimits {
        self.limits
    }

    /// Returns the cumulative encoded-byte scan limit.
    #[must_use]
    pub const fn max_encoded_bytes(self) -> usize {
        self.max_encoded_bytes
    }

    /// Returns the cumulative RLP-header visit limit.
    #[must_use]
    pub const fn max_rlp_headers(self) -> usize {
        self.max_rlp_headers
    }

    /// Returns the cumulative hash count limit.
    #[must_use]
    pub const fn max_hashes(self) -> usize {
        self.max_hashes
    }

    /// Returns the cumulative hashed-byte limit.
    #[must_use]
    pub const fn max_hash_bytes(self) -> usize {
        self.max_hash_bytes
    }

    /// Returns the cumulative compact-path nibble limit.
    #[must_use]
    pub const fn max_nibbles(self) -> usize {
        self.max_nibbles
    }

    /// Returns the cumulative trie-value byte limit.
    #[must_use]
    pub const fn max_value_bytes(self) -> usize {
        self.max_value_bytes
    }

    /// Returns the aggregate work-unit limit.
    #[must_use]
    pub const fn max_total_work(self) -> usize {
        self.max_total_work
    }

    /// Rejects inconsistent component and aggregate ceilings.
    pub const fn validate_relationships(self) -> Result<(), DecodeError> {
        if self.limits.max_list_items > self.limits.max_total_items
            || self.limits.max_proof_nodes > self.limits.max_total_items
            || self.limits.max_total_items > self.max_total_work
            || self.limits.max_proof_nodes > self.max_total_work
            || self.limits.max_input_bytes > self.max_encoded_bytes
            || self.max_encoded_bytes > self.max_total_work
            || self.max_rlp_headers > self.max_total_work
            || self.limits.max_nesting_depth > self.max_total_work
            || self.limits.max_total_allocation > self.max_total_work
            || self.max_hashes > self.max_total_work
            || self.max_hash_bytes > self.max_total_work
            || self.max_nibbles > self.max_total_work
            || self.max_value_bytes > self.max_total_work
        {
            return Err(DecodeError::InvalidSessionPolicy);
        }
        Ok(())
    }

    /// Rejects an unchanged deployment starter policy.
    pub fn validate_deployment_policy(self) -> Result<(), DecodeError> {
        self.validate_relationships()?;
        self.limits.validate_deployment_policy()?;
        if self == Self::DEPLOYMENT_STARTING_POINT {
            return Err(DecodeError::UnreviewedDeploymentPolicy);
        }
        Ok(())
    }
}
