use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory};

use super::AccessListTransactionField;
use crate::transaction::{TransactionEnvelopeError, TransactionEnvelopeErrorCategory};

/// EIP-2930 access-list transaction decode failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessListTransactionDecodeError {
    /// Envelope classification failed before access-list fields could decode.
    Envelope(TransactionEnvelopeError),
    /// A non-EIP-2930 envelope was supplied to this decoder.
    WrongTransactionType {
        /// Observed transaction type, or legacy zero.
        type_byte: u8,
    },
    /// Transaction payload did not contain exactly eleven fields.
    WrongFieldCount {
        /// Expected field count.
        expected: usize,
        /// Actual field count.
        found: usize,
    },
    /// A field failed RLP or primitive-domain decoding.
    FieldDecode {
        /// Field being decoded.
        field: AccessListTransactionField,
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

impl AccessListTransactionDecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.code(),
            Self::WrongTransactionType { .. } => "ETH_ACCESS_LIST_TX_WRONG_TYPE",
            Self::WrongFieldCount { .. } => "ETH_ACCESS_LIST_TX_WRONG_FIELD_COUNT",
            Self::FieldDecode { source, .. } => source.code(),
            Self::InvalidToLength { .. } => "ETH_ACCESS_LIST_TX_INVALID_TO_LENGTH",
            Self::InvalidYParity { .. } => "ETH_ACCESS_LIST_TX_INVALID_Y_PARITY",
            Self::InvalidAccessListEntryFieldCount { .. } => {
                "ETH_ACCESS_LIST_TX_INVALID_ENTRY_FIELD_COUNT"
            }
            Self::InvalidAccessListAddressLength { .. } => {
                "ETH_ACCESS_LIST_TX_INVALID_ADDRESS_LENGTH"
            }
            Self::InvalidStorageKeyLength { .. } => "ETH_ACCESS_LIST_TX_INVALID_STORAGE_KEY_LENGTH",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.message(),
            Self::WrongTransactionType { .. } => {
                "access-list transaction decoder received a different envelope type"
            }
            Self::WrongFieldCount { .. } => {
                "access-list transaction must contain exactly eleven RLP fields"
            }
            Self::FieldDecode { .. } => "access-list transaction field failed bounded decoding",
            Self::InvalidToLength { .. } => {
                "access-list transaction to field must be empty or a 20-byte address"
            }
            Self::InvalidYParity { .. } => "access-list transaction y parity must be 0 or 1",
            Self::InvalidAccessListEntryFieldCount { .. } => {
                "access-list entry must contain exactly address and storage-key list"
            }
            Self::InvalidAccessListAddressLength { .. } => {
                "access-list address must be a 20-byte scalar"
            }
            Self::InvalidStorageKeyLength { .. } => {
                "access-list storage key must be a 32-byte scalar"
            }
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> AccessListTransactionDecodeErrorCategory {
        match self {
            Self::Envelope(error) => match error.category() {
                TransactionEnvelopeErrorCategory::ResourceExhaustion => {
                    AccessListTransactionDecodeErrorCategory::ResourceExhaustion
                }
                TransactionEnvelopeErrorCategory::Unsupported => {
                    AccessListTransactionDecodeErrorCategory::WrongType
                }
                _ => AccessListTransactionDecodeErrorCategory::MalformedInput,
            },
            Self::WrongTransactionType { .. } => {
                AccessListTransactionDecodeErrorCategory::WrongType
            }
            Self::FieldDecode { source, .. } => match source.category() {
                DecodeErrorCategory::ResourceExhaustion => {
                    AccessListTransactionDecodeErrorCategory::ResourceExhaustion
                }
                _ => AccessListTransactionDecodeErrorCategory::MalformedInput,
            },
            Self::WrongFieldCount { .. }
            | Self::InvalidToLength { .. }
            | Self::InvalidYParity { .. }
            | Self::InvalidAccessListEntryFieldCount { .. }
            | Self::InvalidAccessListAddressLength { .. }
            | Self::InvalidStorageKeyLength { .. } => {
                AccessListTransactionDecodeErrorCategory::MalformedInput
            }
        }
    }
}

impl fmt::Display for AccessListTransactionDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AccessListTransactionDecodeError {}

/// Stable high-level access-list transaction decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessListTransactionDecodeErrorCategory {
    /// Input is malformed for an EIP-2930 transaction.
    MalformedInput,
    /// A non-EIP-2930 transaction envelope was supplied.
    WrongType,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}
