use eth_valkyoth_hash::{Keccak256, hash_chunks};
use eth_valkyoth_primitives::{Address, B256, ChainId};

#[cfg(feature = "secp256k1-k256")]
use crate::K256Secp256k1Backend;
use crate::{
    EthereumSignature, RecoverableSecp256k1, VerifyError, recover_sender_from_digest_with_backend,
};

/// EIP-191 versioned prefix used by EIP-712 structured-data signing.
pub const EIP712_SIGNING_PREFIX: [u8; 2] = [0x19, 0x01];

/// EIP-712 domain fields needed by this crate's safety checks.
///
/// EIP-712 allows domains to omit fields that do not make sense for a
/// protocol. This verification boundary is stricter for signing paths that
/// need replay protection: callers must provide both `chainId` and
/// `verifyingContract` before sender recovery is considered domain-safe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eip712Domain {
    chain_id: Option<ChainId>,
    verifying_contract: Option<Address>,
}

/// Expected EIP-712 domain context for a verification boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eip712DomainExpectation {
    chain_id: ChainId,
    verifying_contract: Address,
}

impl Eip712DomainExpectation {
    /// Creates an expected EIP-712 domain context.
    #[must_use]
    pub const fn new(chain_id: ChainId, verifying_contract: Address) -> Self {
        Self {
            chain_id,
            verifying_contract,
        }
    }

    /// Returns the expected EIP-712 `chainId`.
    #[must_use]
    pub const fn chain_id(self) -> ChainId {
        self.chain_id
    }

    /// Returns the expected EIP-712 `verifyingContract`.
    #[must_use]
    pub const fn verifying_contract(self) -> Address {
        self.verifying_contract
    }
}

impl Eip712Domain {
    /// Creates a domain view from optional safety-critical fields.
    #[must_use]
    pub const fn new(chain_id: Option<ChainId>, verifying_contract: Option<Address>) -> Self {
        Self {
            chain_id,
            verifying_contract,
        }
    }

    /// Creates a complete replay-safe domain view.
    #[must_use]
    pub const fn complete(chain_id: ChainId, verifying_contract: Address) -> Self {
        Self::new(Some(chain_id), Some(verifying_contract))
    }

    /// Returns the EIP-712 `chainId` field, if present.
    #[must_use]
    pub const fn chain_id(self) -> Option<ChainId> {
        self.chain_id
    }

    /// Returns the EIP-712 `verifyingContract` field, if present.
    #[must_use]
    pub const fn verifying_contract(self) -> Option<Address> {
        self.verifying_contract
    }

    /// Returns true when both replay-critical EIP-712 fields are present.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.chain_id.is_some() && self.verifying_contract.is_some()
    }
}

/// Requires an EIP-712 domain to match the expected chain and verifying contract.
///
/// This is a domain-safety gate only. It does not prove that
/// `domain_separator` was correctly computed from this domain, and it does not
/// encode arbitrary typed data. Callers that build the separator must still use
/// an EIP-712 conformant encoder.
pub fn require_eip712_domain(
    expected_chain: ChainId,
    expected_verifying_contract: Address,
    domain: Eip712Domain,
) -> Result<(), VerifyError> {
    let actual_chain = domain.chain_id().ok_or(VerifyError::MissingEip712ChainId)?;
    if actual_chain != expected_chain {
        return Err(VerifyError::WrongChain);
    }

    let actual_contract = domain
        .verifying_contract()
        .ok_or(VerifyError::MissingEip712VerifyingContract)?;
    if actual_contract != expected_verifying_contract {
        return Err(VerifyError::WrongVerifyingContract);
    }

    Ok(())
}

/// Builds the EIP-712 signing digest from a domain separator and message hash.
///
/// The digest is `keccak256("\x19\x01" || domainSeparator ||
/// hashStruct(message))`. This helper assumes the caller has already computed
/// the domain separator and message hash with an EIP-712 conformant encoder.
#[must_use]
pub fn eip712_signing_digest<H>(domain_separator: B256, message_hash: B256, hasher: H) -> B256
where
    H: Keccak256,
{
    let domain_bytes = domain_separator.to_bytes();
    let message_bytes = message_hash.to_bytes();
    hash_chunks(
        hasher,
        [
            EIP712_SIGNING_PREFIX.as_slice(),
            domain_bytes.as_slice(),
            message_bytes.as_slice(),
        ],
    )
}

