use core::fmt;

use eth_valkyoth_codec::DecodeError;

use crate::MptProofVerificationError;

/// Field in the canonical Ethereum account tuple.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountField {
    /// Account transaction nonce.
    Nonce,
    /// Account balance.
    Balance,
    /// Account storage trie root.
    StorageRoot,
    /// Account bytecode hash.
    CodeHash,
}

/// Canonical Ethereum account decoding failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountDecodeError {
    /// The outer account RLP or an integer field was malformed.
    Rlp(DecodeError),
    /// The tuple did not contain exactly four fields.
    FieldCount {
        /// Number of fields found.
        found: usize,
    },
    /// A field that must be a scalar was encoded as a list.
    UnexpectedList {
        /// Field with the invalid shape.
        field: AccountField,
    },
    /// A field failed bounded RLP traversal or integer conversion.
    FieldRlp {
        /// Field that failed.
        field: AccountField,
        /// Underlying codec failure.
        source: DecodeError,
    },
    /// A fixed-width hash had the wrong payload size.
    FixedWidth {
        /// Field with the invalid width.
        field: AccountField,
        /// Required byte width.
        expected: usize,
        /// Actual byte width.
        found: usize,
    },
}

impl fmt::Display for AccountDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rlp(_) => formatter.write_str("canonical account RLP is malformed"),
            Self::FieldCount { .. } => {
                formatter.write_str("canonical account must contain exactly four fields")
            }
            Self::UnexpectedList { .. } => {
                formatter.write_str("canonical account field must be an RLP scalar")
            }
            Self::FieldRlp { .. } => {
                formatter.write_str("canonical account field failed RLP decoding")
            }
            Self::FixedWidth { .. } => {
                formatter.write_str("canonical account hash field has the wrong width")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AccountDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rlp(error) | Self::FieldRlp { source: error, .. } => Some(error),
            Self::FieldCount { .. } | Self::UnexpectedList { .. } | Self::FixedWidth { .. } => None,
        }
    }
}

/// Composed account or storage proof verification failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateProofVerificationError {
    /// The Merkle Patricia proof failed.
    Proof(MptProofVerificationError),
    /// The proven account value was not a canonical Ethereum account tuple.
    Account(AccountDecodeError),
    /// A proven storage value was not a canonical RLP U256 integer.
    StorageValue(DecodeError),
    /// A zero storage value was explicitly present instead of canonically absent.
    ExplicitZeroStorageValue,
    /// Noncryptographic planning and verified traversal reached different values.
    InconsistentProofTraversal,
}

impl StateProofVerificationError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Proof(error) => error.code(),
            Self::Account(AccountDecodeError::Rlp(error))
            | Self::Account(AccountDecodeError::FieldRlp { source: error, .. }) => error.code(),
            Self::Account(AccountDecodeError::FieldCount { .. }) => "ETH_STATE_ACCOUNT_FIELD_COUNT",
            Self::Account(AccountDecodeError::UnexpectedList { .. }) => {
                "ETH_STATE_ACCOUNT_UNEXPECTED_LIST"
            }
            Self::Account(AccountDecodeError::FixedWidth { .. }) => "ETH_STATE_ACCOUNT_HASH_WIDTH",
            Self::StorageValue(_) => "ETH_STATE_STORAGE_VALUE_RLP",
            Self::ExplicitZeroStorageValue => "ETH_STATE_STORAGE_EXPLICIT_ZERO",
            Self::InconsistentProofTraversal => "ETH_STATE_PROOF_INCONSISTENT_TRAVERSAL",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Proof(error) => error.message(),
            Self::Account(_) => "proven account value is not canonical",
            Self::StorageValue(_) => "proven storage value is not a canonical RLP U256",
            Self::ExplicitZeroStorageValue => "canonical storage tries omit zero-valued slots",
            Self::InconsistentProofTraversal => "proof planning and verification outcomes differ",
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> StateProofVerificationErrorCategory {
        match self {
            Self::Proof(error) => match error.category() {
                crate::MptProofVerificationErrorCategory::Malformed => {
                    StateProofVerificationErrorCategory::Malformed
                }
                crate::MptProofVerificationErrorCategory::Absent => {
                    StateProofVerificationErrorCategory::Absent
                }
                crate::MptProofVerificationErrorCategory::WrongRoot => {
                    StateProofVerificationErrorCategory::WrongRoot
                }
            },
            Self::Account(_) | Self::StorageValue(_) | Self::ExplicitZeroStorageValue => {
                StateProofVerificationErrorCategory::NonCanonicalState
            }
            Self::InconsistentProofTraversal => StateProofVerificationErrorCategory::Internal,
        }
    }
}

impl From<MptProofVerificationError> for StateProofVerificationError {
    fn from(error: MptProofVerificationError) -> Self {
        Self::Proof(error)
    }
}

impl fmt::Display for StateProofVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for StateProofVerificationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Proof(error) => Some(error),
            Self::Account(error) => Some(error),
            Self::StorageValue(error) => Some(error),
            Self::ExplicitZeroStorageValue | Self::InconsistentProofTraversal => None,
        }
    }
}

/// Stable category for composed state-proof failures.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateProofVerificationErrorCategory {
    /// Proof bytes or shape were malformed or incomplete.
    Malformed,
    /// A valid proof established absence where inclusion was required.
    Absent,
    /// The proof does not match the trusted root or expected path.
    WrongRoot,
    /// Authenticated state bytes violate canonical Ethereum state encoding.
    NonCanonicalState,
    /// A verifier invariant failed after preflight.
    Internal,
}
