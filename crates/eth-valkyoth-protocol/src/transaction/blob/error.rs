use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory};

use super::BlobTransactionField;
use crate::transaction::{TransactionEnvelopeError, TransactionEnvelopeErrorCategory};

/// EIP-4844 blob transaction decode failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobTransactionDecodeError {
    /// Envelope classification failed before blob fields could decode.
    Envelope(TransactionEnvelopeError),
    /// A non-EIP-4844 envelope was supplied to this decoder.
    WrongTransactionType {
        /// Observed transaction type, or legacy zero.
        type_byte: u8,
    },
    /// Transaction payload did not contain exactly fourteen fields.
    WrongFieldCount {
        /// Expected field count.
        expected: usize,
        /// Actual field count.
        found: usize,
    },
    /// A field failed RLP or primitive-domain decoding.
    FieldDecode {
        /// Field being decoded.
        field: BlobTransactionField,
        /// Underlying decode error.
        source: DecodeError,
    },
    /// The `to` field was not a 20-byte address.
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
    /// Blob versioned hash was not a 32-byte scalar.
    InvalidBlobVersionedHashLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
}

impl BlobTransactionDecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.code(),
            Self::WrongTransactionType { .. } => "ETH_BLOB_TX_WRONG_TYPE",
            Self::WrongFieldCount { .. } => "ETH_BLOB_TX_WRONG_FIELD_COUNT",
            Self::FieldDecode { source, .. } => source.code(),
            Self::InvalidToLength { .. } => "ETH_BLOB_TX_INVALID_TO_LENGTH",
            Self::InvalidYParity { .. } => "ETH_BLOB_TX_INVALID_Y_PARITY",
            Self::InvalidAccessListEntryFieldCount { .. } => {
                "ETH_BLOB_TX_INVALID_ENTRY_FIELD_COUNT"
            }
            Self::InvalidAccessListAddressLength { .. } => "ETH_BLOB_TX_INVALID_ADDRESS_LENGTH",
            Self::InvalidStorageKeyLength { .. } => "ETH_BLOB_TX_INVALID_STORAGE_KEY_LENGTH",
            Self::InvalidBlobVersionedHashLength { .. } => {
                "ETH_BLOB_TX_INVALID_VERSIONED_HASH_LENGTH"
            }
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.message(),
            Self::WrongTransactionType { .. } => {
                "blob transaction decoder received a different envelope type"
            }
            Self::WrongFieldCount { .. } => {
                "blob transaction must contain exactly fourteen RLP fields"
            }
            Self::FieldDecode { .. } => "blob transaction field failed bounded decoding",
            Self::InvalidToLength { .. } => "blob transaction to field must be a 20-byte address",
            Self::InvalidYParity { .. } => "blob transaction y parity must be 0 or 1",
            Self::InvalidAccessListEntryFieldCount { .. } => {
                "blob transaction access-list entry must contain exactly address and storage-key list"
            }
            Self::InvalidAccessListAddressLength { .. } => {
                "blob transaction access-list address must be a 20-byte scalar"
            }
            Self::InvalidStorageKeyLength { .. } => {
                "blob transaction access-list storage key must be a 32-byte scalar"
            }
            Self::InvalidBlobVersionedHashLength { .. } => {
                "blob transaction versioned hash must be a 32-byte scalar"
            }
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> BlobTransactionDecodeErrorCategory {
        match self {
            Self::Envelope(error) => match error.category() {
                TransactionEnvelopeErrorCategory::ResourceExhaustion => {
                    BlobTransactionDecodeErrorCategory::ResourceExhaustion
                }
                TransactionEnvelopeErrorCategory::Unsupported => {
                    BlobTransactionDecodeErrorCategory::WrongType
                }
                _ => BlobTransactionDecodeErrorCategory::MalformedInput,
            },
            Self::WrongTransactionType { .. } => BlobTransactionDecodeErrorCategory::WrongType,
            Self::FieldDecode { source, .. } => match source.category() {
                DecodeErrorCategory::ResourceExhaustion => {
                    BlobTransactionDecodeErrorCategory::ResourceExhaustion
                }
                _ => BlobTransactionDecodeErrorCategory::MalformedInput,
            },
            Self::WrongFieldCount { .. }
            | Self::InvalidToLength { .. }
            | Self::InvalidYParity { .. }
            | Self::InvalidAccessListEntryFieldCount { .. }
            | Self::InvalidAccessListAddressLength { .. }
            | Self::InvalidStorageKeyLength { .. }
            | Self::InvalidBlobVersionedHashLength { .. } => {
                BlobTransactionDecodeErrorCategory::MalformedInput
            }
        }
    }
}

impl fmt::Display for BlobTransactionDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BlobTransactionDecodeError {}

/// Stable high-level blob transaction decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobTransactionDecodeErrorCategory {
    /// Input is malformed for an EIP-4844 transaction.
    MalformedInput,
    /// A non-EIP-4844 transaction envelope was supplied.
    WrongType,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}