/// Recovers an EIP-712 sender after checking the expected structured-data domain.
///
/// `digest_hasher` hashes the EIP-712 signing preimage. `address_hasher` hashes
/// the recovered public key payload into an Ethereum address. Both hashers must
/// compute Ethereum Keccak-256, not FIPS SHA3-256.
///
/// This function does not prove `domain_separator` was derived from `domain`.
/// Callers must compute `domain_separator` from the same `chainId` and
/// `verifyingContract` values passed here, using a conformant EIP-712 encoder;
/// otherwise the checked domain and signed digest can silently diverge.
#[cfg(feature = "secp256k1-k256")]
pub fn recover_eip712_sender<DH, AH>(
    expected_domain: Eip712DomainExpectation,
    domain: Eip712Domain,
    domain_separator: B256,
    message_hash: B256,
    signature: EthereumSignature,
    digest_hasher: DH,
    address_hasher: AH,
) -> Result<Address, VerifyError>
where
    DH: Keccak256,
    AH: Keccak256,
{
    recover_eip712_sender_with_backend(
        expected_domain,
        domain,
        domain_separator,
        message_hash,
        signature,
        digest_hasher,
        K256Secp256k1Backend,
        address_hasher,
    )
}

/// Recovers an EIP-712 sender through a caller-provided secp256k1 backend.
#[allow(clippy::too_many_arguments)]
pub fn recover_eip712_sender_with_backend<B, DH, AH>(
    expected_domain: Eip712DomainExpectation,
    domain: Eip712Domain,
    domain_separator: B256,
    message_hash: B256,
    signature: EthereumSignature,
    digest_hasher: DH,
    secp256k1_backend: B,
    address_hasher: AH,
) -> Result<Address, VerifyError>
where
    B: RecoverableSecp256k1,
    DH: Keccak256,
    AH: Keccak256,
{
    require_eip712_domain(
        expected_domain.chain_id(),
        expected_domain.verifying_contract(),
        domain,
    )?;
    let digest = eip712_signing_digest(domain_separator, message_hash, digest_hasher);
    recover_sender_from_digest_with_backend(digest, signature, secp256k1_backend, address_hasher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_crypto::{RealKeccak, TestSecp256k1Backend};
    use k256::ecdsa::SigningKey;

    const TEST_PRIVATE_KEY: [u8; 32] = [
        0x4c, 0x08, 0x83, 0xa6, 0x91, 0x02, 0x93, 0x7d, 0x62, 0x31, 0x47, 0x1b, 0x5d, 0xbb, 0x62,
        0x04, 0xfe, 0x51, 0x29, 0x61, 0x70, 0x82, 0x79, 0x2a, 0xe4, 0x68, 0xd0, 0x1a, 0x3f, 0x36,
        0x23, 0x18,
    ];
    const TEST_PRIVATE_KEY_ADDRESS: Address = Address::from_bytes([
        0x2c, 0x75, 0x36, 0xe3, 0x60, 0x5d, 0x9c, 0x16, 0xa7, 0xa3, 0xd7, 0xb1, 0x89, 0x8e, 0x52,
        0x93, 0x96, 0xa6, 0x5c, 0x23,
    ]);

    use eth_valkyoth_protocol::SignatureYParity;

    struct Eip712TranscriptHasher {
        state: u8,
        valid: bool,
    }

    struct FixedHasher {
        digest: B256,
    }

    impl Eip712TranscriptHasher {
        const fn new() -> Self {
            Self {
                state: 0,
                valid: true,
            }
        }
    }

    impl Keccak256 for Eip712TranscriptHasher {
        fn update(&mut self, input: &[u8]) {
            let expected = match self.state {
                0 => EIP712_SIGNING_PREFIX.as_slice(),
                1 => &[0x11_u8; 32],
                2 => &[0x22_u8; 32],
                _ => &[],
            };
            self.valid &= input == expected;
            self.state = self.state.saturating_add(1);
        }

        fn finalize(self) -> B256 {
            let mut bytes = [0_u8; 32];
            if self.valid
                && self.state == 3
                && let Some(first) = bytes.first_mut()
            {
                *first = 0x7a;
            }
            B256::from_bytes(bytes)
        }
    }

    impl Keccak256 for FixedHasher {
        fn update(&mut self, _input: &[u8]) {}

        fn finalize(self) -> B256 {
            self.digest
        }
    }

    fn expected_chain() -> ChainId {
        ChainId::new(1)
    }

    fn expected_contract() -> Address {
        Address::from_bytes([0xcc_u8; 20])
    }

    fn expected_domain() -> Eip712DomainExpectation {
        Eip712DomainExpectation::new(expected_chain(), expected_contract())
    }

    fn complete_domain() -> Eip712Domain {
        Eip712Domain::complete(expected_chain(), expected_contract())
    }

    fn invalid_signature() -> EthereumSignature {
        EthereumSignature::from_parts([0_u8; 32], [0_u8; 32], SignatureYParity::Even)
    }

    fn signing_key() -> Result<SigningKey, VerifyError> {
        SigningKey::from_bytes((&TEST_PRIVATE_KEY).into())
            .map_err(|_| VerifyError::InvalidSignature)
    }

    fn sign_digest(digest: B256) -> Result<EthereumSignature, VerifyError> {
        let key = signing_key()?;
        let (signature, recovery_id) = key.sign_prehash_recoverable(&<[u8; 32]>::from(digest));
        let bytes = signature.to_bytes();
        let r = bytes
            .get(..32)
            .and_then(|value| <[u8; 32]>::try_from(value).ok())
            .ok_or(VerifyError::InvalidSignature)?;
        let s = bytes
            .get(32..)
            .and_then(|value| <[u8; 32]>::try_from(value).ok())
            .ok_or(VerifyError::InvalidSignature)?;
        EthereumSignature::try_from_parts_with_y_parity(r, s, recovery_id.to_byte())
    }

    #[test]
    fn complete_domain_exposes_required_fields() {
        let domain = complete_domain();
        let expected = expected_domain();

        assert!(domain.is_complete());
        assert_eq!(domain.chain_id(), Some(expected_chain()));
        assert_eq!(domain.verifying_contract(), Some(expected_contract()));
        assert_eq!(expected.chain_id(), expected_chain());
        assert_eq!(expected.verifying_contract(), expected_contract());
    }

    #[test]
    fn eip712_domain_requires_chain_id() {
        let domain = Eip712Domain::new(None, Some(expected_contract()));

        assert_eq!(
            require_eip712_domain(expected_chain(), expected_contract(), domain),
            Err(VerifyError::MissingEip712ChainId)
        );
    }

    #[test]
    fn eip712_domain_requires_verifying_contract() {
        let domain = Eip712Domain::new(Some(expected_chain()), None);

        assert_eq!(
            require_eip712_domain(expected_chain(), expected_contract(), domain),
            Err(VerifyError::MissingEip712VerifyingContract)
        );
    }

    #[test]
    fn eip712_domain_rejects_wrong_chain() {
        let domain = Eip712Domain::complete(ChainId::new(5), expected_contract());

        assert_eq!(
            require_eip712_domain(expected_chain(), expected_contract(), domain),
            Err(VerifyError::WrongChain)
        );
    }

    #[test]
    fn eip712_domain_rejects_wrong_verifying_contract() {
        let domain = Eip712Domain::complete(expected_chain(), Address::from_bytes([0xdd_u8; 20]));

        assert_eq!(
            require_eip712_domain(expected_chain(), expected_contract(), domain),
            Err(VerifyError::WrongVerifyingContract)
        );
    }

    #[test]
    fn eip712_signing_digest_uses_eip191_prefix_and_32_byte_domains() {
        let digest = eip712_signing_digest(
            B256::from_bytes([0x11_u8; 32]),
            B256::from_bytes([0x22_u8; 32]),
            Eip712TranscriptHasher::new(),
        );
        let mut expected = [0_u8; 32];
        if let Some(first) = expected.first_mut() {
            *first = 0x7a;
        }

        assert_eq!(digest, B256::from_bytes(expected));
    }

    #[test]
    fn recovers_known_eip712_vector() -> Result<(), VerifyError> {
        let domain_separator = B256::from_bytes([0x11_u8; 32]);
        let message_hash = B256::from_bytes([0x22_u8; 32]);
        let digest = eip712_signing_digest(domain_separator, message_hash, RealKeccak::default());
        let signature = sign_digest(digest)?;

        assert_eq!(
            recover_eip712_sender_with_backend(
                expected_domain(),
                complete_domain(),
                domain_separator,
                message_hash,
                signature,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            ),
            Ok(TEST_PRIVATE_KEY_ADDRESS)
        );
        Ok(())
    }

    #[test]
    fn eip712_recovery_checks_domain_before_signature() {
        let domain = Eip712Domain::complete(ChainId::new(5), expected_contract());

        assert_eq!(
            recover_eip712_sender_with_backend(
                expected_domain(),
                domain,
                B256::from_bytes([0x11_u8; 32]),
                B256::from_bytes([0x22_u8; 32]),
                invalid_signature(),
                FixedHasher {
                    digest: B256::from_bytes([0x33_u8; 32])
                },
                TestSecp256k1Backend,
                FixedHasher {
                    digest: B256::from_bytes([0x44_u8; 32])
                },
            ),
            Err(VerifyError::WrongChain)
        );
    }

    #[test]
    fn eip712_recovery_uses_signature_after_complete_domain() {
        assert_eq!(
            recover_eip712_sender_with_backend(
                expected_domain(),
                complete_domain(),
                B256::from_bytes([0x11_u8; 32]),
                B256::from_bytes([0x22_u8; 32]),
                invalid_signature(),
                FixedHasher {
                    digest: B256::from_bytes([0x33_u8; 32])
                },
                TestSecp256k1Backend,
                FixedHasher {
                    digest: B256::from_bytes([0x44_u8; 32])
                },
            ),
            Err(VerifyError::InvalidSignature)
        );
    }
}
