use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::{Address, B256};
use eth_valkyoth_protocol::SignatureYParity;
#[cfg(feature = "secp256k1-k256")]
use k256::ecdsa::{
    RecoveryId as Secp256k1RecoveryId, Signature as Secp256k1Signature,
    VerifyingKey as Secp256k1VerifyingKey,
};

use crate::VerifyError;

/// Ethereum compact ECDSA signature length without recovery metadata.
pub const COMPACT_SIGNATURE_BYTES: usize = 64;
/// Ethereum compact ECDSA signature length with the y-parity byte appended.
pub const ETHEREUM_SIGNATURE_BYTES: usize = 65;
/// Ethereum signing digest length.
pub const SIGNING_DIGEST_BYTES: usize = 32;
/// Uncompressed secp256k1 public key payload length without the SEC1 prefix.
pub const ETHEREUM_PUBLIC_KEY_BYTES: usize = 64;

/// Recoverable secp256k1 backend boundary for Ethereum sender recovery.
///
/// Implementations must recover the uncompressed public key payload for the
/// exact `(digest, signature)` pair, enforce Ethereum's `r`/`s` scalar policy,
/// enforce the EIP-2 low-s rule, and accept only y-parity recovery IDs `0` and
/// `1`. Backends that hold private implementation state should document and
/// test their state-clearing behavior for key-adjacent hashing paths.
pub trait RecoverableSecp256k1 {
    /// Recovers the uncompressed public key payload without the SEC1 `0x04`
    /// prefix.
    fn recover_uncompressed_public_key(
        &mut self,
        signing_digest: B256,
        signature: EthereumSignature,
    ) -> Result<[u8; ETHEREUM_PUBLIC_KEY_BYTES], VerifyError>;
}

impl<T> RecoverableSecp256k1 for &mut T
where
    T: RecoverableSecp256k1 + ?Sized,
{
    fn recover_uncompressed_public_key(
        &mut self,
        signing_digest: B256,
        signature: EthereumSignature,
    ) -> Result<[u8; ETHEREUM_PUBLIC_KEY_BYTES], VerifyError> {
        (**self).recover_uncompressed_public_key(signing_digest, signature)
    }
}

/// Reviewed `k256` recoverable secp256k1 adapter.
///
/// This adapter is available only with the explicit `secp256k1-k256` feature.
/// The default crate graph exposes only [`RecoverableSecp256k1`].
#[cfg(feature = "secp256k1-k256")]
#[derive(Clone, Copy, Debug, Default)]
pub struct K256Secp256k1Backend;

/// Ethereum secp256k1 ECDSA signature with explicit y parity.
///
/// This is a representation-level type. It checks the recovery-id policy
/// (`0` or `1`) when constructed, and the recovery function checks scalar
/// validity plus the EIP-2 low-s policy before recovering a sender address.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EthereumSignature {
    r: [u8; 32],
    s: [u8; 32],
    y_parity: SignatureYParity,
}

impl EthereumSignature {
    /// Builds a signature from 32-byte `r`, 32-byte `s`, and typed y parity.
    #[must_use]
    pub const fn from_parts(r: [u8; 32], s: [u8; 32], y_parity: SignatureYParity) -> Self {
        Self { r, s, y_parity }
    }

    /// Builds a signature from 32-byte `r`, 32-byte `s`, and a raw y-parity bit.
    ///
    /// Only Ethereum recovery IDs `0` and `1` are accepted. Recovery IDs `2`
    /// and `3` are not accepted because this API is for Ethereum signatures,
    /// which encode only y parity.
    pub fn try_from_parts_with_y_parity(
        r: [u8; 32],
        s: [u8; 32],
        y_parity: u8,
    ) -> Result<Self, VerifyError> {
        let y_parity = SignatureYParity::try_new(u64::from(y_parity))
            .map_err(|_| VerifyError::InvalidSignature)?;
        Ok(Self::from_parts(r, s, y_parity))
    }

    /// Builds a signature from `r || s || y_parity`.
    pub fn try_from_bytes(bytes: [u8; ETHEREUM_SIGNATURE_BYTES]) -> Result<Self, VerifyError> {
        let (compact, y_parity) = bytes.split_at(COMPACT_SIGNATURE_BYTES);
        let (r, s) = compact.split_at(SIGNING_DIGEST_BYTES);
        let r = <[u8; 32]>::try_from(r).map_err(|_| VerifyError::InvalidSignature)?;
        let s = <[u8; 32]>::try_from(s).map_err(|_| VerifyError::InvalidSignature)?;
        let y_parity = y_parity
            .first()
            .copied()
            .ok_or(VerifyError::InvalidSignature)?;
        Self::try_from_parts_with_y_parity(r, s, y_parity)
    }

