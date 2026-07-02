use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory};

/// Withdrawal field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WithdrawalField {
    /// Whole withdrawals list.
    List,
    /// One withdrawal entry.
    Withdrawal,
    /// Withdrawal global index.
    Index,
    /// Consensus-layer validator index.
    ValidatorIndex,
    /// Withdrawal recipient address.
    Address,
    /// Withdrawal amount in Gwei.
    Amount,
}

/// Withdrawal decode failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WithdrawalDecodeError {
    /// A withdrawal entry did not contain exactly four fields.
    WrongFieldCount {
        /// Expected field count.
        expected: usize,
        /// Actual field count.
        found: usize,
    },
    /// A field failed bounded RLP or primitive-domain decoding.
    FieldDecode {
        /// Field being decoded.
        field: WithdrawalField,
        /// Underlying decode error.
        source: DecodeError,
    },
    /// Withdrawal address was not exactly 20 bytes.
    InvalidAddressLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
    /// EIP-4895 withdrawal amounts must be nonzero.
    ZeroAmount,
}

impl WithdrawalDecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::WrongFieldCount { .. } => "ETH_WITHDRAWAL_WRONG_FIELD_COUNT",
            Self::FieldDecode { source, .. } => source.code(),
            Self::InvalidAddressLength { .. } => "ETH_WITHDRAWAL_INVALID_ADDRESS_LENGTH",
            Self::ZeroAmount => "ETH_WITHDRAWAL_ZERO_AMOUNT",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::WrongFieldCount { .. } => "withdrawal must contain exactly four RLP fields",
            Self::FieldDecode { .. } => "withdrawal field failed bounded decoding",
            Self::InvalidAddressLength { .. } => "withdrawal address must be exactly 20 bytes",
            Self::ZeroAmount => "withdrawal amount must be nonzero",
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> WithdrawalDecodeErrorCategory {
        match self {
            Self::WrongFieldCount { .. } | Self::InvalidAddressLength { .. } | Self::ZeroAmount => {
                WithdrawalDecodeErrorCategory::MalformedInput
            }
            Self::FieldDecode { source, .. } => match source.category() {
                DecodeErrorCategory::ResourceExhaustion => {
                    WithdrawalDecodeErrorCategory::ResourceExhaustion
                }
                _ => WithdrawalDecodeErrorCategory::MalformedInput,
            },
        }
    }
}

impl fmt::Display for WithdrawalDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WithdrawalDecodeError {}

/// Stable high-level withdrawal decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WithdrawalDecodeErrorCategory {
    /// Input is malformed for a withdrawals list or withdrawal entry.
    MalformedInput,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}
