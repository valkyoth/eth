use core::fmt;

use eth_valkyoth_hash::Keccak256;
use eth_valkyoth_primitives::{Address, ChainId};
use eth_valkyoth_protocol::{
    UnvalidatedAccessListTransaction, UnvalidatedBlobTransaction, UnvalidatedDynamicFeeTransaction,
    UnvalidatedLegacyTransaction, UnvalidatedTransaction,
};

#[cfg(feature = "secp256k1-k256")]
use crate::K256Secp256k1Backend;
pub(crate) use crate::transaction_signature_set_code::validate_set_code_transaction_signature_with_backend;
use crate::{
    EthereumSignature, RecoverableSecp256k1, TransactionSigningHash, TransactionSigningHashError,
    VerifyError, access_list_transaction_signing_hash, blob_transaction_signing_hash,
    dynamic_fee_transaction_signing_hash, legacy_eip155_transaction_signing_hash,
    require_access_list_replay_domain, require_blob_replay_domain,
    require_dynamic_fee_replay_domain, require_legacy_replay_domain,
    transaction_signature_helpers::{legacy_signature, recover_and_check},
};

#[cfg(test)]
#[path = "transaction_signature_external_tests.rs"]
mod external_tests;
#[cfg(test)]
#[path = "transaction_signature_tests.rs"]
mod tests;

/// A decoded transaction signature that passed replay-domain policy,
/// signing-hash construction, low-s/y-parity policy, and sender recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedTransactionSignature {
    sender: Address,
    signing_hash: TransactionSigningHash,
}

impl ValidatedTransactionSignature {
    /// Creates a validated signature result from its checked components.
    #[must_use]
    pub(crate) const fn new(sender: Address, signing_hash: TransactionSigningHash) -> Self {
        Self {
            sender,
            signing_hash,
        }
    }

    /// Returns the recovered transaction sender.
    #[must_use]
    pub const fn sender(self) -> Address {
        self.sender
    }

    /// Returns the transaction signing hash that was recovered against.
    #[must_use]
    pub const fn signing_hash(self) -> TransactionSigningHash {
        self.signing_hash
    }
}

/// Decoded transaction signature validation failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionSignatureValidationError {
    /// Replay-domain policy rejected the transaction.
    ReplayDomain(VerifyError),
    /// Signing-hash construction failed before recovery.
    SigningHash(TransactionSigningHashError),
    /// Signature scalar, y-parity, or public-key recovery failed.
    InvalidSignature,
    /// This transaction family has no full signature-validation helper yet.
    UnsupportedTransactionType,
    /// The recovered sender does not match the expected sender.
    WrongSender,
}

impl TransactionSignatureValidationError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ReplayDomain(error) => error.code(),
            Self::SigningHash(error) => error.code(),
            Self::InvalidSignature => "ETH_TX_SIGNATURE_INVALID",
            Self::UnsupportedTransactionType => "ETH_TX_SIGNATURE_UNSUPPORTED_TYPE",
            Self::WrongSender => "ETH_TX_SIGNATURE_WRONG_SENDER",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::ReplayDomain(error) => error.message(),
            Self::SigningHash(_) => "transaction signing hash construction failed",
            Self::InvalidSignature => "transaction signature is not accepted",
            Self::UnsupportedTransactionType => {
                "transaction type has no signature validation helper yet"
            }
            Self::WrongSender => "transaction signature recovered a different sender",
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> TransactionSignatureValidationErrorCategory {
        match self {
            Self::ReplayDomain(_) => TransactionSignatureValidationErrorCategory::ReplayDomain,
            Self::SigningHash(_) => TransactionSignatureValidationErrorCategory::SigningHash,
            Self::InvalidSignature | Self::WrongSender | Self::UnsupportedTransactionType => {
                TransactionSignatureValidationErrorCategory::Signature
            }
        }
    }
}

impl fmt::Display for TransactionSignatureValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TransactionSignatureValidationError {}

