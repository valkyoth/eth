#![no_std]
#![forbid(unsafe_code)]
//! Verification boundaries for Ethereum replay domains, signatures, and proofs.

#[cfg(feature = "std")]
extern crate std;

use core::fmt;
use eth_valkyoth_primitives::ChainId;

/// Verification failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerifyError {
    /// The input is bound to a different chain.
    WrongChain,
    /// The signature representation is not accepted.
    InvalidSignature,
    /// The proof is malformed or does not verify against its root.
    InvalidProof,
}

impl VerifyError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::WrongChain => "ETH_VERIFY_WRONG_CHAIN",
            Self::InvalidSignature => "ETH_VERIFY_INVALID_SIGNATURE",
            Self::InvalidProof => "ETH_VERIFY_INVALID_PROOF",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::WrongChain => "input is bound to a different chain",
            Self::InvalidSignature => "signature representation is not accepted",
            Self::InvalidProof => "proof is malformed or does not verify",
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> VerifyErrorCategory {
        match self {
            Self::WrongChain => VerifyErrorCategory::ReplayDomain,
            Self::InvalidSignature => VerifyErrorCategory::Signature,
            Self::InvalidProof => VerifyErrorCategory::Proof,
        }
    }
}

impl fmt::Display for VerifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VerifyError {}

/// Stable high-level verification error categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerifyErrorCategory {
    /// Replay-domain or chain binding failure.
    ReplayDomain,
    /// Signature representation or verification failure.
    Signature,
    /// Proof structure or root verification failure.
    Proof,
}

/// Checks that a replay domain matches the expected chain.
///
/// Chain IDs are public replay-domain metadata, so this uses ordinary integer
/// equality rather than constant-time comparison.
pub fn require_chain(expected: ChainId, actual: ChainId) -> Result<(), VerifyError> {
    if expected == actual {
        Ok(())
    } else {
        Err(VerifyError::WrongChain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::string::ToString;

    #[test]
    fn rejects_wrong_chain() {
        assert_eq!(
            require_chain(ChainId::new(1), ChainId::new(2)),
            Err(VerifyError::WrongChain)
        );
    }

    #[test]
    fn verify_errors_have_stable_codes_and_messages() {
        let error = VerifyError::InvalidProof;

        assert_eq!(error.code(), "ETH_VERIFY_INVALID_PROOF");
        assert_eq!(error.message(), "proof is malformed or does not verify");
        assert_eq!(error.category(), VerifyErrorCategory::Proof);
        assert_eq!(error.to_string(), "proof is malformed or does not verify");
    }
}
