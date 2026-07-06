use core::fmt;

use eth_valkyoth_primitives::{B256, Gas};

use crate::{ExecutionReport, ExecutionRequest, StateSnapshot};

/// Termination guard required for every future gas-estimation backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GasEstimationTermination {
    /// Backend must stop after this many deterministic execution steps.
    BackendStepLimit {
        /// Maximum backend steps admitted for one estimation attempt.
        max_backend_steps: u64,
    },
    /// Backend must run under a caller-enforced worker timeout.
    WorkerTimeout {
        /// Timeout in milliseconds for one estimation attempt.
        timeout_millis: u64,
    },
    /// Backend must run in an isolated worker with a caller-enforced timeout.
    WorkerIsolation {
        /// Timeout in milliseconds for one isolated worker attempt.
        timeout_millis: u64,
    },
}

impl GasEstimationTermination {
    /// Returns `true` when the termination policy has a non-zero bound.
    #[must_use]
    pub const fn is_bounded(self) -> bool {
        match self {
            Self::BackendStepLimit { max_backend_steps } => max_backend_steps > 0,
            Self::WorkerTimeout { timeout_millis } => timeout_millis > 0,
            Self::WorkerIsolation { timeout_millis } => timeout_millis > 0,
        }
    }
}

/// Bounded gas-estimation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GasEstimationPolicy {
    max_attempts: u32,
    gas_cap: Gas,
    termination: GasEstimationTermination,
}

impl GasEstimationPolicy {
    /// Creates a reviewed gas-estimation policy.
    pub const fn try_new(
        max_attempts: u32,
        gas_cap: Gas,
        termination: GasEstimationTermination,
    ) -> Result<Self, GasEstimationError> {
        if max_attempts == 0 {
            return Err(GasEstimationError::ZeroMaxAttempts);
        }
        if gas_cap.get() == 0 {
            return Err(GasEstimationError::ZeroGasCap);
        }
        if !termination.is_bounded() {
            return Err(GasEstimationError::UnboundedTerminationPolicy);
        }
        Ok(Self {
            max_attempts,
            gas_cap,
            termination,
        })
    }

    /// Maximum backend execution attempts admitted for one estimate.
    #[must_use]
    pub const fn max_attempts(self) -> u32 {
        self.max_attempts
    }

    /// Highest gas value an estimator may try or return.
    #[must_use]
    pub const fn gas_cap(self) -> Gas {
        self.gas_cap
    }

    /// Deterministic termination guard for each backend attempt.
    #[must_use]
    pub const fn termination(self) -> GasEstimationTermination {
        self.termination
    }
}

/// Complete bounded gas-estimation request.
#[derive(Debug)]
pub struct GasEstimationRequest<'a, S: StateSnapshot + ?Sized> {
    execution: ExecutionRequest<'a, S>,
    policy: GasEstimationPolicy,
}

impl<'a, S: StateSnapshot + ?Sized> GasEstimationRequest<'a, S> {
    /// Creates an estimation request after checking the policy against block gas.
    pub fn try_new(
        execution: ExecutionRequest<'a, S>,
        policy: GasEstimationPolicy,
    ) -> Result<Self, GasEstimationError> {
        let block_gas_limit = execution.environment().block().gas_limit;
        if policy.gas_cap().get() > block_gas_limit.get() {
            return Err(GasEstimationError::GasCapExceedsBlockLimit);
        }
        Ok(Self { execution, policy })
    }

    /// Execution request selected for estimation.
    #[must_use]
    pub const fn execution(&self) -> &ExecutionRequest<'a, S> {
        &self.execution
    }

    /// Reviewed estimation policy selected for this request.
    #[must_use]
    pub const fn policy(&self) -> GasEstimationPolicy {
        self.policy
    }

    /// Builds an auditable gas-estimation report.
    pub fn report(
        &self,
        transaction_hash: B256,
        status: GasEstimationStatus,
        attempts: u32,
        estimated_gas: Option<Gas>,
    ) -> Result<GasEstimationReport, GasEstimationError> {
        if attempts > self.policy.max_attempts() {
            return Err(GasEstimationError::AttemptLimitExceeded);
        }
        if let Some(gas) = estimated_gas
            && gas.get() > self.policy.gas_cap().get()
        {
            return Err(GasEstimationError::EstimateExceedsGasCap);
        }
        Ok(GasEstimationReport {
            execution: self.execution.report(transaction_hash),
            policy: self.policy,
            status,
            attempts,
            estimated_gas,
        })
    }
}

/// Deterministic gas-estimation outcome class.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GasEstimationStatus {
    /// A bounded estimate was produced.
    Estimated,
    /// The transaction reverted during estimation.
    Reverted,
    /// A backend would be required but is not admitted by this crate version.
    BackendUnavailable,
}

/// Auditable gas-estimation report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GasEstimationReport {
    /// Execution report binding the estimate to fork, block, transaction, and state.
    pub execution: ExecutionReport,
    /// Policy used to bound the estimation attempt.
    pub policy: GasEstimationPolicy,
    /// Deterministic outcome class.
    pub status: GasEstimationStatus,
    /// Number of backend attempts performed.
    pub attempts: u32,
    /// Estimated gas when available.
    pub estimated_gas: Option<Gas>,
}

/// Gas-estimation boundary failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GasEstimationError {
    /// `max_attempts` must be non-zero.
    ZeroMaxAttempts,
    /// `gas_cap` must be non-zero.
    ZeroGasCap,
    /// Termination policy must have a non-zero bound.
    UnboundedTerminationPolicy,
    /// The estimation gas cap exceeds the selected block gas limit.
    GasCapExceedsBlockLimit,
    /// A report attempted to record more attempts than the policy admits.
    AttemptLimitExceeded,
    /// A report attempted to return an estimate above the selected gas cap.
    EstimateExceedsGasCap,
}

impl GasEstimationError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ZeroMaxAttempts => "ETH_EVM_GAS_ESTIMATION_ZERO_MAX_ATTEMPTS",
            Self::ZeroGasCap => "ETH_EVM_GAS_ESTIMATION_ZERO_GAS_CAP",
            Self::UnboundedTerminationPolicy => "ETH_EVM_GAS_ESTIMATION_UNBOUNDED_POLICY",
            Self::GasCapExceedsBlockLimit => "ETH_EVM_GAS_ESTIMATION_CAP_EXCEEDS_BLOCK",
            Self::AttemptLimitExceeded => "ETH_EVM_GAS_ESTIMATION_ATTEMPT_LIMIT_EXCEEDED",
            Self::EstimateExceedsGasCap => "ETH_EVM_GAS_ESTIMATION_ESTIMATE_EXCEEDS_CAP",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::ZeroMaxAttempts => "gas-estimation maximum attempts must be non-zero",
            Self::ZeroGasCap => "gas-estimation gas cap must be non-zero",
            Self::UnboundedTerminationPolicy => {
                "gas-estimation termination policy must have a non-zero bound"
            }
            Self::GasCapExceedsBlockLimit => "gas-estimation gas cap exceeds block gas limit",
            Self::AttemptLimitExceeded => "gas-estimation attempt count exceeds policy limit",
            Self::EstimateExceedsGasCap => "gas-estimation result exceeds policy gas cap",
        }
    }
}

impl fmt::Display for GasEstimationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GasEstimationError {}
