use eth_valkyoth_hash::Keccak256;
use eth_valkyoth_primitives::{Address, ChainId};
use eth_valkyoth_protocol::UnvalidatedSetCodeTransaction;

#[cfg(feature = "secp256k1-k256")]
use crate::K256Secp256k1Backend;
use crate::{
    EthereumSignature, RecoverableSecp256k1, require_set_code_replay_domain,
    set_code_transaction_signing_hash,
    transaction_signature::{TransactionSignatureValidationError, ValidatedTransactionSignature},
    transaction_signature_helpers::recover_and_check,
};

/// Validates a decoded EIP-7702 set-code transaction sender signature.
///
/// This validates only the outer transaction signature. Authorization-list
/// tuple signatures use [`crate::validate_set_code_authorization_signature`]
/// because EIP-7702 defines a distinct authorization signing domain.
#[cfg(feature = "secp256k1-k256")]
pub fn validate_set_code_transaction_signature<H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedSetCodeTransaction<'_>,
    expected_sender: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    address_hasher: H2,
) -> Result<ValidatedTransactionSignature, TransactionSignatureValidationError>
where
    H1: Keccak256,
    H2: Keccak256,
{
    validate_set_code_transaction_signature_with_backend(
        expected_chain,
        transaction,
        expected_sender,
        scratch,
        signing_hasher,
        K256Secp256k1Backend,
        address_hasher,
    )
}

/// Validates a decoded EIP-7702 set-code transaction sender signature through a backend.
pub fn validate_set_code_transaction_signature_with_backend<B, H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedSetCodeTransaction<'_>,
    expected_sender: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    secp256k1_backend: B,
    address_hasher: H2,
) -> Result<ValidatedTransactionSignature, TransactionSignatureValidationError>
where
    B: RecoverableSecp256k1,
    H1: Keccak256,
    H2: Keccak256,
{
    require_set_code_replay_domain(expected_chain, transaction)
        .map_err(TransactionSignatureValidationError::ReplayDomain)?;
    let signing_hash = set_code_transaction_signing_hash(transaction, scratch, signing_hasher)
        .map_err(TransactionSignatureValidationError::SigningHash)?;
    let signature =
        EthereumSignature::from_parts(transaction.r, transaction.s, transaction.y_parity);
    recover_and_check(
        signing_hash,
        signature,
        expected_sender,
        secp256k1_backend,
        address_hasher,
    )
}
