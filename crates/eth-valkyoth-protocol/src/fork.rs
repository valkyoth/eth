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
    /// Creates a chain spec from caller-reviewed fork entries.
    #[must_use]
    pub const fn new(chain_id: ChainId, forks: &'a [ForkSpec]) -> Self {
        Self { chain_id, forks }
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
        for fork in self.forks {
            if fork.hardfork == hardfork {
                if fork.chain_id != self.chain_id {
                    return Err(ForkError::ChainMismatch);
                }
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

    /// Returns the last active fork in the supplied fork order.
    pub fn active_fork(
        self,
        block_number: BlockNumber,
        timestamp: UnixTimestamp,
    ) -> Result<ForkSpec, ForkError> {
        let mut active = None;
        for fork in self.forks {
            if fork.chain_id != self.chain_id {
                return Err(ForkError::ChainMismatch);
            }
            if fork.is_active_at(block_number, timestamp) {
                active = Some(*fork);
            }
        }
        active.ok_or(ForkError::Inactive)
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
mod tests {
    use super::*;

    const TEST_CHAIN_ID: ChainId = ChainId::new(1);
    const TEST_FORKS: &[ForkSpec] = &[
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::Frontier,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(0),
            },
        },
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::Shanghai,
            activation: ForkActivation::BlockAndTimestamp {
                activation_block: BlockNumber::new(10),
                activation_timestamp: UnixTimestamp::new(20),
            },
        },
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::Cancun,
            activation: ForkActivation::BlockAndTimestamp {
                activation_block: BlockNumber::new(20),
                activation_timestamp: UnixTimestamp::new(30),
            },
        },
    ];
    const TEST_CHAIN: ChainSpec<'static> = ChainSpec::new(TEST_CHAIN_ID, TEST_FORKS);

    #[test]
    fn activation_requires_block_and_time() {
        let context = ValidationContext {
            fork: ForkSpec {
                chain_id: ChainId::new(1),
                hardfork: Hardfork::Shanghai,
                activation: ForkActivation::BlockAndTimestamp {
                    activation_block: BlockNumber::new(10),
                    activation_timestamp: UnixTimestamp::new(20),
                },
            },
            block_number: BlockNumber::new(10),
            timestamp: UnixTimestamp::new(19),
        };
        assert!(!context.fork_is_active());
    }

    #[test]
    fn block_only_activation_ignores_timestamp() {
        let context = ValidationContext {
            fork: ForkSpec {
                chain_id: ChainId::new(1),
                hardfork: Hardfork::Frontier,
                activation: ForkActivation::BlockOnly {
                    activation_block: BlockNumber::new(10),
                },
            },
            block_number: BlockNumber::new(10),
            timestamp: UnixTimestamp::new(0),
        };
        assert!(context.fork_is_active());
    }

    #[test]
    fn chain_spec_builds_context_only_for_active_fork() {
        assert_eq!(
            TEST_CHAIN.validation_context(
                Hardfork::Cancun,
                BlockNumber::new(20),
                UnixTimestamp::new(29)
            ),
            Err(ForkError::Inactive)
        );

        let context = TEST_CHAIN.validation_context(
            Hardfork::Cancun,
            BlockNumber::new(20),
            UnixTimestamp::new(30),
        );
        assert!(matches!(context, Ok(context) if context.fork.hardfork == Hardfork::Cancun));
    }

    #[test]
    fn active_fork_returns_last_active_entry() {
        let fork = TEST_CHAIN.active_fork(BlockNumber::new(20), UnixTimestamp::new(30));
        assert!(matches!(fork, Ok(fork) if fork.hardfork == Hardfork::Cancun));
    }

    #[test]
    fn unsupported_fork_is_explicit() {
        assert_eq!(
            TEST_CHAIN.fork_spec(Hardfork::Amsterdam),
            Err(ForkError::Unsupported)
        );
    }

    #[test]
    fn chain_mismatch_is_rejected() {
        let forks = [ForkSpec {
            chain_id: ChainId::new(5),
            hardfork: Hardfork::Frontier,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(0),
            },
        }];
        let spec = ChainSpec::new(TEST_CHAIN_ID, &forks);

        assert_eq!(
            spec.fork_spec(Hardfork::Frontier),
            Err(ForkError::ChainMismatch)
        );
    }
}
