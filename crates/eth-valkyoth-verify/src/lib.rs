#![no_std]
#![forbid(unsafe_code)]
//! Verification boundaries for Ethereum replay domains, signatures, and proofs.

#[cfg(feature = "std")]
extern crate std;

use core::fmt;
use eth_valkyoth_primitives::ChainId;

mod eip712;
mod eip712_typed;
mod mpt;
mod replay;
mod sender;
mod set_code_authorization;
mod transaction_hash;
mod transaction_signature;

pub use eip712::{
    EIP712_SIGNING_PREFIX, Eip712Domain, Eip712DomainExpectation, eip712_signing_digest,
    recover_eip712_sender, require_eip712_domain,
};
pub use eip712_typed::{
    Eip712DomainData, Eip712EncodeError, Eip712Field, Eip712StructType, Eip712Value,
    Eip712ValueKind, eip712_domain_separator, eip712_hash_struct, eip712_type_hash,
    eip712_typed_data_signing_digest, encode_eip712_data, encode_eip712_type,
};
#[cfg(feature = "json")]
pub use eip712_typed::{Eip712JsonError, Eip712JsonLimits, eip712_json_typed_data_signing_digest};
pub use mpt::{
    MPT_BRANCH_CHILD_COUNT, MPT_BRANCH_NODE_FIELD_COUNT, MPT_COMPACT_NODE_FIELD_COUNT,
    MPT_HASH_REFERENCE_BYTES, MPT_INLINE_REFERENCE_DEPTH_LIMIT, MptBranchChildren, MptBranchNode,
    MptCompactPath, MptCompactPathKind, MptExtensionNode, MptInlineNode, MptLeafNode, MptNode,
    MptNodeDecodeError, MptNodeDecodeErrorCategory, MptNodeField, MptNodeReference, MptProofNodes,
    decode_mpt_node, decode_mpt_proof_nodes,
};
pub use replay::{
    require_access_list_replay_domain, require_blob_replay_domain,
    require_dynamic_fee_replay_domain, require_legacy_replay_domain,
    require_set_code_replay_domain, require_transaction_replay_domain,
};
pub use sender::{
    COMPACT_SIGNATURE_BYTES, ETHEREUM_PUBLIC_KEY_BYTES, ETHEREUM_SIGNATURE_BYTES,
    EthereumSignature, SIGNING_DIGEST_BYTES, recover_sender_from_digest,
};
pub use set_code_authorization::{
    SetCodeAuthorizationValidationError, SetCodeAuthorizationValidationErrorCategory,
    ValidatedSetCodeAuthorization, validate_set_code_authorization_signature,
};
pub use transaction_hash::{
    SetCodeAuthorizationSigningHash, TransactionSigningHash, TransactionSigningHashError,
    access_list_transaction_signing_hash, blob_transaction_signing_hash,
    dynamic_fee_transaction_signing_hash, legacy_eip155_transaction_signing_hash,
    set_code_authorization_signing_hash, set_code_transaction_signing_hash,
};
pub use transaction_signature::{
    TransactionSignatureValidationError, TransactionSignatureValidationErrorCategory,
    ValidatedTransactionSignature, validate_access_list_transaction_signature,
    validate_blob_transaction_signature, validate_dynamic_fee_transaction_signature,
    validate_legacy_transaction_signature, validate_set_code_transaction_signature,
    validate_transaction_signature,
};

/// Verification failure categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerifyError {
    /// The input has no chain-bound replay domain.
    MissingReplayDomain,
    /// The input is bound to a different chain.
    WrongChain,
    /// The EIP-712 domain has no `chainId` field.
    MissingEip712ChainId,
    /// The EIP-712 domain has no `verifyingContract` field.
    MissingEip712VerifyingContract,
    /// The EIP-712 domain is bound to a different verifying contract.
    WrongVerifyingContract,
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
            Self::MissingReplayDomain => "ETH_VERIFY_MISSING_REPLAY_DOMAIN",
            Self::WrongChain => "ETH_VERIFY_WRONG_CHAIN",
            Self::MissingEip712ChainId => "ETH_VERIFY_MISSING_EIP712_CHAIN_ID",
            Self::MissingEip712VerifyingContract => "ETH_VERIFY_MISSING_EIP712_VERIFYING_CONTRACT",
            Self::WrongVerifyingContract => "ETH_VERIFY_WRONG_VERIFYING_CONTRACT",
            Self::InvalidSignature => "ETH_VERIFY_INVALID_SIGNATURE",
            Self::InvalidProof => "ETH_VERIFY_INVALID_PROOF",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::MissingReplayDomain => "input has no chain-bound replay domain",
            Self::WrongChain => "input is bound to a different chain",
            Self::MissingEip712ChainId => "EIP-712 domain is missing chainId",
            Self::MissingEip712VerifyingContract => "EIP-712 domain is missing verifyingContract",
            Self::WrongVerifyingContract => {
                "EIP-712 domain is bound to a different verifying contract"
            }
            Self::InvalidSignature => "signature representation is not accepted",
            Self::InvalidProof => "proof is malformed or does not verify",
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> VerifyErrorCategory {
        match self {
            Self::MissingReplayDomain => VerifyErrorCategory::ReplayDomain,
            Self::WrongChain => VerifyErrorCategory::ReplayDomain,
            Self::MissingEip712ChainId
            | Self::MissingEip712VerifyingContract
            | Self::WrongVerifyingContract => VerifyErrorCategory::StructuredDataDomain,
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
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerifyErrorCategory {
    /// Replay-domain or chain binding failure.
    ReplayDomain,
    /// EIP-712 structured-data domain failure.
    StructuredDataDomain,
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

        let error = VerifyError::MissingReplayDomain;

        assert_eq!(error.code(), "ETH_VERIFY_MISSING_REPLAY_DOMAIN");
        assert_eq!(error.message(), "input has no chain-bound replay domain");
        assert_eq!(error.category(), VerifyErrorCategory::ReplayDomain);

        let error = VerifyError::WrongVerifyingContract;

        assert_eq!(error.code(), "ETH_VERIFY_WRONG_VERIFYING_CONTRACT");
        assert_eq!(
            error.message(),
            "EIP-712 domain is bound to a different verifying contract"
        );
        assert_eq!(error.category(), VerifyErrorCategory::StructuredDataDomain);
    }
}
