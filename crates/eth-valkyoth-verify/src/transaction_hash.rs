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
///
/// This alone is not signature validation: replay-domain policy beyond chain-ID
/// presence, low-s/y-parity enforcement, sender recovery, and sender-state
/// promotion are deferred to a later validation API. Do not treat
/// `recover_sender_from_digest` composed with this hash as a validated
/// transaction sender.
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
///
/// This alone is not signature validation: low-s/y-parity enforcement, sender
/// recovery, replay-domain policy composition, and sender-state promotion are
/// deferred to a later validation API. Do not treat `recover_sender_from_digest`
/// composed with this hash as a validated transaction sender.
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
///
/// This alone is not signature validation: low-s/y-parity enforcement, sender
/// recovery, replay-domain policy composition, and sender-state promotion are
/// deferred to a later validation API. Do not treat `recover_sender_from_digest`
/// composed with this hash as a validated transaction sender.
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
///
/// This alone is not signature validation: low-s/y-parity enforcement, sender
/// recovery, replay-domain policy composition, blob validity, and sender-state
/// promotion are deferred to a later validation API. Do not treat
/// `recover_sender_from_digest` composed with this hash as a validated
/// transaction sender.
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
        decode_access_list_transaction, decode_blob_transaction, decode_dynamic_fee_transaction,
        decode_legacy_transaction,
    };
    use sha3::Digest;

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

    struct RealKeccak {
        inner: sha3::Keccak256,
    }

    impl RealKeccak {
        fn new() -> Self {
            Self {
                inner: sha3::Keccak256::new(),
            }
        }
    }

    impl Keccak256 for RealKeccak {
        fn update(&mut self, input: &[u8]) {
            Digest::update(&mut self.inner, input);
        }

        fn finalize(self) -> Keccak256Digest {
            B256::from_bytes(self.inner.finalize().into())
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

    #[test]
    fn signing_hashes_match_real_keccak_vectors() {
        let legacy = [
            0xec, 0x09, 0x85, 0x04, 0xa8, 0x17, 0xc8, 0x00, 0x82, 0x52, 0x08, 0x94, 0x35, 0x35,
            0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35,
            0x35, 0x35, 0x35, 0x35, 0x88, 0x0d, 0xe0, 0xb6, 0xb3, 0xa7, 0x64, 0x00, 0x00, 0x80,
            0x25, 0x01, 0x02,
        ];
        let access_list = [
            0x01, 0xcd, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01, 0x01,
            0x02,
        ];
        let dynamic_fee = [
            0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01,
            0x01, 0x02,
        ];
        let blob = [
            0x03, 0xf8, 0x45, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x94, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x05, 0x80, 0xc0, 0x06, 0xe1, 0xa0, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x02,
        ];

        let legacy = decode_legacy_transaction(&legacy, TEST_LIMITS);
        let access_list = decode_access_list_transaction(&access_list, TEST_LIMITS);
        let dynamic_fee = decode_dynamic_fee_transaction(&dynamic_fee, TEST_LIMITS);
        let blob = decode_blob_transaction(&blob, TEST_LIMITS);
        assert!(legacy.is_ok());
        assert!(access_list.is_ok());
        assert!(dynamic_fee.is_ok());
        assert!(blob.is_ok());

        if let (Ok(legacy), Ok(access_list), Ok(dynamic_fee), Ok(blob)) =
            (legacy, access_list, dynamic_fee, blob)
        {
            let mut scratch = [0_u8; 128];
            assert_eq!(
                legacy_eip155_transaction_signing_hash(&legacy, &mut scratch, RealKeccak::new())
                    .map(TransactionSigningHash::to_b256),
                Ok(B256::from_bytes([
                    0xda, 0xf5, 0xa7, 0x79, 0xae, 0x97, 0x2f, 0x97, 0x21, 0x97, 0x30, 0x3d, 0x7b,
                    0x57, 0x47, 0x46, 0xc7, 0xef, 0x83, 0xea, 0xda, 0xc0, 0xf2, 0x79, 0x1a, 0xd2,
                    0x3d, 0xb9, 0x2e, 0x4c, 0x8e, 0x53,
                ]))
            );
            assert_eq!(
                access_list_transaction_signing_hash(&access_list, &mut scratch, RealKeccak::new())
                    .map(TransactionSigningHash::to_b256),
                Ok(B256::from_bytes([
                    0xc1, 0x2d, 0xe1, 0x45, 0x2f, 0x6c, 0xf1, 0xbe, 0xd2, 0x67, 0xef, 0xfc, 0x37,
                    0x20, 0xc6, 0xf3, 0x94, 0x6a, 0x96, 0x75, 0x1d, 0x76, 0xd6, 0x79, 0x7e, 0x5b,
                    0x64, 0x19, 0x10, 0xdb, 0x93, 0x9f,
                ]))
            );
            assert_eq!(
                dynamic_fee_transaction_signing_hash(&dynamic_fee, &mut scratch, RealKeccak::new())
                    .map(TransactionSigningHash::to_b256),
                Ok(B256::from_bytes([
                    0x23, 0x3e, 0xd8, 0xc0, 0xf5, 0xda, 0x09, 0xf6, 0xd7, 0x63, 0x33, 0x37, 0x88,
                    0x57, 0x50, 0x5c, 0x87, 0xb1, 0x8f, 0xfb, 0x85, 0xa1, 0x0d, 0x5a, 0xd4, 0x44,
                    0xe7, 0x66, 0x93, 0x0b, 0x7d, 0xae,
                ]))
            );
            assert_eq!(
                blob_transaction_signing_hash(&blob, &mut scratch, RealKeccak::new())
                    .map(TransactionSigningHash::to_b256),
                Ok(B256::from_bytes([
                    0x1a, 0x8b, 0xce, 0x34, 0xe3, 0x2f, 0x6a, 0x3c, 0x95, 0x7c, 0xcb, 0x04, 0x02,
                    0xf6, 0x6f, 0x69, 0xc1, 0x18, 0x2b, 0x0a, 0xba, 0x34, 0xda, 0x1e, 0xfa, 0xef,
                    0x77, 0x21, 0xa7, 0x51, 0xc2, 0xd6,
                ]))
            );
        }
    }
}
