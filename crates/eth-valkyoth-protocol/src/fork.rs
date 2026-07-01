use core::fmt;

use eth_valkyoth_primitives::{BlockNumber, ChainId, UnixTimestamp};

/// Execution-layer hardfork identity used by explicit chain specs.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Hardfork {
    /// Frontier genesis rules.
    Frontier,
    /// Homestead rules.
    Homestead,
    /// Byzantium rules.
    Byzantium,
    /// London rules.
    London,
    /// Shanghai rules.
    Shanghai,
    /// Cancun rules.
    Cancun,
    /// Prague/Pectra rules.
    Prague,
    /// Amsterdam rules.
    Amsterdam,
}

/// Fork selection or activation failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForkError {
    /// The selected fork is absent from the supplied chain spec.
    Unsupported,
    /// The selected fork is not active for the supplied validation context.
    Inactive,
    /// Fork activation data is incomplete.
    MissingActivation,
    /// A fork spec was supplied for a different chain.
    ChainMismatch,
    /// The chain spec contains the same hardfork more than once.
    DuplicateFork,
    /// The chain spec hardfork entries are not in chronological order.
    NonMonotonicForkOrder,
}

impl ForkError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Unsupported => "ETH_FORK_UNSUPPORTED",
            Self::Inactive => "ETH_FORK_INACTIVE",
            Self::MissingActivation => "ETH_FORK_MISSING_ACTIVATION",
            Self::ChainMismatch => "ETH_FORK_CHAIN_MISMATCH",
            Self::DuplicateFork => "ETH_FORK_DUPLICATE",
            Self::NonMonotonicForkOrder => "ETH_FORK_NON_MONOTONIC_ORDER",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Unsupported => "fork is not supported by this crate version",
            Self::Inactive => "fork is not active for the validation context",
            Self::MissingActivation => "fork activation data is incomplete",
            Self::ChainMismatch => "fork chain id does not match the selected chain spec",
            Self::DuplicateFork => "chain spec contains a duplicate hardfork entry",
            Self::NonMonotonicForkOrder => "chain spec fork entries are not monotonic",
        }
    }
}

impl fmt::Display for ForkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ForkError {}

/// Unambiguous fork activation rule.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForkActivation {
    /// Block number alone determines activation.
    BlockOnly {
        /// Activation block for this fork view.
        activation_block: BlockNumber,
    },
    /// Both block number and timestamp must be satisfied.
    BlockAndTimestamp {
        /// Activation block for this fork view.
        activation_block: BlockNumber,
        /// Activation timestamp for timestamp-based forks.
        activation_timestamp: UnixTimestamp,
    },
}

/// Ethereum fork rules selected for a validation operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForkSpec {
    /// Chain being validated.
    pub chain_id: ChainId,
    /// Hardfork these activation rules identify.
    pub hardfork: Hardfork,
    /// Activation rule for this fork view.
    pub activation: ForkActivation,
}

impl ForkSpec {
    /// Returns whether this fork is active at the supplied block and timestamp.
    #[must_use]
    pub fn is_active_at(self, block_number: BlockNumber, timestamp: UnixTimestamp) -> bool {
        match self.activation {
            ForkActivation::BlockOnly { activation_block } => block_number >= activation_block,
            ForkActivation::BlockAndTimestamp {
                activation_block,
                activation_timestamp,
            } => block_number >= activation_block && timestamp >= activation_timestamp,
        }
    }
}

/// Explicit chain rules used to select fork validation context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChainSpec<'a> {
    chain_id: ChainId,
    forks: &'a [ForkSpec],
}

impl<'a> ChainSpec<'a> {
    /// Creates a chain spec from caller-reviewed static fork entries.
    ///
    /// This constructor is intentionally `const` for hand-audited static
    /// tables. Use [`Self::try_new`] when entries come from dynamic config,
    /// generated data, or any source that is not reviewed in code. Selection
    /// methods still reject duplicate, unordered, or wrong-chain entries before
    /// returning fork context.
    #[must_use]
    pub const fn new(chain_id: ChainId, forks: &'a [ForkSpec]) -> Self {
        Self { chain_id, forks }
    }

