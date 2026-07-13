use core::fmt;

use eth_valkyoth_hash::Keccak256;
use eth_valkyoth_primitives::Address;
use eth_valkyoth_protocol::SetCodeAuthorization;

#[cfg(feature = "secp256k1-k256")]
use crate::K256Secp256k1Backend;
use crate::{
    EthereumSignature, RecoverableSecp256k1, SetCodeAuthorizationSigningHash,
    TransactionSigningHashError, recover_sender_from_digest_with_backend,
    set_code_authorization_signing_hash,
};

/// A decoded EIP-7702 authorization tuple signature that recovered correctly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedSetCodeAuthorization {
    authority: Address,
    signing_hash: SetCodeAuthorizationSigningHash,
}

impl ValidatedSetCodeAuthorization {
    /// Creates a validated authorization result from checked components.
    #[must_use]
    pub(crate) const fn new(
        authority: Address,
        signing_hash: SetCodeAuthorizationSigningHash,
    ) -> Self {
        Self {
            authority,
            signing_hash,
        }
    }

    /// Returns the recovered authorizing account.
    #[must_use]
    pub const fn authority(self) -> Address {
        self.authority
    }

    /// Returns the authorization signing hash that was recovered against.
    #[must_use]
    pub const fn signing_hash(self) -> SetCodeAuthorizationSigningHash {
        self.signing_hash
    }
}

/// EIP-7702 authorization tuple signature validation failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetCodeAuthorizationValidationError {
    /// Authorization signing-hash construction failed before recovery.
    SigningHash(TransactionSigningHashError),
    /// Signature scalar, y-parity, or public-key recovery failed.
    InvalidSignature,
    /// The recovered authority does not match the expected account.
    WrongAuthority,
}

impl SetCodeAuthorizationValidationError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::SigningHash(error) => error.code(),
            Self::InvalidSignature => "ETH_SET_CODE_AUTHORIZATION_SIGNATURE_INVALID",
            Self::WrongAuthority => "ETH_SET_CODE_AUTHORIZATION_WRONG_AUTHORITY",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::SigningHash(_) => "set-code authorization signing hash construction failed",
            Self::InvalidSignature => "set-code authorization signature is not accepted",
            Self::WrongAuthority => {
                "set-code authorization signature recovered a different authority"
            }
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> SetCodeAuthorizationValidationErrorCategory {
        match self {
            Self::SigningHash(_) => SetCodeAuthorizationValidationErrorCategory::SigningHash,
            Self::InvalidSignature | Self::WrongAuthority => {
                SetCodeAuthorizationValidationErrorCategory::Signature
            }
        }
    }
}

impl fmt::Display for SetCodeAuthorizationValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SetCodeAuthorizationValidationError {}

/// Stable high-level set-code authorization validation categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetCodeAuthorizationValidationErrorCategory {
    /// Signing preimage or hash construction failure.
    SigningHash,
    /// Signature representation, recovery, or authority-match failure.
    Signature,
}

/// Validates a decoded EIP-7702 authorization tuple signature.
///
/// This validates only the tuple signature domain
/// `keccak256(0x05 || rlp([chain_id, address, nonce]))`. Chain-ID policy,
/// nonce/account-state checks, delegation indicator checks, and empty-list
/// rejection are the v0.24.2 transaction-validity gate.
#[cfg(feature = "secp256k1-k256")]
pub fn validate_set_code_authorization_signature<H1, H2>(
    authorization: SetCodeAuthorization,
    expected_authority: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    address_hasher: H2,
) -> Result<ValidatedSetCodeAuthorization, SetCodeAuthorizationValidationError>
where
    H1: Keccak256,
    H2: Keccak256,
{
    validate_set_code_authorization_signature_with_backend(
        authorization,
        expected_authority,
        scratch,
        signing_hasher,
        K256Secp256k1Backend,
        address_hasher,
    )
}

