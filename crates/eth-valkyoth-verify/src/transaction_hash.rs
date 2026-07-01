use core::fmt;

use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::B256;
use eth_valkyoth_protocol::{
    TransactionEncodeError, UnvalidatedAccessListTransaction, UnvalidatedBlobTransaction,
    UnvalidatedDynamicFeeTransaction, UnvalidatedLegacyTransaction,
    encode_access_list_signing_preimage, encode_blob_signing_preimage,
    encode_dynamic_fee_signing_preimage, encode_legacy_eip155_signing_preimage,
};

/// Ethereum transaction signing hash domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionSigningHash(B256);

impl TransactionSigningHash {
    /// Creates a transaction signing hash from a raw Keccak-256 digest.
    #[must_use]
    pub const fn from_b256(value: B256) -> Self {
        Self(value)
    }

    /// Returns the raw digest.
    #[must_use]
    pub const fn to_b256(self) -> B256 {
        self.0
    }
}

impl From<TransactionSigningHash> for B256 {
    fn from(value: TransactionSigningHash) -> Self {
        value.to_b256()
    }
}

/// Transaction signing-hash construction failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionSigningHashError {
    /// A legacy transaction did not encode an EIP-155 replay domain.
    MissingReplayDomain,
    /// The canonical signing preimage could not be encoded into caller scratch.
    Encode(TransactionEncodeError),
}

impl TransactionSigningHashError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingReplayDomain => "ETH_TX_SIGNING_HASH_MISSING_REPLAY_DOMAIN",
            Self::Encode(error) => error.code(),
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::MissingReplayDomain => "legacy transaction has no EIP-155 replay domain",
            Self::Encode(_) => "transaction signing preimage encoding failed",
        }
    }
}

impl fmt::Display for TransactionSigningHashError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TransactionSigningHashError {}

impl From<TransactionEncodeError> for TransactionSigningHashError {
    fn from(error: TransactionEncodeError) -> Self {
        Self::Encode(error)
    }
}

/// Builds the EIP-155 legacy transaction signing hash.
///
/// The signing preimage is the canonical nine-field legacy list with
/// `[chain_id, 0, 0]` replacing the signature fields. The chain ID is recovered
/// from the decoded `v` field; pre-EIP-155 legacy transactions return
/// [`TransactionSigningHashError::MissingReplayDomain`].
pub fn legacy_eip155_transaction_signing_hash<H>(
    transaction: &UnvalidatedLegacyTransaction<'_>,
    scratch: &mut [u8],
    hasher: H,
) -> Result<TransactionSigningHash, TransactionSigningHashError>
where
    H: Keccak256,
{
    let chain_id = transaction
        .eip155_chain_id()
        .ok_or(TransactionSigningHashError::MissingReplayDomain)?;
    let written = encode_legacy_eip155_signing_preimage(transaction, chain_id, scratch)?;
    hash_written_preimage(scratch, written, hasher)
}

/// Builds the EIP-2930 access-list transaction signing hash.
pub fn access_list_transaction_signing_hash<H>(
    transaction: &UnvalidatedAccessListTransaction<'_>,
    scratch: &mut [u8],
    hasher: H,
) -> Result<TransactionSigningHash, TransactionSigningHashError>
where
    H: Keccak256,
{
    let written = encode_access_list_signing_preimage(transaction, scratch)?;
    hash_written_preimage(scratch, written, hasher)
}

/// Builds the EIP-1559 dynamic-fee transaction signing hash.
pub fn dynamic_fee_transaction_signing_hash<H>(
    transaction: &UnvalidatedDynamicFeeTransaction<'_>,
    scratch: &mut [u8],
    hasher: H,
) -> Result<TransactionSigningHash, TransactionSigningHashError>
where
    H: Keccak256,
{
    let written = encode_dynamic_fee_signing_preimage(transaction, scratch)?;
    hash_written_preimage(scratch, written, hasher)
}

