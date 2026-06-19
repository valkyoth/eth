#![no_std]
#![forbid(unsafe_code)]
//! Verification boundaries for Ethereum replay domains, signatures, and proofs.

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

/// Checks that a replay domain matches the expected chain.
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

    #[test]
    fn rejects_wrong_chain() {
        assert_eq!(
            require_chain(ChainId::new(1), ChainId::new(2)),
            Err(VerifyError::WrongChain)
        );
    }
}
