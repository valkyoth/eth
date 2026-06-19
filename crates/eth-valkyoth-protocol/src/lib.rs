#![no_std]
#![forbid(unsafe_code)]
//! Fork-aware Ethereum protocol validation state.

use eth_valkyoth_primitives::{BlockNumber, ChainId, UnixTimestamp};

/// Ethereum fork rules selected for a validation operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForkSpec {
    /// Chain being validated.
    pub chain_id: ChainId,
    /// Activation block for this fork view.
    pub activation_block: BlockNumber,
    /// Activation timestamp for timestamp-based forks.
    pub activation_timestamp: Option<UnixTimestamp>,
    /// Whether this fork requires timestamp activation.
    pub requires_timestamp: bool,
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
    pub fn fork_is_active(self) -> Result<bool, ForkError> {
        if self.fork.requires_timestamp && self.fork.activation_timestamp.is_none() {
            return Err(ForkError::MissingTimestamp);
        }
        let block_active = self.block_number >= self.fork.activation_block;
        let time_active = match self.fork.activation_timestamp {
            Some(activation) => self.timestamp >= activation,
            None => true,
        };
        Ok(block_active && time_active)
    }
}

/// Fork validation configuration errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForkError {
    /// A timestamp-based fork was configured without an activation timestamp.
    MissingTimestamp,
}

/// Transaction validation state marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionState {
    /// Raw wire input was accepted by the codec.
    Decoded,
    /// Canonical wire form and type-specific structure were checked.
    Canonical,
    /// Fork-specific validity was checked.
    ForkValidated,
    /// Sender recovery succeeded.
    SenderRecovered,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_requires_block_and_time() {
        let context = ValidationContext {
            fork: ForkSpec {
                chain_id: ChainId::new(1),
                activation_block: BlockNumber::new(10),
                activation_timestamp: Some(UnixTimestamp::new(20)),
                requires_timestamp: true,
            },
            block_number: BlockNumber::new(10),
            timestamp: UnixTimestamp::new(19),
        };
        assert_eq!(context.fork_is_active(), Ok(false));
    }

    #[test]
    fn timestamp_fork_requires_timestamp() {
        let context = ValidationContext {
            fork: ForkSpec {
                chain_id: ChainId::new(1),
                activation_block: BlockNumber::new(10),
                activation_timestamp: None,
                requires_timestamp: true,
            },
            block_number: BlockNumber::new(10),
            timestamp: UnixTimestamp::new(20),
        };
        assert_eq!(context.fork_is_active(), Err(ForkError::MissingTimestamp));
    }
}