    /// Returns the raw 32-byte `r` scalar bytes.
    #[must_use]
    pub const fn r(self) -> [u8; 32] {
        self.r
    }

    /// Returns the raw 32-byte `s` scalar bytes.
    #[must_use]
    pub const fn s(self) -> [u8; 32] {
        self.s
    }

    /// Returns the Ethereum y-parity bit.
    #[must_use]
    pub const fn y_parity(self) -> SignatureYParity {
        self.y_parity
    }

    #[cfg(feature = "secp256k1-k256")]
    fn secp256k1_signature(self) -> Result<Secp256k1Signature, VerifyError> {
        let signature = Secp256k1Signature::from_scalars(self.r, self.s)
            .map_err(|_| VerifyError::InvalidSignature)?;
        if signature.normalize_s().is_some() {
            return Err(VerifyError::InvalidSignature);
        }
        Ok(signature)
    }

    #[cfg(feature = "secp256k1-k256")]
    fn secp256k1_recovery_id(self) -> Result<Secp256k1RecoveryId, VerifyError> {
        Secp256k1RecoveryId::try_from(self.y_parity.get())
            .map_err(|_| VerifyError::InvalidSignature)
    }
}

#[cfg(feature = "secp256k1-k256")]
impl RecoverableSecp256k1 for K256Secp256k1Backend {
    fn recover_uncompressed_public_key(
        &mut self,
        signing_digest: B256,
        signature: EthereumSignature,
    ) -> Result<[u8; ETHEREUM_PUBLIC_KEY_BYTES], VerifyError> {
        let secp256k1_signature = signature.secp256k1_signature()?;
        let recovery_id = signature.secp256k1_recovery_id()?;
        let digest_bytes = <[u8; SIGNING_DIGEST_BYTES]>::from(signing_digest);
        let verifying_key = Secp256k1VerifyingKey::recover_from_prehash(
            &digest_bytes,
            &secp256k1_signature,
            recovery_id,
        )
        .map_err(|_| VerifyError::InvalidSignature)?;
        public_key_payload_from_verifying_key(verifying_key)
    }
}

/// Recovers an Ethereum sender address through a caller-provided secp256k1 backend.
///
/// `signing_digest` must already be the Ethereum transaction signing hash for
/// the relevant transaction type. This function does not build the transaction
/// preimage, apply replay-domain policy, or validate fork/fee/account rules.
///
/// `hasher` must compute Ethereum Keccak-256. It is used only to hash the
/// recovered uncompressed public key payload into the sender address. Callers
/// should provide a hasher with an explicit state-clearing policy for this
/// path; the optional sanitization bridge is the preferred place to enforce
/// `SecureSanitize` on concrete stateful hashers.
pub fn recover_sender_from_digest_with_backend<B, H>(
    signing_digest: B256,
    signature: EthereumSignature,
    mut backend: B,
    hasher: H,
) -> Result<Address, VerifyError>
where
    B: RecoverableSecp256k1,
    H: Keccak256,
{
    let public_key = backend.recover_uncompressed_public_key(signing_digest, signature)?;
    address_from_uncompressed_public_key(public_key, hasher)
}

/// Recovers an Ethereum sender address with the reviewed `k256` adapter.
///
/// This convenience function is available only with the explicit
/// `secp256k1-k256` feature. Default builds should call
/// [`recover_sender_from_digest_with_backend`] with a deployment-selected
/// hardware, platform, WASM, or audited software backend.
#[cfg(feature = "secp256k1-k256")]
pub fn recover_sender_from_digest<H>(
    signing_digest: B256,
    signature: EthereumSignature,
    hasher: H,
) -> Result<Address, VerifyError>
where
    H: Keccak256,
{
    recover_sender_from_digest_with_backend(signing_digest, signature, K256Secp256k1Backend, hasher)
}

#[cfg(feature = "secp256k1-k256")]
fn public_key_payload_from_verifying_key(
    verifying_key: Secp256k1VerifyingKey,
) -> Result<[u8; ETHEREUM_PUBLIC_KEY_BYTES], VerifyError> {
    let encoded = verifying_key.to_encoded_point(false);
    let public_key = encoded
        .as_bytes()
        .get(1..)
        .ok_or(VerifyError::InvalidSignature)?;
    if public_key.len() != ETHEREUM_PUBLIC_KEY_BYTES {
        return Err(VerifyError::InvalidSignature);
    }
    <[u8; ETHEREUM_PUBLIC_KEY_BYTES]>::try_from(public_key)
        .map_err(|_| VerifyError::InvalidSignature)
}

