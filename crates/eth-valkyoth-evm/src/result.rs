use core::fmt;

use eth_valkyoth_primitives::{B256, Gas, TransactionType};

use crate::{ExecutionEnvironment, ExecutionReadyTransaction, StateView};

/// Complete execution request boundary.
#[derive(Debug)]
pub struct ExecutionRequest<'a, S: StateView + ?Sized> {
    transaction: ExecutionReadyTransaction<'a>,
    state: &'a S,
}

impl<'a, S: StateView + ?Sized> ExecutionRequest<'a, S> {
    /// Creates a request from an execution-ready transaction and pure state view.
    #[must_use]
    pub const fn new(transaction: ExecutionReadyTransaction<'a>, state: &'a S) -> Self {
        Self { transaction, state }
    }

    /// Fork and block environment already bound during transaction admission.
    #[must_use]
    pub const fn environment(&self) -> ExecutionEnvironment {
        self.transaction.environment()
    }

    /// Execution-ready transaction selected for execution.
    #[must_use]
    pub const fn transaction(&self) -> &ExecutionReadyTransaction<'a> {
        &self.transaction
    }

    /// Snapshot-pure state view selected for execution.
    #[must_use]
    pub const fn state(&self) -> &S {
        self.state
    }

    /// Compatibility name for the selected state view.
    #[must_use]
    pub const fn snapshot(&self) -> &S {
        self.state
    }

    /// Builds a report with a caller-computed transaction hash.
    ///
    /// This crate does not compute Keccak-256 here because concrete hash
    /// implementations stay outside the EVM boundary. Callers must supply the
    /// hash of [`ExecutionReadyTransaction::raw`] using their reviewed backend.
    #[must_use]
    pub fn report(&self, transaction_hash: B256) -> ExecutionReport {
        ExecutionReport {
            environment: self.environment(),
            transaction_type: self.transaction.transaction_type(),
            transaction_hash,
            snapshot_id: self.state.snapshot_id(),
        }
    }
}

/// Auditable execution report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionReport {
    /// Environment used for the execution attempt.
    pub environment: ExecutionEnvironment,
    /// Legacy or typed transaction domain selected for execution.
    pub transaction_type: TransactionType,
    /// Caller-computed Keccak-256 hash of the exact raw transaction bytes.
    pub transaction_hash: B256,
    /// Caller-provided state snapshot identity.
    pub snapshot_id: B256,
}

/// Execution status from a future backend.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionStatus {
    /// Execution completed successfully.
    Success,
    /// Execution reverted.
    Reverted,
    /// Execution stopped with an exceptional halt.
    Halted,
}

/// Future execution result model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionResult {
    /// Execution status.
    pub status: ExecutionStatus,
    /// Gas consumed by the execution attempt.
    pub gas_used: Gas,
    /// Report binding this result to exact inputs.
    pub report: ExecutionReport,
}

/// Execution failure before or during backend execution.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionError {
    /// No execution backend is admitted by this crate version.
    BackendUnavailable,
}

impl ExecutionError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::BackendUnavailable => "ETH_EVM_BACKEND_UNAVAILABLE",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::BackendUnavailable => "no execution backend is admitted by this crate version",
        }
    }
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ExecutionError {}
