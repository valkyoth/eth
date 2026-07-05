use core::fmt;

use eth_valkyoth_primitives::{Address, B256, ChainId, Gas, UnixTimestamp, Wei};
use eth_valkyoth_protocol::ValidationContext;

/// Block data required before any execution backend may run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockExecutionContext {
    /// Chain selected for execution.
    pub chain_id: ChainId,
    /// Current execution block number.
    pub block_number: eth_valkyoth_primitives::BlockNumber,
    /// Current execution block timestamp.
    pub timestamp: UnixTimestamp,
    /// Block beneficiary/coinbase address.
    pub beneficiary: Address,
    /// Block gas limit.
    pub gas_limit: Gas,
    /// Current EIP-1559 base fee.
    pub base_fee_per_gas: Wei,
    /// Previous randomness mix used by post-merge execution.
    pub prev_randao: B256,
}

/// Fully explicit execution environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionEnvironment {
    fork: ValidationContext,
    block: BlockExecutionContext,
}

impl ExecutionEnvironment {
    /// Creates an execution environment after checking fork/block consistency.
    pub fn try_new(
        fork: ValidationContext,
        block: BlockExecutionContext,
    ) -> Result<Self, ExecutionEnvironmentError> {
        if !fork.fork_is_active() {
            return Err(ExecutionEnvironmentError::InactiveFork);
        }
        if fork.fork.chain_id != block.chain_id {
            return Err(ExecutionEnvironmentError::ChainMismatch);
        }
        if fork.block_number != block.block_number {
            return Err(ExecutionEnvironmentError::BlockNumberMismatch);
        }
        if fork.timestamp != block.timestamp {
            return Err(ExecutionEnvironmentError::TimestampMismatch);
        }
        Ok(Self { fork, block })
    }

    /// Fork validation context used for this execution.
    #[must_use]
    pub const fn fork(self) -> ValidationContext {
        self.fork
    }

    /// Block execution context used for this execution.
    #[must_use]
    pub const fn block(self) -> BlockExecutionContext {
        self.block
    }
}

/// Execution environment construction failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionEnvironmentError {
    /// The selected fork is not active for the supplied block context.
    InactiveFork,
    /// Fork and block chain IDs differ.
    ChainMismatch,
    /// Fork and block numbers differ.
    BlockNumberMismatch,
    /// Fork and block timestamps differ.
    TimestampMismatch,
}

impl ExecutionEnvironmentError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InactiveFork => "ETH_EVM_INACTIVE_FORK",
            Self::ChainMismatch => "ETH_EVM_CHAIN_MISMATCH",
            Self::BlockNumberMismatch => "ETH_EVM_BLOCK_NUMBER_MISMATCH",
            Self::TimestampMismatch => "ETH_EVM_TIMESTAMP_MISMATCH",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::InactiveFork => "execution fork is not active for the block context",
            Self::ChainMismatch => "execution fork and block chain ids differ",
            Self::BlockNumberMismatch => "execution fork and block numbers differ",
            Self::TimestampMismatch => "execution fork and block timestamps differ",
        }
    }
}

impl fmt::Display for ExecutionEnvironmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ExecutionEnvironmentError {}