    /// Creates a chain spec after validating ordering, uniqueness, and chain ID.
    pub fn try_new(chain_id: ChainId, forks: &'a [ForkSpec]) -> Result<Self, ForkError> {
        let spec = Self { chain_id, forks };
        spec.validate()?;
        Ok(spec)
    }

    /// Chain ID this spec validates.
    #[must_use]
    pub const fn chain_id(self) -> ChainId {
        self.chain_id
    }

    /// Fork entries in caller-reviewed activation order.
    #[must_use]
    pub const fn forks(self) -> &'a [ForkSpec] {
        self.forks
    }

    /// Returns the spec for a hardfork admitted by this chain spec.
    pub fn fork_spec(self, hardfork: Hardfork) -> Result<ForkSpec, ForkError> {
        self.validate()?;
        for fork in self.forks {
            if fork.hardfork == hardfork {
                return Ok(*fork);
            }
        }
        Err(ForkError::Unsupported)
    }

    /// Builds an explicit validation context for an active hardfork.
    pub fn validation_context(
        self,
        hardfork: Hardfork,
        block_number: BlockNumber,
        timestamp: UnixTimestamp,
    ) -> Result<ValidationContext, ForkError> {
        let fork = self.fork_spec(hardfork)?;
        let context = ValidationContext {
            fork,
            block_number,
            timestamp,
        };
        if context.fork_is_active() {
            Ok(context)
        } else {
            Err(ForkError::Inactive)
        }
    }

    /// Returns the highest active hardfork in this chain spec.
    pub fn active_fork(
        self,
        block_number: BlockNumber,
        timestamp: UnixTimestamp,
    ) -> Result<ForkSpec, ForkError> {
        let mut active: Option<ForkSpec> = None;
        self.validate()?;
        for fork in self.forks {
            if fork.is_active_at(block_number, timestamp) {
                active = match active {
                    Some(previous) if previous.hardfork > fork.hardfork => Some(previous),
                    _ => Some(*fork),
                };
            }
        }
        active.ok_or(ForkError::Inactive)
    }

    fn validate(self) -> Result<(), ForkError> {
        let mut previous: Option<ForkSpec> = None;
        for fork in self.forks {
            if fork.chain_id != self.chain_id {
                return Err(ForkError::ChainMismatch);
            }
            if let Some(previous_fork) = previous {
                if fork.hardfork == previous_fork.hardfork {
                    return Err(ForkError::DuplicateFork);
                }
                if fork.hardfork < previous_fork.hardfork
                    || !fork_activation_is_monotonic(previous_fork.activation, fork.activation)
                {
                    return Err(ForkError::NonMonotonicForkOrder);
                }
            }
            previous = Some(*fork);
        }
        Ok(())
    }
}

fn fork_activation_is_monotonic(previous: ForkActivation, next: ForkActivation) -> bool {
    let previous_block = fork_activation_block(previous);
    let next_block = fork_activation_block(next);
    if next_block < previous_block {
        return false;
    }

    match (previous, next) {
        (
            ForkActivation::BlockAndTimestamp {
                activation_timestamp: previous_timestamp,
                ..
            },
            ForkActivation::BlockAndTimestamp {
                activation_timestamp: next_timestamp,
                ..
            },
        ) => next_timestamp >= previous_timestamp,
        _ => true,
    }
}

fn fork_activation_block(activation: ForkActivation) -> BlockNumber {
    match activation {
        ForkActivation::BlockOnly { activation_block }
        | ForkActivation::BlockAndTimestamp {
            activation_block, ..
        } => activation_block,
    }
}

/// Validation context that must be explicit for consensus-sensitive operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationContext {
    /// Fork rules.
    pub fork: ForkSpec,
    /// Current block number.
    pub block_number: BlockNumber,
    /// Current block timestamp.
    pub timestamp: UnixTimestamp,
}

impl ValidationContext {
    /// Returns whether the configured fork is active for this context.
    #[must_use]
    pub fn fork_is_active(self) -> bool {
        self.fork.is_active_at(self.block_number, self.timestamp)
    }
}

#[cfg(test)]
#[path = "fork_tests.rs"]
mod tests;
