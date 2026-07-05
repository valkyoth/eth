use core::fmt;

use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_primitives::{B256, Gas, TransactionType};
use eth_valkyoth_protocol::{
    TransactionEnvelope, TransactionEnvelopeError, decode_transaction_envelope,
};

use crate::{ExecutionEnvironment, StateSnapshot};

/// Transaction bytes admitted to the execution boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionTransaction<'a> {
    raw: &'a [u8],
    envelope: TransactionEnvelope<'a>,
}

impl<'a> ExecutionTransaction<'a> {
    /// Decodes and binds raw transaction bytes to a bounded envelope shell.
    pub fn decode(raw: &'a [u8], limits: DecodeLimits) -> Result<Self, TransactionEnvelopeError> {
        let envelope = decode_transaction_envelope(raw, limits)?;
        Ok(Self { raw, envelope })
    }

    /// Raw transaction bytes selected for execution.
    #[must_use]
    pub const fn raw(self) -> &'a [u8] {
        self.raw
    }

    /// Decoded transaction envelope shell selected for execution.
    #[must_use]
    pub const fn envelope(self) -> TransactionEnvelope<'a> {
        self.envelope
    }

    /// Explicit legacy or typed transaction domain.
    #[must_use]
    pub const fn transaction_type(self) -> TransactionType {
        match self.envelope {
            TransactionEnvelope::Legacy(_) => TransactionType::LEGACY,
            TransactionEnvelope::Typed(typed) => typed.transaction_type,
        }
    }
}

/// Complete execution request boundary.
#[derive(Debug)]
pub struct ExecutionRequest<'a, S: StateSnapshot + ?Sized> {
    environment: ExecutionEnvironment,
    transaction: ExecutionTransaction<'a>,
    snapshot: &'a S,
}

impl<'a, S: StateSnapshot + ?Sized> ExecutionRequest<'a, S> {
    /// Creates a request from explicit environment, transaction, and state.
    #[must_use]
    pub const fn new(
        environment: ExecutionEnvironment,
        transaction: ExecutionTransaction<'a>,
        snapshot: &'a S,
    ) -> Self {
        Self {
            environment,
            transaction,
            snapshot,
        }
    }

    /// Execution environment for this request.
    #[must_use]
    pub const fn environment(&self) -> ExecutionEnvironment {
        self.environment
    }

    /// Transaction selected for execution.
    #[must_use]
    pub const fn transaction(&self) -> ExecutionTransaction<'a> {
        self.transaction
    }

    /// State snapshot selected for execution.
    #[must_use]
    pub const fn snapshot(&self) -> &S {
        self.snapshot
    }

    /// Builds a report with a caller-computed transaction hash.
    ///
    /// This crate does not compute Keccak-256 here because concrete hash
    /// implementations stay outside the EVM boundary. Callers must supply the
    /// hash of [`ExecutionTransaction::raw`] using their reviewed hash backend.
    #[must_use]
    pub fn report(&self, transaction_hash: B256) -> ExecutionReport {
        ExecutionReport {
            environment: self.environment,
            transaction_type: self.transaction.transaction_type(),
            transaction_hash,
            snapshot_id: self.snapshot.snapshot_id(),
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
