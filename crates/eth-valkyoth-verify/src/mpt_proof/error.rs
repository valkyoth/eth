use core::fmt;

use eth_valkyoth_codec::DecodeError;

use crate::mpt::MptNodeDecodeError;

/// MPT inclusion proof verification failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptProofVerificationError {
    /// The transaction or receipt index key could not be RLP-encoded.
    KeyEncode(DecodeError),
    /// A proof node failed bounded MPT decoding.
    MalformedNode(MptNodeDecodeError),
    /// The proof did not contain the root node or a required hashed child.
    MissingProofNode,
    /// The first proof node does not match the trusted root or a hashed child
    /// does not match its parent reference.
    WrongRoot,
    /// The proof is valid for a different key or proves absence at this key.
    Absent,
    /// The proof reaches the requested key but with a different value.
    ValueMismatch,
    /// The proof had unused nodes after the matching value was reached.
    TrailingProofNodes,
    /// The proof traversal exceeded the verifier's fixed stack-safety cap.
    ProofTooDeep,
}

impl MptProofVerificationError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::KeyEncode(error) => error.code(),
            Self::MalformedNode(error) => error.code(),
            Self::MissingProofNode => "ETH_MPT_PROOF_MISSING_NODE",
            Self::WrongRoot => "ETH_MPT_PROOF_WRONG_ROOT",
            Self::Absent => "ETH_MPT_PROOF_ABSENT",
            Self::ValueMismatch => "ETH_MPT_PROOF_VALUE_MISMATCH",
            Self::TrailingProofNodes => "ETH_MPT_PROOF_TRAILING_NODES",
            Self::ProofTooDeep => "ETH_MPT_PROOF_TOO_DEEP",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::KeyEncode(_) => "MPT proof key encoding failed",
            Self::MalformedNode(_) => "MPT proof node is malformed",
            Self::MissingProofNode => "MPT proof is missing a required node",
            Self::WrongRoot => "MPT proof node hash does not match the trusted root",
            Self::Absent => "MPT proof does not contain the requested key",
            Self::ValueMismatch => "MPT proof value does not match the expected value",
            Self::TrailingProofNodes => "MPT proof contains unused trailing nodes",
            Self::ProofTooDeep => "MPT proof traversal depth limit exceeded",
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> MptProofVerificationErrorCategory {
        match self {
            Self::KeyEncode(_)
            | Self::MalformedNode(_)
            | Self::MissingProofNode
            | Self::ProofTooDeep => MptProofVerificationErrorCategory::Malformed,
            Self::WrongRoot | Self::ValueMismatch | Self::TrailingProofNodes => {
                MptProofVerificationErrorCategory::WrongRoot
            }
            Self::Absent => MptProofVerificationErrorCategory::Absent,
        }
    }
}

impl fmt::Display for MptProofVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MptProofVerificationError {}

/// Stable high-level MPT proof verification categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptProofVerificationErrorCategory {
    /// The proof bytes are malformed or incomplete.
    Malformed,
    /// The proof is well-formed but proves absence at the requested key.
    Absent,
    /// The proof does not match the trusted root or expected value.
    WrongRoot,
}