/// Builds the EIP-4844 blob transaction signing hash.
pub fn blob_transaction_signing_hash<H>(
    transaction: &UnvalidatedBlobTransaction<'_>,
    scratch: &mut [u8],
    hasher: H,
) -> Result<TransactionSigningHash, TransactionSigningHashError>
where
    H: Keccak256,
{
    let written = encode_blob_signing_preimage(transaction, scratch)?;
    hash_written_preimage(scratch, written, hasher)
}

fn hash_written_preimage<H>(
    scratch: &[u8],
    written: usize,
    hasher: H,
) -> Result<TransactionSigningHash, TransactionSigningHashError>
where
    H: Keccak256,
{
    let preimage = scratch.get(..written).ok_or(TransactionEncodeError::Codec(
        eth_valkyoth_codec::DecodeError::OffsetOutOfBounds,
    ))?;
    Ok(TransactionSigningHash::from_b256(hash_one(
        hasher, preimage,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eth_valkyoth_codec::DecodeLimits;
    use eth_valkyoth_hash::Keccak256Digest;
    use eth_valkyoth_protocol::{
        decode_access_list_transaction, decode_dynamic_fee_transaction, decode_legacy_transaction,
    };

    const TEST_LIMITS: DecodeLimits = DecodeLimits {
        max_input_bytes: 128,
        max_list_items: 16,
        max_nesting_depth: 8,
        max_total_allocation: 128,
        max_proof_nodes: 4,
        max_total_items: 32,
    };

    #[derive(Default)]
    struct TranscriptHasher {
        output: [u8; 32],
    }

    impl Keccak256 for TranscriptHasher {
        fn update(&mut self, input: &[u8]) {
            if let Some(target) = self.output.first_mut() {
                *target = input.first().copied().unwrap_or(0);
            }
            if let Some(target) = self.output.get_mut(1) {
                *target = input.len().try_into().unwrap_or(u8::MAX);
            }
            if let Some(target) = self.output.last_mut() {
                *target = input.last().copied().unwrap_or(0);
            }
        }

        fn finalize(self) -> Keccak256Digest {
            B256::from_bytes(self.output)
        }
    }

    #[test]
    fn legacy_hash_requires_eip155_replay_domain() {
        let tx = [
            0xcb, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0x1b, 0x01, 0x02,
        ];
        let decoded = decode_legacy_transaction(&tx, TEST_LIMITS);
        assert!(decoded.is_ok());
        if let Ok(decoded) = decoded {
            let mut scratch = [0_u8; 32];
            assert_eq!(
                legacy_eip155_transaction_signing_hash(
                    &decoded,
                    &mut scratch,
                    TranscriptHasher::default(),
                ),
                Err(TransactionSigningHashError::MissingReplayDomain)
            );
        }
    }

    #[test]
    fn typed_transaction_hashes_use_unsigned_preimage() {
        let access_list = [
            0x01, 0xcd, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01, 0x01,
            0x02,
        ];
        let dynamic_fee = [
            0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01,
            0x01, 0x02,
        ];
        let access_list = decode_access_list_transaction(&access_list, TEST_LIMITS);
        let dynamic_fee = decode_dynamic_fee_transaction(&dynamic_fee, TEST_LIMITS);
        assert!(access_list.is_ok());
        assert!(dynamic_fee.is_ok());
        if let (Ok(access_list), Ok(dynamic_fee)) = (access_list, dynamic_fee) {
            let mut scratch = [0_u8; 32];
            let access_hash = access_list_transaction_signing_hash(
                &access_list,
                &mut scratch,
                TranscriptHasher::default(),
            );
            let dynamic_hash = dynamic_fee_transaction_signing_hash(
                &dynamic_fee,
                &mut scratch,
                TranscriptHasher::default(),
            );

            assert_eq!(
                access_hash.map(TransactionSigningHash::to_b256),
                Ok(B256::from_bytes([
                    1, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 192,
                ]))
            );
            assert_eq!(
                dynamic_hash.map(TransactionSigningHash::to_b256),
                Ok(B256::from_bytes([
                    2, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 192,
                ]))
            );
        }
    }
}
