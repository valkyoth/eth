use eth_valkyoth_hash::Keccak256;
use eth_valkyoth_primitives::Address;
use eth_valkyoth_protocol::{SignatureYParity, UnvalidatedLegacyTransaction};

use crate::{
    EthereumSignature, RecoverableSecp256k1, TransactionSigningHash, VerifyError,
    recover_sender_from_digest_with_backend,
    transaction_signature::{TransactionSignatureValidationError, ValidatedTransactionSignature},
};

pub(crate) fn recover_and_check<B, H>(
    signing_hash: TransactionSigningHash,
    signature: EthereumSignature,
    expected_sender: Option<Address>,
    secp256k1_backend: B,
    address_hasher: H,
) -> Result<ValidatedTransactionSignature, TransactionSignatureValidationError>
where
    B: RecoverableSecp256k1,
    H: Keccak256,
{
    let sender = recover_sender_from_digest_with_backend(
        signing_hash.to_b256(),
        signature,
        secp256k1_backend,
        address_hasher,
    )
    .map_err(|_| TransactionSignatureValidationError::InvalidSignature)?;
    if let Some(expected) = expected_sender
        && sender != expected
    {
        return Err(TransactionSignatureValidationError::WrongSender);
    }
    Ok(ValidatedTransactionSignature::new(sender, signing_hash))
}

pub(crate) fn legacy_signature(
    transaction: &UnvalidatedLegacyTransaction<'_>,
) -> Result<EthereumSignature, TransactionSignatureValidationError> {
    let chain_id =
        transaction
            .eip155_chain_id()
            .ok_or(TransactionSignatureValidationError::ReplayDomain(
                VerifyError::MissingReplayDomain,
            ))?;
    let v = legacy_v_u64(transaction.v).ok_or(
        TransactionSignatureValidationError::ReplayDomain(VerifyError::MissingReplayDomain),
    )?;
    let base = chain_id
        .get()
        .checked_mul(2)
        .and_then(|value| value.checked_add(35))
        .ok_or(TransactionSignatureValidationError::InvalidSignature)?;
    let y_parity = v
        .checked_sub(base)
        .and_then(|value| SignatureYParity::try_new(value).ok())
        .ok_or(TransactionSignatureValidationError::InvalidSignature)?;
    Ok(EthereumSignature::from_parts(
        transaction.r,
        transaction.s,
        y_parity,
    ))
}

fn legacy_v_u64(value: [u8; 32]) -> Option<u64> {
    const U64_TAIL_START: usize = 24;

    let high = value.get(..U64_TAIL_START)?;
    if high.iter().any(|byte| *byte != 0) {
        return None;
    }
    let tail = value.get(U64_TAIL_START..)?.try_into().ok()?;
    Some(u64::from_be_bytes(tail))
}
