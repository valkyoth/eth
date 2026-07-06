use core::fmt;

use eth_valkyoth_primitives::{B256, Gas};

use crate::{ExecutionReport, ExecutionRequest, StateSnapshot};

/// Maximum backend attempts admitted for one gas-estimation request.
pub const MAX_GAS_ESTIMATION_ATTEMPTS: u32 = 32;
/// Maximum gas cap admitted for one gas-estimation request.
pub const MAX_GAS_ESTIMATION_GAS_CAP: Gas = Gas::new(1_000_000_000);
/// Maximum backend steps admitted for one gas-estimation attempt.
pub const MAX_GAS_ESTIMATION_BACKEND_STEPS: u64 = 10_000_000;
/// Maximum worker timeout admitted for one gas-estimation attempt.
pub const MAX_GAS_ESTIMATION_TIMEOUT_MILLIS: u64 = 30_000;

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
    /// Returns `true` when the termination policy has a non-zero reviewed bound.
    #[must_use]
    pub const fn is_bounded(self) -> bool {
        match self.validate() {
            Ok(()) => true,
            Err(_) => false,
        }
    }

    const fn validate(self) -> Result<(), GasEstimationError> {
        match self {
            Self::BackendStepLimit { max_backend_steps } => {
                if max_backend_steps == 0 {
                    return Err(GasEstimationError::UnboundedTerminationPolicy);
                }
                if max_backend_steps > MAX_GAS_ESTIMATION_BACKEND_STEPS {
                    return Err(GasEstimationError::TerminationLimitTooHigh);
                }
            }
            Self::WorkerTimeout { timeout_millis } | Self::WorkerIsolation { timeout_millis } => {
                if timeout_millis == 0 {
                    return Err(GasEstimationError::UnboundedTerminationPolicy);
                }
                if timeout_millis > MAX_GAS_ESTIMATION_TIMEOUT_MILLIS {
                    return Err(GasEstimationError::TerminationLimitTooHigh);
                }
            }
        }
        Ok(())
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
        if max_attempts > MAX_GAS_ESTIMATION_ATTEMPTS {
            return Err(GasEstimationError::AttemptLimitTooHigh);
        }
        if gas_cap.get() == 0 {
            return Err(GasEstimationError::ZeroGasCap);
        }
        if gas_cap.get() > MAX_GAS_ESTIMATION_GAS_CAP.get() {
            return Err(GasEstimationError::GasCapTooHigh);
        }
        if let Err(error) = termination.validate() {
            return Err(error);
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
    /// `max_attempts` exceeds the hard release ceiling.
    AttemptLimitTooHigh,
    /// `gas_cap` exceeds the hard release ceiling.
    GasCapTooHigh,
    /// Termination policy exceeds the hard release ceiling.
    TerminationLimitTooHigh,
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
            Self::AttemptLimitTooHigh => "ETH_EVM_GAS_ESTIMATION_ATTEMPT_LIMIT_TOO_HIGH",
            Self::GasCapTooHigh => "ETH_EVM_GAS_ESTIMATION_GAS_CAP_TOO_HIGH",
            Self::TerminationLimitTooHigh => "ETH_EVM_GAS_ESTIMATION_TERMINATION_LIMIT_TOO_HIGH",
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
            Self::AttemptLimitTooHigh => "gas-estimation maximum attempts exceed release ceiling",
            Self::GasCapTooHigh => "gas-estimation gas cap exceeds release ceiling",
            Self::TerminationLimitTooHigh => {
                "gas-estimation termination policy exceeds release ceiling"
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