/// Stable high-level decoded transaction signature validation categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionSignatureValidationErrorCategory {
    /// Replay-domain or chain binding failure.
    ReplayDomain,
    /// Signing preimage or hash construction failure.
    SigningHash,
    /// Signature representation, recovery, or sender-match failure.
    Signature,
}

/// Validates any supported decoded transaction signature.
///
/// This combines replay-domain checking, canonical signing-hash construction,
/// low-s/y-parity policy, and sender recovery. It intentionally does not prove
/// fork validity, fee validity, account-state validity, blob/KZG validity, or
/// protocol typestate promotion.
///
/// # EIP-7702 set-code transactions
///
/// For [`UnvalidatedTransaction::SetCode`], `Ok` proves only that the outer
/// transaction sender signature in the `0x04` domain is valid. It does not
/// validate any authorization-tuple signature inside `authorization_list`.
/// Callers must additionally call
/// [`crate::validate_set_code_authorization_signature`] for every tuple before
/// treating the authorization list as authenticated. Applying a delegation from
/// an unchecked authorization tuple is equivalent to skipping signature
/// verification for that authorizing account.
#[cfg(feature = "secp256k1-k256")]
pub fn validate_transaction_signature<H1, H2>(
    expected_chain: ChainId,
    transaction: UnvalidatedTransaction<'_>,
    expected_sender: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    address_hasher: H2,
) -> Result<ValidatedTransactionSignature, TransactionSignatureValidationError>
where
    H1: Keccak256,
    H2: Keccak256,
{
    validate_transaction_signature_with_backend(
        expected_chain,
        transaction,
        expected_sender,
        scratch,
        signing_hasher,
        K256Secp256k1Backend,
        address_hasher,
    )
}

/// Validates any supported decoded transaction signature through a caller-provided backend.
///
/// This is the default dependency-free sender-recovery entry point. Use the
/// `secp256k1-k256` feature only when the reviewed `k256` adapter is the
/// desired concrete backend.
pub fn validate_transaction_signature_with_backend<B, H1, H2>(
    expected_chain: ChainId,
    transaction: UnvalidatedTransaction<'_>,
    expected_sender: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    mut secp256k1_backend: B,
    address_hasher: H2,
) -> Result<ValidatedTransactionSignature, TransactionSignatureValidationError>
where
    B: RecoverableSecp256k1,
    H1: Keccak256,
    H2: Keccak256,
{
    match transaction {
        UnvalidatedTransaction::Legacy(tx) => validate_legacy_transaction_signature_with_backend(
            expected_chain,
            &tx,
            expected_sender,
            scratch,
            signing_hasher,
            &mut secp256k1_backend,
            address_hasher,
        ),
        UnvalidatedTransaction::AccessList(tx) => {
            validate_access_list_transaction_signature_with_backend(
                expected_chain,
                &tx,
                expected_sender,
                scratch,
                signing_hasher,
                &mut secp256k1_backend,
                address_hasher,
            )
        }
        UnvalidatedTransaction::DynamicFee(tx) => {
            validate_dynamic_fee_transaction_signature_with_backend(
                expected_chain,
                &tx,
                expected_sender,
                scratch,
                signing_hasher,
                &mut secp256k1_backend,
                address_hasher,
            )
        }
        UnvalidatedTransaction::Blob(tx) => validate_blob_transaction_signature_with_backend(
            expected_chain,
            &tx,
            expected_sender,
            scratch,
            signing_hasher,
            &mut secp256k1_backend,
            address_hasher,
        ),
        UnvalidatedTransaction::SetCode(tx) => {
            validate_set_code_transaction_signature_with_backend(
                expected_chain,
                &tx,
                expected_sender,
                scratch,
                signing_hasher,
                &mut secp256k1_backend,
                address_hasher,
            )
        }
    }
}

/// Validates a decoded legacy EIP-155 transaction signature.
#[cfg(feature = "secp256k1-k256")]
pub fn validate_legacy_transaction_signature<H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedLegacyTransaction<'_>,
    expected_sender: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    address_hasher: H2,
) -> Result<ValidatedTransactionSignature, TransactionSignatureValidationError>
where
    H1: Keccak256,
    H2: Keccak256,
{
    validate_legacy_transaction_signature_with_backend(
        expected_chain,
        transaction,
        expected_sender,
        scratch,
        signing_hasher,
        K256Secp256k1Backend,
        address_hasher,
    )
}

