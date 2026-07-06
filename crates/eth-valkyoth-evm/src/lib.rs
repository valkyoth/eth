#![no_std]
#![forbid(unsafe_code)]
//! Explicit no_std EVM execution boundary for `eth`.

#[cfg(feature = "std")]
extern crate std;

mod environment;
mod gas_estimation;
mod result;
mod snapshot;

pub use environment::{BlockExecutionContext, ExecutionEnvironment, ExecutionEnvironmentError};
pub use gas_estimation::{
    GasEstimationError, GasEstimationPolicy, GasEstimationReport, GasEstimationRequest,
    GasEstimationStatus, GasEstimationTermination, MAX_GAS_ESTIMATION_ATTEMPTS,
    MAX_GAS_ESTIMATION_BACKEND_STEPS, MAX_GAS_ESTIMATION_GAS_CAP,
    MAX_GAS_ESTIMATION_TIMEOUT_MILLIS,
};
pub use result::{
    ExecutionError, ExecutionReport, ExecutionRequest, ExecutionResult, ExecutionStatus,
    ExecutionTransaction,
};
pub use snapshot::{SnapshotAccount, SnapshotError, StateSnapshot};

/// Placeholder proving the EVM adapter is explicit and feature-gated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmAdapterBoundary;

/// Result of the v0.37.0 REVM dependency admission review.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RevmDependencyReview {
    /// Date this registry/dependency review was performed.
    pub reviewed_on: &'static str,
    /// Date by which this point-in-time review must be refreshed.
    pub re_review_before: &'static str,
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
///
/// Reviewed 2026-07-05. Re-review is required before 2026-10-05 or before any
/// `eth-valkyoth-evm` feature work, whichever is sooner.
pub const REVM_DEPENDENCY_REVIEW: RevmDependencyReview = RevmDependencyReview {
    reviewed_on: "2026-07-05",
    re_review_before: "2026-10-05",
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
#[path = "tests.rs"]
mod tests;