/// Validates a decoded EIP-7702 authorization tuple signature through a backend.
pub fn validate_set_code_authorization_signature_with_backend<B, H1, H2>(
    authorization: SetCodeAuthorization,
    expected_authority: Option<Address>,
    scratch: &mut [u8],
    signing_hasher: H1,
    secp256k1_backend: B,
    address_hasher: H2,
) -> Result<ValidatedSetCodeAuthorization, SetCodeAuthorizationValidationError>
where
    B: RecoverableSecp256k1,
    H1: Keccak256,
    H2: Keccak256,
{
    let signing_hash = set_code_authorization_signing_hash(authorization, scratch, signing_hasher)
        .map_err(SetCodeAuthorizationValidationError::SigningHash)?;
    let signature =
        EthereumSignature::from_parts(authorization.r, authorization.s, authorization.y_parity);
    let authority = recover_sender_from_digest_with_backend(
        signing_hash.to_b256(),
        signature,
        secp256k1_backend,
        address_hasher,
    )
    .map_err(|_| SetCodeAuthorizationValidationError::InvalidSignature)?;
    if let Some(expected) = expected_authority
        && authority != expected
    {
        return Err(SetCodeAuthorizationValidationError::WrongAuthority);
    }
    Ok(ValidatedSetCodeAuthorization::new(authority, signing_hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_crypto::{RealKeccak, TestSecp256k1Backend};
    use eth_valkyoth_primitives::Nonce;
    use eth_valkyoth_protocol::{
        SET_CODE_AUTHORIZATION_MAGIC, SetCodeAuthorizationChainId, SignatureYParity,
        encode_set_code_authorization_signing_preimage,
    };
    use k256::ecdsa::SigningKey;

    fn signing_key() -> Result<SigningKey, SetCodeAuthorizationValidationError> {
        SigningKey::from_bytes((&fixture_bytes(b"set-code authorization test key")).into())
            .map_err(|_| SetCodeAuthorizationValidationError::InvalidSignature)
    }

    fn unsigned_authorization() -> SetCodeAuthorization {
        let mut chain_id = [0_u8; 32];
        if let Some(last) = chain_id.last_mut() {
            *last = u8::try_from("chain-one".len()).unwrap_or_default();
        }
        SetCodeAuthorization {
            chain_id: SetCodeAuthorizationChainId::from_be_bytes(chain_id),
            address: Address::from_bytes(address_bytes(b"delegate")),
            nonce: Nonce::new(u64::try_from("nonce".len()).unwrap_or_default()),
            y_parity: SignatureYParity::Even,
            r: [0_u8; 32],
            s: [0_u8; 32],
        }
    }

    fn signed_authorization() -> Result<SetCodeAuthorization, SetCodeAuthorizationValidationError> {
        let authorization = unsigned_authorization();
        let mut scratch = [0_u8; 128];
        let signing_hash =
            set_code_authorization_signing_hash(authorization, &mut scratch, RealKeccak::default())
                .map_err(SetCodeAuthorizationValidationError::SigningHash)?;
        let key = signing_key()?;
        let (signature, recovery_id) =
            key.sign_prehash_recoverable(&<[u8; 32]>::from(signing_hash.to_b256()));
        let bytes = signature.to_bytes();
        let r = bytes
            .get(..32)
            .and_then(|value| <[u8; 32]>::try_from(value).ok())
            .ok_or(SetCodeAuthorizationValidationError::InvalidSignature)?;
        let s = bytes
            .get(32..)
            .and_then(|value| <[u8; 32]>::try_from(value).ok())
            .ok_or(SetCodeAuthorizationValidationError::InvalidSignature)?;
        let y_parity = SignatureYParity::try_new(u64::from(recovery_id.to_byte()))
            .map_err(|_| SetCodeAuthorizationValidationError::InvalidSignature)?;
        Ok(SetCodeAuthorization {
            y_parity,
            r,
            s,
            ..authorization
        })
    }

    fn expected_authority() -> Result<Address, SetCodeAuthorizationValidationError> {
        let key = signing_key()?;
        let encoded = key.verifying_key().to_sec1_point(false);
        let public_key = encoded
            .as_bytes()
            .get(1..)
            .ok_or(SetCodeAuthorizationValidationError::InvalidSignature)?;
        let digest = eth_valkyoth_hash::hash_one(RealKeccak::default(), public_key);
        let bytes = <[u8; 32]>::from(digest);
        let address = bytes
            .get(12..)
            .and_then(|value| <[u8; 20]>::try_from(value).ok())
            .ok_or(SetCodeAuthorizationValidationError::InvalidSignature)?;
        Ok(Address::from_bytes(address))
    }

    fn fixture_bytes(label: &[u8]) -> [u8; 32] {
        let mut bytes = [0_u8; 32];
        fill_fixture_bytes(&mut bytes, label);
        bytes
    }

    fn address_bytes(label: &[u8]) -> [u8; 20] {
        let mut bytes = [0_u8; 20];
        fill_fixture_bytes(&mut bytes, label);
        bytes
    }

    fn fill_fixture_bytes(bytes: &mut [u8], label: &[u8]) {
        if label.is_empty() {
            return;
        }
        for (index, byte) in bytes.iter_mut().enumerate() {
            let Some(label_index) = index.checked_rem(label.len()) else {
                return;
            };
            let label_byte = label.get(label_index).copied().unwrap_or_default();
            *byte = label_byte.wrapping_add(u8::try_from(index).unwrap_or_default());
        }
    }

    #[test]
    fn validates_authorization_signature_in_authorization_domain() {
        let authorization = signed_authorization();
        let expected = expected_authority();
        assert!(authorization.is_ok());
        assert!(expected.is_ok());

        if let (Ok(authorization), Ok(expected)) = (authorization, expected) {
            let mut scratch = [0_u8; 128];
            let result = validate_set_code_authorization_signature_with_backend(
                authorization,
                Some(expected),
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            );
            assert_eq!(
                result.map(ValidatedSetCodeAuthorization::authority),
                Ok(expected)
            );
        }
    }

    #[test]
    fn authorization_preimage_uses_magic_domain() {
        let authorization = unsigned_authorization();
        let mut output = [0_u8; 128];
        let written = encode_set_code_authorization_signing_preimage(authorization, &mut output);
        assert!(written.is_ok());
        if let Ok(written) = written {
            assert_eq!(output.first(), Some(&SET_CODE_AUTHORIZATION_MAGIC));
            assert_ne!(output.get(1..written), Some(&[][..]));
        }
    }

    #[test]
    fn rejects_wrong_authority_and_short_scratch() {
        let authorization = signed_authorization();
        assert!(authorization.is_ok());
        if let Ok(authorization) = authorization {
            let mut scratch = [0_u8; 128];
            assert_eq!(
                validate_set_code_authorization_signature_with_backend(
                    authorization,
                    Some(Address::from_bytes(address_bytes(b"wrong"))),
                    &mut scratch,
                    RealKeccak::default(),
                    TestSecp256k1Backend,
                    RealKeccak::default(),
                ),
                Err(SetCodeAuthorizationValidationError::WrongAuthority)
            );

            let mut short = [0_u8; 8];
            assert!(matches!(
                validate_set_code_authorization_signature_with_backend(
                    authorization,
                    None,
                    &mut short,
                    RealKeccak::default(),
                    TestSecp256k1Backend,
                    RealKeccak::default(),
                ),
                Err(SetCodeAuthorizationValidationError::SigningHash(_))
            ));
        }
    }
}
