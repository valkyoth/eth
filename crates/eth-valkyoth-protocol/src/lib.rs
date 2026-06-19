#![no_std]
#![forbid(unsafe_code)]
//! Fork-aware Ethereum protocol validation state.

use core::marker::PhantomData;

use eth_valkyoth_primitives::{BlockNumber, ChainId, UnixTimestamp};

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
    /// Activation rule for this fork view.
    pub activation: ForkActivation,
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
        match self.fork.activation {
            ForkActivation::BlockOnly { activation_block } => self.block_number >= activation_block,
            ForkActivation::BlockAndTimestamp {
                activation_block,
                activation_timestamp,
            } => self.block_number >= activation_block && self.timestamp >= activation_timestamp,
        }
    }
}

/// Raw wire input was accepted by the codec.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Decoded;

/// Canonical wire form and type-specific structure were checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Canonical;

/// Fork-specific validity was checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForkValidated;

/// Sender recovery succeeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SenderRecovered;

/// A transaction token whose validation state is tracked at compile time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transaction<State> {
    _state: PhantomData<State>,
}

impl Transaction<Decoded> {
    /// Creates a token for a decoded transaction.
    #[must_use]
    pub const fn decoded() -> Self {
        Self {
            _state: PhantomData,
        }
    }

    /// Advances to canonical form after canonical checks pass.
    #[must_use]
    pub const fn into_canonical(self) -> Transaction<Canonical> {
        Transaction {
            _state: PhantomData,
        }
    }
}

impl Transaction<Canonical> {
    /// Advances after fork-specific validation passes.
    #[must_use]
    pub const fn into_fork_validated(self) -> Transaction<ForkValidated> {
        Transaction {
            _state: PhantomData,
        }
    }
}

impl Transaction<ForkValidated> {
    /// Advances after sender recovery succeeds.
    #[must_use]
    pub const fn into_sender_recovered(self) -> Transaction<SenderRecovered> {
        Transaction {
            _state: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_requires_block_and_time() {
        let context = ValidationContext {
            fork: ForkSpec {
                chain_id: ChainId::new(1),
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
    fn transaction_typestate_advances_in_order() {
        let transaction = Transaction::decoded()
            .into_canonical()
            .into_fork_validated()
            .into_sender_recovered();
        assert_eq!(
            transaction,
            Transaction::<SenderRecovered> {
                _state: PhantomData
            }
        );
    }
}
