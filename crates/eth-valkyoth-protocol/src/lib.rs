#![no_std]
#![forbid(unsafe_code)]
//! Fork-aware Ethereum protocol validation state.

#[cfg(feature = "std")]
extern crate std;

use core::fmt;

mod fork;
mod state;
mod transaction;

pub use fork::{ChainSpec, ForkActivation, ForkError, ForkSpec, Hardfork, ValidationContext};
pub use state::{
    Canonical, CanonicalValidationProof, Decoded, ForkValidated, ForkValidationProof,
    SenderRecovered, SenderRecoveryProof, StateTransitionError, Transaction,
};
pub use transaction::{
    ACCESS_LIST_TRANSACTION_FIELD_COUNT, ACCESS_LIST_TRANSACTION_TYPE, AccessList,
    AccessListEntries, AccessListEntry, AccessListStorageKeyItems, AccessListStorageKeys,
    AccessListTransactionDecodeError, AccessListTransactionDecodeErrorCategory,
    AccessListTransactionField, AccessListTransactionTo, BLOB_TRANSACTION_FIELD_COUNT,
    BLOB_TRANSACTION_TYPE, BlobTransactionDecodeError, BlobTransactionDecodeErrorCategory,
    BlobTransactionField, BlobVersionedHashItems, BlobVersionedHashes,
    DYNAMIC_FEE_TRANSACTION_FIELD_COUNT, DYNAMIC_FEE_TRANSACTION_TYPE,
    DynamicFeeTransactionDecodeError, DynamicFeeTransactionDecodeErrorCategory,
    DynamicFeeTransactionField, DynamicFeeTransactionTo, EIP_2718_MAX_TYPED_PREFIX,
    EIP_2718_RESERVED_PREFIX, EIP_2718_SCALAR_PREFIX_START, EIP_2718_TYPED_ZERO_PREFIX,
    InvalidSignatureYParity, LEGACY_TRANSACTION_FIELD_COUNT, LEGACY_TRANSACTION_PREFIX_START,
    LegacyTransactionDecodeError, LegacyTransactionDecodeErrorCategory, LegacyTransactionField,
    LegacyTransactionTo, SignatureYParity, TransactionEncodeError, TransactionEncodeErrorCategory,
    TransactionEnvelope, TransactionEnvelopeError, TransactionEnvelopeErrorCategory,
    TypedTransactionEnvelope, UnvalidatedAccessListTransaction, UnvalidatedBlobTransaction,
    UnvalidatedDynamicFeeTransaction, UnvalidatedLegacyTransaction, UnvalidatedTransaction,
    decode_access_list_transaction, decode_blob_transaction, decode_dynamic_fee_transaction,
    decode_legacy_transaction, decode_transaction_envelope, encode_access_list_signing_preimage,
    encode_access_list_transaction, encode_blob_signing_preimage, encode_blob_transaction,
    encode_dynamic_fee_signing_preimage, encode_dynamic_fee_transaction,
    encode_legacy_eip155_signing_preimage, encode_legacy_transaction, encode_transaction,
    encoded_access_list_signing_preimage_len, encoded_access_list_transaction_len,
    encoded_blob_signing_preimage_len, encoded_blob_transaction_len,
    encoded_dynamic_fee_signing_preimage_len, encoded_dynamic_fee_transaction_len,
    encoded_legacy_eip155_signing_preimage_len, encoded_legacy_transaction_len,
    encoded_transaction_len,
};

/// Protocol validation failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolError {
    /// A feature is disabled or unsupported for the selected operation.
    Feature(FeatureError),
    /// Fork context is missing, unsupported, or inactive.
    Fork(ForkError),
    /// A validation state transition was attempted out of order.
    InvalidStateTransition,
}

impl ProtocolError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Feature(error) => error.code(),
            Self::Fork(error) => error.code(),
            Self::InvalidStateTransition => "ETH_PROTOCOL_INVALID_STATE_TRANSITION",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Feature(error) => error.message(),
            Self::Fork(error) => error.message(),
            Self::InvalidStateTransition => {
                "validation state transition is not allowed from the current state"
            }
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> ProtocolErrorCategory {
        match self {
            Self::Feature(_) => ProtocolErrorCategory::Feature,
            Self::Fork(_) => ProtocolErrorCategory::Fork,
            Self::InvalidStateTransition => ProtocolErrorCategory::State,
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ProtocolError {}

/// Stable high-level protocol error categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolErrorCategory {
    /// Feature support or configuration failure.
    Feature,
    /// Fork selection or activation failure.
    Fork,
    /// Validation state failure.
    State,
}

/// Feature availability failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeatureError {
    /// The feature is intentionally not enabled for this build or context.
    Disabled,
    /// The feature is not implemented by this crate version.
    Unsupported,
}

impl FeatureError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Disabled => "ETH_FEATURE_DISABLED",
            Self::Unsupported => "ETH_FEATURE_UNSUPPORTED",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Disabled => "feature is disabled for this operation",
            Self::Unsupported => "feature is not supported by this crate version",
        }
    }
}

impl fmt::Display for FeatureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FeatureError {}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::string::ToString;

    #[test]
    fn protocol_errors_have_stable_codes_and_categories() {
        let error = ProtocolError::Fork(ForkError::Inactive);

        assert_eq!(error.code(), "ETH_FORK_INACTIVE");
        assert_eq!(
            error.message(),
            "fork is not active for the validation context"
        );
        assert_eq!(error.category(), ProtocolErrorCategory::Fork);
        assert_eq!(
            error.to_string(),
            "fork is not active for the validation context"
        );
    }

    #[test]
    fn feature_errors_format_without_payloads() {
        let error = ProtocolError::Feature(FeatureError::Unsupported);

        assert_eq!(error.code(), "ETH_FEATURE_UNSUPPORTED");
        assert_eq!(
            FeatureError::Unsupported.to_string(),
            "feature is not supported by this crate version"
        );
    }
}
