use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory};

use super::DynamicFeeTransactionField;
use crate::transaction::{TransactionEnvelopeError, TransactionEnvelopeErrorCategory};

/// EIP-1559 dynamic-fee transaction decode failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynamicFeeTransactionDecodeError {
    /// Envelope classification failed before dynamic-fee fields could decode.
    Envelope(TransactionEnvelopeError),
    /// A non-EIP-1559 envelope was supplied to this decoder.
    WrongTransactionType {
        /// Observed transaction type, or legacy zero.
        type_byte: u8,
    },
    /// Transaction payload did not contain exactly twelve fields.
    WrongFieldCount {
        /// Expected field count.
        expected: usize,
        /// Actual field count.
        found: usize,
    },
    /// A field failed RLP or primitive-domain decoding.
    FieldDecode {
        /// Field being decoded.
        field: DynamicFeeTransactionField,
        /// Underlying decode error.
        source: DecodeError,
    },
    /// The `to` field was neither empty nor a 20-byte address.
    InvalidToLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
    /// Signature y parity was not `0` or `1`.
    InvalidYParity {
        /// Observed y-parity integer.
        value: u64,
    },
    /// Access-list entry was not `[address, storageKeys]`.
    InvalidAccessListEntryFieldCount {
        /// Actual entry field count.
        found: usize,
    },
    /// Access-list address was not a 20-byte scalar.
    InvalidAccessListAddressLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
    /// Access-list storage key was not a 32-byte scalar.
    InvalidStorageKeyLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
}

impl DynamicFeeTransactionDecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.code(),
            Self::WrongTransactionType { .. } => "ETH_DYNAMIC_FEE_TX_WRONG_TYPE",
            Self::WrongFieldCount { .. } => "ETH_DYNAMIC_FEE_TX_WRONG_FIELD_COUNT",
            Self::FieldDecode { source, .. } => source.code(),
            Self::InvalidToLength { .. } => "ETH_DYNAMIC_FEE_TX_INVALID_TO_LENGTH",
            Self::InvalidYParity { .. } => "ETH_DYNAMIC_FEE_TX_INVALID_Y_PARITY",
            Self::InvalidAccessListEntryFieldCount { .. } => {
                "ETH_DYNAMIC_FEE_TX_INVALID_ENTRY_FIELD_COUNT"
            }
            Self::InvalidAccessListAddressLength { .. } => {
                "ETH_DYNAMIC_FEE_TX_INVALID_ADDRESS_LENGTH"
            }
            Self::InvalidStorageKeyLength { .. } => "ETH_DYNAMIC_FEE_TX_INVALID_STORAGE_KEY_LENGTH",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.message(),
            Self::WrongTransactionType { .. } => {
                "dynamic-fee transaction decoder received a different envelope type"
            }
            Self::WrongFieldCount { .. } => {
                "dynamic-fee transaction must contain exactly twelve RLP fields"
            }
            Self::FieldDecode { .. } => "dynamic-fee transaction field failed bounded decoding",
            Self::InvalidToLength { .. } => {
                "dynamic-fee transaction to field must be empty or a 20-byte address"
            }
            Self::InvalidYParity { .. } => "dynamic-fee transaction y parity must be 0 or 1",
            Self::InvalidAccessListEntryFieldCount { .. } => {
                "dynamic-fee access-list entry must contain exactly address and storage-key list"
            }
            Self::InvalidAccessListAddressLength { .. } => {
                "dynamic-fee access-list address must be a 20-byte scalar"
            }
            Self::InvalidStorageKeyLength { .. } => {
                "dynamic-fee access-list storage key must be a 32-byte scalar"
            }
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> DynamicFeeTransactionDecodeErrorCategory {
        match self {
            Self::Envelope(error) => match error.category() {
                TransactionEnvelopeErrorCategory::ResourceExhaustion => {
                    DynamicFeeTransactionDecodeErrorCategory::ResourceExhaustion
                }
                TransactionEnvelopeErrorCategory::Unsupported => {
                    DynamicFeeTransactionDecodeErrorCategory::WrongType
                }
                _ => DynamicFeeTransactionDecodeErrorCategory::MalformedInput,
            },
            Self::WrongTransactionType { .. } => {
                DynamicFeeTransactionDecodeErrorCategory::WrongType
            }
            Self::FieldDecode { source, .. } => match source.category() {
                DecodeErrorCategory::ResourceExhaustion => {
                    DynamicFeeTransactionDecodeErrorCategory::ResourceExhaustion
                }
                _ => DynamicFeeTransactionDecodeErrorCategory::MalformedInput,
            },
            Self::WrongFieldCount { .. }
            | Self::InvalidToLength { .. }
            | Self::InvalidYParity { .. }
            | Self::InvalidAccessListEntryFieldCount { .. }
            | Self::InvalidAccessListAddressLength { .. }
            | Self::InvalidStorageKeyLength { .. } => {
                DynamicFeeTransactionDecodeErrorCategory::MalformedInput
            }
        }
    }
}

impl fmt::Display for DynamicFeeTransactionDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DynamicFeeTransactionDecodeError {}

/// Stable high-level dynamic-fee transaction decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynamicFeeTransactionDecodeErrorCategory {
    /// Input is malformed for an EIP-1559 transaction.
    MalformedInput,
    /// A non-EIP-1559 transaction envelope was supplied.
    WrongType,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}