/// Converts an uncompressed secp256k1 public key payload into an Ethereum address.
///
/// `public_key` must be the 64-byte `x || y` payload without the SEC1 `0x04`
/// prefix. The address is the low 20 bytes of `keccak256(public_key)`.
pub fn address_from_uncompressed_public_key<H>(
    public_key: [u8; ETHEREUM_PUBLIC_KEY_BYTES],
    hasher: H,
) -> Result<Address, VerifyError>
where
    H: Keccak256,
{
    let digest = hash_one(hasher, &public_key);
    let digest_bytes = <[u8; SIGNING_DIGEST_BYTES]>::from(digest);
    let address = digest_bytes
        .get(12..)
        .and_then(|bytes| <[u8; 20]>::try_from(bytes).ok())
        .ok_or(VerifyError::InvalidSignature)?;
    Ok(Address::from_bytes(address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_crypto::{RealKeccak, TestSecp256k1Backend};
    use k256::ecdsa::SigningKey;

    struct AddressFromPublicKeyHasher {
        digest: [u8; 32],
    }

    impl AddressFromPublicKeyHasher {
        const fn new() -> Self {
            Self { digest: [0_u8; 32] }
        }
    }

    impl Keccak256 for AddressFromPublicKeyHasher {
        fn update(&mut self, input: &[u8]) {
            let Some(source) = input.get(input.len().saturating_sub(20)..) else {
                return;
            };
            let Some(target) = self.digest.get_mut(12..) else {
                return;
            };
            target.copy_from_slice(source);
        }

        fn finalize(self) -> B256 {
            B256::from_bytes(self.digest)
        }
    }

    fn signing_digest() -> B256 {
        B256::from_bytes([
            0x44, 0x17, 0x9d, 0x9b, 0x5e, 0x7a, 0x31, 0x65, 0xd6, 0x78, 0x1a, 0x10, 0x82, 0x4f,
            0x9f, 0x35, 0x4d, 0xa4, 0x50, 0x21, 0xf8, 0x70, 0x21, 0x2f, 0x52, 0xee, 0x01, 0x22,
            0xae, 0x71, 0x42, 0x60,
        ])
    }

    fn signing_key() -> Result<SigningKey, VerifyError> {
        SigningKey::from_bytes(
            (&[
                0x4c, 0x08, 0x83, 0xa6, 0x91, 0x02, 0x93, 0x7d, 0x62, 0x31, 0x47, 0x1b, 0x5d, 0xbb,
                0x62, 0x04, 0xfe, 0x51, 0x29, 0x61, 0x70, 0x82, 0x79, 0x2a, 0xe4, 0x68, 0xd0, 0x1a,
                0x3f, 0x36, 0x23, 0x18,
            ])
                .into(),
        )
        .map_err(|_| VerifyError::InvalidSignature)
    }

    fn signature_fixture() -> Result<EthereumSignature, VerifyError> {
        let key = signing_key()?;
        let (signature, recovery_id) = key
            .sign_prehash_recoverable(&<[u8; SIGNING_DIGEST_BYTES]>::from(signing_digest()))
            .map_err(|_| VerifyError::InvalidSignature)?;
        let bytes = signature.to_bytes();
        let r = bytes
            .get(..SIGNING_DIGEST_BYTES)
            .and_then(|value| <[u8; 32]>::try_from(value).ok())
            .ok_or(VerifyError::InvalidSignature)?;
        let s = bytes
            .get(SIGNING_DIGEST_BYTES..)
            .and_then(|value| <[u8; 32]>::try_from(value).ok())
            .ok_or(VerifyError::InvalidSignature)?;
        EthereumSignature::try_from_parts_with_y_parity(r, s, recovery_id.to_byte())
    }

    fn expected_address() -> Result<Address, VerifyError> {
        let key = signing_key()?;
        let encoded = key.verifying_key().to_encoded_point(false);
        let source = encoded
            .as_bytes()
            .get(1..)
            .and_then(|bytes| bytes.get(bytes.len().saturating_sub(20)..))
            .and_then(|bytes| <[u8; 20]>::try_from(bytes).ok())
            .ok_or(VerifyError::InvalidSignature)?;
        Ok(Address::from_bytes(source))
    }

    #[test]
    fn recovers_known_ethereum_vector() {
        let message = [
            0xe9, 0x80, 0x85, 0x04, 0xe3, 0xb2, 0x92, 0x00, 0x83, 0x1e, 0x84, 0x80, 0x94, 0xf0,
            0x10, 0x9f, 0xc8, 0xdf, 0x28, 0x30, 0x27, 0xb6, 0x28, 0x5c, 0xc8, 0x89, 0xf5, 0xaa,
            0x62, 0x4e, 0xac, 0x1f, 0x55, 0x84, 0x3b, 0x9a, 0xca, 0x00, 0x80, 0x01, 0x80, 0x80,
        ];
        let signing_digest = eth_valkyoth_hash::hash_one(RealKeccak::default(), &message);
        let signature = EthereumSignature::from_parts(
            [
                0xc9, 0xcf, 0x86, 0x33, 0x3b, 0xcb, 0x06, 0x5d, 0x14, 0x00, 0x32, 0xec, 0xaa, 0xb5,
                0xd9, 0x28, 0x1b, 0xde, 0x80, 0xf2, 0x1b, 0x96, 0x87, 0xb3, 0xe9, 0x41, 0x61, 0xde,
                0x42, 0xd5, 0x18, 0x95,
            ],
            [
                0x72, 0x7a, 0x10, 0x8a, 0x0b, 0x8d, 0x10, 0x14, 0x65, 0x41, 0x40, 0x33, 0xc3, 0xf7,
                0x05, 0xa9, 0xc7, 0xb8, 0x26, 0xe5, 0x96, 0x76, 0x60, 0x46, 0xee, 0x11, 0x83, 0xdb,
                0xc8, 0xae, 0xaa, 0x68,
            ],
            SignatureYParity::Even,
        );
        let expected = Address::from_bytes([
            0x2c, 0x75, 0x36, 0xe3, 0x60, 0x5d, 0x9c, 0x16, 0xa7, 0xa3, 0xd7, 0xb1, 0x89, 0x8e,
            0x52, 0x93, 0x96, 0xa6, 0x5c, 0x23,
        ]);

        assert_eq!(
            recover_sender_from_digest_with_backend(
                signing_digest,
                signature,
                TestSecp256k1Backend,
                RealKeccak::default()
            ),
            Ok(expected)
        );
    }

    #[test]
    fn recovers_sender_from_digest_through_hash_boundary() {
        let signature = signature_fixture();
        assert!(signature.is_ok());
        let expected = expected_address();
        assert!(expected.is_ok());

        if let (Ok(signature), Ok(expected)) = (signature, expected) {
            assert_eq!(
                recover_sender_from_digest_with_backend(
                    signing_digest(),
                    signature,
                    TestSecp256k1Backend,
                    AddressFromPublicKeyHasher::new()
                ),
                Ok(expected)
            );
        }
    }

    #[test]
    fn rejects_non_ethereum_recovery_id() {
        assert_eq!(
            EthereumSignature::try_from_parts_with_y_parity([1_u8; 32], [1_u8; 32], 2),
            Err(VerifyError::InvalidSignature)
        );
    }

    #[test]
    fn rejects_zero_scalars() {
        let signature =
            EthereumSignature::from_parts([0_u8; 32], [1_u8; 32], SignatureYParity::Even);

        assert_eq!(
            recover_sender_from_digest_with_backend(
                signing_digest(),
                signature,
                TestSecp256k1Backend,
                AddressFromPublicKeyHasher::new()
            ),
            Err(VerifyError::InvalidSignature)
        );
    }

    #[test]
    fn rejects_high_s_signatures() {
        let signature = EthereumSignature::from_parts(
            [
                0x20, 0xc0, 0x1a, 0x91, 0x0e, 0xbb, 0x26, 0x10, 0xaf, 0x2d, 0x76, 0x3f, 0xa0, 0x9b,
                0x3b, 0x30, 0x92, 0x3c, 0x8e, 0x40, 0x8b, 0x11, 0xdf, 0x2c, 0x61, 0xad, 0x76, 0xd9,
                0x70, 0xa2, 0xf1, 0xbc,
            ],
            [
                0xee, 0x2f, 0x11, 0xef, 0x8c, 0xb0, 0x0a, 0x49, 0x61, 0x7d, 0x13, 0x57, 0xf4, 0xd5,
                0x56, 0x41, 0x09, 0x0a, 0x48, 0xf2, 0x01, 0xe9, 0xb9, 0x59, 0xc4, 0x8f, 0x6f, 0x6b,
                0xec, 0x6f, 0x93, 0x8f,
            ],
            SignatureYParity::Even,
        );

        assert_eq!(
            recover_sender_from_digest_with_backend(
                signing_digest(),
                signature,
                TestSecp256k1Backend,
                AddressFromPublicKeyHasher::new()
            ),
            Err(VerifyError::InvalidSignature)
        );
    }

    #[test]
    fn parses_signature_bytes() {
        let signature = EthereumSignature::try_from_bytes([1_u8; ETHEREUM_SIGNATURE_BYTES]);

        assert!(signature.is_ok());
    }
}
