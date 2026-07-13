use core::cmp::Ordering;

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
/// secp256k1 curve order as a big-endian scalar boundary.
pub const SECP256K1_ORDER_BE: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41,
];
/// EIP-2 low-s upper bound, `floor(secp256k1_order / 2)`.
pub const SECP256K1_HALF_ORDER_BE: [u8; 32] = [
    0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x5d, 0x57, 0x6e, 0x73, 0x57, 0xa4, 0x50, 0x1d, 0xdf, 0xe9, 0x2f, 0x46, 0x68, 0x1b, 0x20, 0xa0,
];

/// Recoverable secp256k1 backend boundary for Ethereum sender recovery.
///
/// Implementations must recover the uncompressed public key payload for the
/// exact `(digest, signature)` pair. The library enforces Ethereum's `r`/`s`
/// scalar policy and the EIP-2 low-s rule before calling the backend. Backends
/// may revalidate defensively, and backends that hold private implementation
/// state should document and test their state-clearing behavior for
/// key-adjacent hashing paths.
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

    /// Checks Ethereum's secp256k1 scalar policy independent of any backend.
    ///
    /// `r` and `s` must be nonzero scalars smaller than the secp256k1 order,
    /// and `s` must satisfy the EIP-2 low-s bound. This is enforced before any
    /// caller-provided backend is invoked so malleability protection is not
    /// delegated to backend prose.
    pub fn validate_scalar_policy(self) -> Result<(), VerifyError> {
        if !scalar_is_nonzero(self.r) || cmp_be(self.r, SECP256K1_ORDER_BE) != Ordering::Less {
            return Err(VerifyError::InvalidSignature);
        }
        if !scalar_is_nonzero(self.s)
            || cmp_be(self.s, SECP256K1_HALF_ORDER_BE) == Ordering::Greater
        {
            return Err(VerifyError::InvalidSignature);
        }
        Ok(())
    }

    #[cfg(feature = "secp256k1-k256")]
    fn secp256k1_signature(self) -> Result<Secp256k1Signature, VerifyError> {
        let signature = Secp256k1Signature::from_scalars(self.r, self.s)
            .map_err(|_| VerifyError::InvalidSignature)?;
        if signature.normalize_s() != signature {
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
    signature.validate_scalar_policy()?;
    let public_key = backend.recover_uncompressed_public_key(signing_digest, signature)?;
    address_from_uncompressed_public_key(public_key, hasher)
}

fn scalar_is_nonzero(value: [u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn cmp_be(left: [u8; 32], right: [u8; 32]) -> Ordering {
    let mut ordering = Ordering::Equal;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        if ordering == Ordering::Equal {
            ordering = left_byte.cmp(right_byte);
        }
    }
    ordering
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
    let encoded = verifying_key.to_sec1_point(false);
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
#[path = "sender_tests.rs"]
mod tests;
