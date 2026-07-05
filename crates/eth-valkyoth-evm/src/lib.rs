#![no_std]
#![forbid(unsafe_code)]
//! Optional REVM adapter boundary for `eth`.

/// Placeholder proving the EVM adapter is explicit and feature-gated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmAdapterBoundary;

/// Result of the v0.37.0 REVM dependency admission review.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RevmDependencyReview {
    /// Latest `revm` version observed in the registry.
    pub latest_revm_version: &'static str,
    /// Latest `revm` Rust version requirement observed in the registry.
    pub latest_revm_rust_version: &'static str,
    /// Newest `revm` line compatible with this workspace's Rust 1.90 floor.
    pub newest_msrv_compatible_revm_version: &'static str,
    /// Whether a REVM crate is admitted into this package graph.
    pub admitted: bool,
    /// Stable reason for the admission decision.
    pub reason: &'static str,
}

/// v0.37.0 REVM dependency admission review result.
///
/// REVM is not admitted in this release. The full `revm` crate and the
/// narrower `revm-primitives` crate were both checked and rejected because
/// their current transitive graph fails the repository dependency policy.
pub const REVM_DEPENDENCY_REVIEW: RevmDependencyReview = RevmDependencyReview {
    latest_revm_version: "41.0.0",
    latest_revm_rust_version: "1.91.0",
    newest_msrv_compatible_revm_version: "36.0.0",
    admitted: false,
    reason: "current REVM graph fails cargo-deny duplicate-version and unmaintained-advisory policy",
};

/// Returns the reviewed REVM dependency admission result.
#[must_use]
pub const fn revm_dependency_review() -> RevmDependencyReview {
    REVM_DEPENDENCY_REVIEW
}

#[cfg(test)]
mod tests {
    use super::{EvmAdapterBoundary, revm_dependency_review};

    #[test]
    fn boundary_is_explicit() {
        assert_eq!(EvmAdapterBoundary, EvmAdapterBoundary);
    }

    #[test]
    fn revm_dependency_is_not_admitted_until_policy_passes() {
        let review = revm_dependency_review();
        assert_eq!(review.latest_revm_version, "41.0.0");
        assert_eq!(review.latest_revm_rust_version, "1.91.0");
        assert_eq!(review.newest_msrv_compatible_revm_version, "36.0.0");
        assert!(!review.admitted);
    }
}
