use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeErrorCategory};

use super::SetCodeTransactionField;
use crate::transaction::{TransactionEnvelopeError, TransactionEnvelopeErrorCategory};

/// EIP-7702 authorization tuple sub-field identifier.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetCodeAuthorizationField {
    /// Authorization chain ID.
    ChainId,
    /// Authorized account address.
    Address,
    /// Authorized account nonce.
    Nonce,
    /// Authorization signature y parity.
    YParity,
    /// Authorization signature `r`.
    R,
    /// Authorization signature `s`.
    S,
}

/// EIP-7702 set-code transaction decode failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetCodeTransactionDecodeError {
    /// Envelope classification failed before set-code fields could decode.
    Envelope(TransactionEnvelopeError),
    /// A non-EIP-7702 envelope was supplied to this decoder.
    WrongTransactionType {
        /// Observed transaction type, or legacy zero.
        type_byte: u8,
    },
    /// Transaction payload did not contain exactly thirteen fields.
    WrongFieldCount {
        /// Expected field count.
        expected: usize,
        /// Actual field count.
        found: usize,
    },
    /// A field failed RLP or primitive-domain decoding.
    FieldDecode {
        /// Field being decoded.
        field: SetCodeTransactionField,
        /// Underlying decode error.
        source: DecodeError,
    },
    /// An authorization tuple sub-field failed RLP or primitive-domain decoding.
    AuthorizationFieldDecode {
        /// Authorization tuple sub-field being decoded.
        field: SetCodeAuthorizationField,
        /// Underlying decode error.
        source: DecodeError,
    },
    /// The `destination` field was not a 20-byte address.
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
    /// Authorization tuple did not contain exactly six fields.
    InvalidAuthorizationFieldCount {
        /// Actual tuple field count.
        found: usize,
    },
    /// Authorization address was not a 20-byte scalar.
    InvalidAuthorizationAddressLength {
        /// Actual decoded scalar byte length.
        found: usize,
    },
    /// Authorization y parity was not `0` or `1`.
    InvalidAuthorizationYParity {
        /// Observed y-parity integer.
        value: u64,
    },
}

impl SetCodeTransactionDecodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.code(),
            Self::WrongTransactionType { .. } => "ETH_SET_CODE_TX_WRONG_TYPE",
            Self::WrongFieldCount { .. } => "ETH_SET_CODE_TX_WRONG_FIELD_COUNT",
            Self::FieldDecode { source, .. } => source.code(),
            Self::AuthorizationFieldDecode { source, .. } => source.code(),
            Self::InvalidToLength { .. } => "ETH_SET_CODE_TX_INVALID_TO_LENGTH",
            Self::InvalidYParity { .. } => "ETH_SET_CODE_TX_INVALID_Y_PARITY",
            Self::InvalidAccessListEntryFieldCount { .. } => {
                "ETH_SET_CODE_TX_INVALID_ENTRY_FIELD_COUNT"
            }
            Self::InvalidAccessListAddressLength { .. } => {
                "ETH_SET_CODE_TX_INVALID_ACCESS_ADDRESS_LENGTH"
            }
            Self::InvalidStorageKeyLength { .. } => "ETH_SET_CODE_TX_INVALID_STORAGE_KEY_LENGTH",
            Self::InvalidAuthorizationFieldCount { .. } => {
                "ETH_SET_CODE_TX_INVALID_AUTH_FIELD_COUNT"
            }
            Self::InvalidAuthorizationAddressLength { .. } => {
                "ETH_SET_CODE_TX_INVALID_AUTH_ADDRESS_LENGTH"
            }
            Self::InvalidAuthorizationYParity { .. } => "ETH_SET_CODE_TX_INVALID_AUTH_Y_PARITY",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Envelope(error) => error.message(),
            Self::WrongTransactionType { .. } => {
                "set-code transaction decoder received a different envelope type"
            }
            Self::WrongFieldCount { .. } => {
                "set-code transaction must contain exactly thirteen RLP fields"
            }
            Self::FieldDecode { .. } => "set-code transaction field failed bounded decoding",
            Self::AuthorizationFieldDecode { .. } => {
                "set-code authorization tuple field failed bounded decoding"
            }
            Self::InvalidToLength { .. } => {
                "set-code transaction destination field must be a 20-byte address"
            }
            Self::InvalidYParity { .. } => "set-code transaction y parity must be 0 or 1",
            Self::InvalidAccessListEntryFieldCount { .. } => {
                "set-code transaction access-list entry must contain exactly address and storage-key list"
            }
            Self::InvalidAccessListAddressLength { .. } => {
                "set-code transaction access-list address must be a 20-byte scalar"
            }
            Self::InvalidStorageKeyLength { .. } => {
                "set-code transaction access-list storage key must be a 32-byte scalar"
            }
            Self::InvalidAuthorizationFieldCount { .. } => {
                "set-code authorization tuple must contain exactly chain id, address, nonce, y parity, r, and s"
            }
            Self::InvalidAuthorizationAddressLength { .. } => {
                "set-code authorization address must be a 20-byte scalar"
            }
            Self::InvalidAuthorizationYParity { .. } => {
                "set-code authorization y parity must be 0 or 1"
            }
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> SetCodeTransactionDecodeErrorCategory {
        match self {
            Self::Envelope(error) => match error.category() {
                TransactionEnvelopeErrorCategory::ResourceExhaustion => {
                    SetCodeTransactionDecodeErrorCategory::ResourceExhaustion
                }
                TransactionEnvelopeErrorCategory::Unsupported => {
                    SetCodeTransactionDecodeErrorCategory::WrongType
                }
                _ => SetCodeTransactionDecodeErrorCategory::MalformedInput,
            },
            Self::WrongTransactionType { .. } => SetCodeTransactionDecodeErrorCategory::WrongType,
            Self::FieldDecode { source, .. } | Self::AuthorizationFieldDecode { source, .. } => {
                match source.category() {
                    DecodeErrorCategory::ResourceExhaustion => {
                        SetCodeTransactionDecodeErrorCategory::ResourceExhaustion
                    }
                    _ => SetCodeTransactionDecodeErrorCategory::MalformedInput,
                }
            }
            Self::WrongFieldCount { .. }
            | Self::InvalidToLength { .. }
            | Self::InvalidYParity { .. }
            | Self::InvalidAccessListEntryFieldCount { .. }
            | Self::InvalidAccessListAddressLength { .. }
            | Self::InvalidStorageKeyLength { .. }
            | Self::InvalidAuthorizationFieldCount { .. }
            | Self::InvalidAuthorizationAddressLength { .. }
            | Self::InvalidAuthorizationYParity { .. } => {
                SetCodeTransactionDecodeErrorCategory::MalformedInput
            }
        }
    }
}

impl fmt::Display for SetCodeTransactionDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SetCodeTransactionDecodeError {}

/// Stable high-level set-code transaction decode error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetCodeTransactionDecodeErrorCategory {
    /// Input is malformed for an EIP-7702 transaction.
    MalformedInput,
    /// A non-EIP-7702 transaction envelope was supplied.
    WrongType,
    /// The active decode policy rejected the input as too large or too deep.
    ResourceExhaustion,
}