/// Validates a decoded legacy EIP-155 transaction signature through a backend.
pub fn validate_legacy_transaction_signature_with_backend<B, H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedLegacyTransaction<'_>,
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
    require_legacy_replay_domain(expected_chain, transaction)
        .map_err(TransactionSignatureValidationError::ReplayDomain)?;
    let signing_hash = legacy_eip155_transaction_signing_hash(transaction, scratch, signing_hasher)
        .map_err(TransactionSignatureValidationError::SigningHash)?;
    let signature = legacy_signature(transaction)?;
    recover_and_check(
        signing_hash,
        signature,
        expected_sender,
        secp256k1_backend,
        address_hasher,
    )
}

/// Validates a decoded EIP-2930 transaction signature.
#[cfg(feature = "secp256k1-k256")]
pub fn validate_access_list_transaction_signature<H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedAccessListTransaction<'_>,
    expected_sender: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    address_hasher: H2,
) -> Result<ValidatedTransactionSignature, TransactionSignatureValidationError>
where
    H1: Keccak256,
    H2: Keccak256,
{
    validate_access_list_transaction_signature_with_backend(
        expected_chain,
        transaction,
        expected_sender,
        scratch,
        signing_hasher,
        K256Secp256k1Backend,
        address_hasher,
    )
}

/// Validates a decoded EIP-2930 transaction signature through a backend.
pub fn validate_access_list_transaction_signature_with_backend<B, H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedAccessListTransaction<'_>,
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
    require_access_list_replay_domain(expected_chain, transaction)
        .map_err(TransactionSignatureValidationError::ReplayDomain)?;
    let signing_hash = access_list_transaction_signing_hash(transaction, scratch, signing_hasher)
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

/// Validates a decoded EIP-1559 transaction signature.
#[cfg(feature = "secp256k1-k256")]
pub fn validate_dynamic_fee_transaction_signature<H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedDynamicFeeTransaction<'_>,
    expected_sender: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    address_hasher: H2,
) -> Result<ValidatedTransactionSignature, TransactionSignatureValidationError>
where
    H1: Keccak256,
    H2: Keccak256,
{
    validate_dynamic_fee_transaction_signature_with_backend(
        expected_chain,
        transaction,
        expected_sender,
        scratch,
        signing_hasher,
        K256Secp256k1Backend,
        address_hasher,
    )
}

/// Validates a decoded EIP-1559 transaction signature through a backend.
pub fn validate_dynamic_fee_transaction_signature_with_backend<B, H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedDynamicFeeTransaction<'_>,
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
    require_dynamic_fee_replay_domain(expected_chain, transaction)
        .map_err(TransactionSignatureValidationError::ReplayDomain)?;
    let signing_hash = dynamic_fee_transaction_signing_hash(transaction, scratch, signing_hasher)
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

/// Validates a decoded EIP-4844 transaction signature.
#[cfg(feature = "secp256k1-k256")]
pub fn validate_blob_transaction_signature<H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedBlobTransaction<'_>,
    expected_sender: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    address_hasher: H2,
) -> Result<ValidatedTransactionSignature, TransactionSignatureValidationError>
where
    H1: Keccak256,
    H2: Keccak256,
{
    validate_blob_transaction_signature_with_backend(
        expected_chain,
        transaction,
        expected_sender,
        scratch,
        signing_hasher,
        K256Secp256k1Backend,
        address_hasher,
    )
}

/// Validates a decoded EIP-4844 transaction signature through a backend.
pub fn validate_blob_transaction_signature_with_backend<B, H1, H2>(
    expected_chain: ChainId,
    transaction: &UnvalidatedBlobTransaction<'_>,
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
    require_blob_replay_domain(expected_chain, transaction)
        .map_err(TransactionSignatureValidationError::ReplayDomain)?;
    let signing_hash = blob_transaction_signing_hash(transaction, scratch, signing_hasher)
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
