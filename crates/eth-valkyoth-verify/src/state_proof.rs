use eth_valkyoth_codec::{DecodeLimits, DecodeSession, RlpInteger, decode_rlp_scalar_in_session};
use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::{Address, B256, Wei};

use crate::mpt_proof::{
    MptProofRoot, MptProofValue, MptProofVerificationError,
    check_preflighted_key_inclusion_capacity, compatibility_session, plan_preflighted_key_value,
    preflight_proof, proof_resource_error, verify_preflighted_key_inclusion,
    verify_preflighted_key_value,
};

mod account;
mod error;
mod types;

use account::decode_account;
pub use error::{
    AccountDecodeError, AccountField, StateProofVerificationError,
    StateProofVerificationErrorCategory,
};
pub use types::{EthereumAccount, VerifiedAccount, VerifiedStorageValue};

/// Canonical Ethereum root hash for an empty Merkle Patricia trie.
pub const EMPTY_TRIE_ROOT_BYTES: [u8; 32] = [
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
];

/// Ethereum state trie root hash domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountTrieRoot(B256);

impl AccountTrieRoot {
    /// Canonical root of an empty Ethereum account trie.
    pub const EMPTY: Self = Self(B256::from_bytes(EMPTY_TRIE_ROOT_BYTES));

    /// Creates an account trie root from raw hash bytes.
    #[must_use]
    pub const fn from_b256(value: B256) -> Self {
        Self(value)
    }

    /// Returns the raw root hash.
    #[must_use]
    pub const fn to_b256(self) -> B256 {
        self.0
    }
}

impl From<AccountTrieRoot> for MptProofRoot {
    fn from(value: AccountTrieRoot) -> Self {
        Self::from_b256(value.to_b256())
    }
}

/// Ethereum storage trie root hash domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageTrieRoot(B256);

impl StorageTrieRoot {
    /// Canonical root of an empty Ethereum storage trie.
    pub const EMPTY: Self = Self(B256::from_bytes(EMPTY_TRIE_ROOT_BYTES));

    /// Creates a storage trie root from raw hash bytes.
    #[must_use]
    pub const fn from_b256(value: B256) -> Self {
        Self(value)
    }

    /// Returns the raw root hash.
    #[must_use]
    pub const fn to_b256(self) -> B256 {
        self.0
    }
}

impl From<StorageTrieRoot> for MptProofRoot {
    fn from(value: StorageTrieRoot) -> Self {
        Self::from_b256(value.to_b256())
    }
}

/// Ethereum storage slot key before trie-key hashing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageSlotKey(B256);

impl StorageSlotKey {
    /// Creates a storage slot key from its canonical 32-byte representation.
    #[must_use]
    pub const fn from_b256(value: B256) -> Self {
        Self(value)
    }

    /// Returns the canonical 32-byte storage slot key.
    #[must_use]
    pub const fn to_b256(self) -> B256 {
        self.0
    }
}

/// Successful account proof verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedAccountInclusion {
    address: Address,
    root: AccountTrieRoot,
}

impl VerifiedAccountInclusion {
    const fn new(address: Address, root: AccountTrieRoot) -> Self {
        Self { address, root }
    }

    /// Returns the account address proven in the state trie.
    #[must_use]
    pub const fn address(self) -> Address {
        self.address
    }

    /// Returns the account trie root used for verification.
    #[must_use]
    pub const fn root(self) -> AccountTrieRoot {
        self.root
    }
}

/// Successful storage proof verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedStorageInclusion {
    slot: StorageSlotKey,
    root: StorageTrieRoot,
}

impl VerifiedStorageInclusion {
    const fn new(slot: StorageSlotKey, root: StorageTrieRoot) -> Self {
        Self { slot, root }
    }

    /// Returns the storage slot proven in the storage trie.
    #[must_use]
    pub const fn slot(self) -> StorageSlotKey {
        self.slot
    }

    /// Returns the storage trie root used for verification.
    #[must_use]
    pub const fn root(self) -> StorageTrieRoot {
        self.root
    }
}

/// Verifies a canonical account inclusion or absence proof under `root`.
///
/// The proof determines the account bytes; callers do not supply a parallel
/// account value that could drift from the authenticated trie value. A present
/// value is decoded as `[nonce, balance, storageRoot, codeHash]`. The returned
/// capability is the only authority accepted by composed storage verification.
pub fn verify_account_proof<H>(
    root: AccountTrieRoot,
    address: Address,
    proof_nodes: &[&[u8]],
    limits: DecodeLimits,
    new_hasher: impl FnMut() -> H,
) -> Result<VerifiedAccount, StateProofVerificationError>
where
    H: Keccak256,
{
    let mut session = compatibility_session(limits)?;
    verify_account_proof_in_session(root, address, proof_nodes, &mut session, new_hasher)
}

/// Verifies a canonical account proof through one shared work session.
pub fn verify_account_proof_in_session<H>(
    root: AccountTrieRoot,
    address: Address,
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
    mut new_hasher: impl FnMut() -> H,
) -> Result<VerifiedAccount, StateProofVerificationError>
where
    H: Keccak256,
{
    if proof_nodes.is_empty() {
        return if root == AccountTrieRoot::EMPTY {
            Ok(VerifiedAccount::absent(address, root))
        } else {
            Err(MptProofVerificationError::MissingProofNode.into())
        };
    }

    let address_bytes = address.to_bytes();
    preflight_proof(proof_nodes, &[], 1, address_bytes.len(), session)?;
    session
        .account_hashes(1, address_bytes.len())
        .map_err(proof_resource_error)?;
    let key = hash_one(new_hasher(), &address_bytes).to_bytes();
    let planned = plan_preflighted_key_value(&key, proof_nodes, session)?;
    let account = match planned.value {
        MptProofValue::Present(encoded) => {
            Some(decode_account(encoded, session).map_err(StateProofVerificationError::Account)?)
        }
        MptProofValue::Absent => None,
    };
    session
        .check_remaining_capacity(planned.charges)
        .map_err(proof_resource_error)?;
    let verified =
        verify_preflighted_key_value(root.into(), &key, proof_nodes, session, new_hasher)?;
    require_same_outcome(planned.value, verified)?;

    Ok(match account {
        Some(account) => VerifiedAccount::present(address, root, account),
        None => VerifiedAccount::absent(address, root),
    })
}

/// Verifies one storage slot using the root authenticated by `account`.
///
/// A missing trie key is canonical storage zero. An explicitly stored zero is
/// rejected because Ethereum state tries omit zero-valued storage slots.
pub fn verify_account_storage<H>(
    account: &VerifiedAccount,
    slot: StorageSlotKey,
    proof_nodes: &[&[u8]],
    limits: DecodeLimits,
    new_hasher: impl FnMut() -> H,
) -> Result<VerifiedStorageValue, StateProofVerificationError>
where
    H: Keccak256,
{
    let mut session = compatibility_session(limits)?;
    verify_account_storage_in_session(account, slot, proof_nodes, &mut session, new_hasher)
}

/// Verifies account-bound storage through one shared work session.
pub fn verify_account_storage_in_session<H>(
    account: &VerifiedAccount,
    slot: StorageSlotKey,
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
    mut new_hasher: impl FnMut() -> H,
) -> Result<VerifiedStorageValue, StateProofVerificationError>
where
    H: Keccak256,
{
    let root = account.storage_root();
    if proof_nodes.is_empty() {
        return if root == StorageTrieRoot::EMPTY {
            Ok(VerifiedStorageValue::new(slot, root, Wei::ZERO, false))
        } else {
            Err(MptProofVerificationError::MissingProofNode.into())
        };
    }

    let slot_bytes = slot.to_b256().to_bytes();
    preflight_proof(proof_nodes, &[], 1, slot_bytes.len(), session)?;
    session
        .account_hashes(1, slot_bytes.len())
        .map_err(proof_resource_error)?;
    let key = hash_one(new_hasher(), &slot_bytes).to_bytes();
    let planned = plan_preflighted_key_value(&key, proof_nodes, session)?;
    let value = match planned.value {
        MptProofValue::Present(encoded) => {
            let scalar = decode_rlp_scalar_in_session(encoded, session)
                .map_err(StateProofVerificationError::StorageValue)?;
            let bytes = RlpInteger::try_from_scalar(scalar)
                .and_then(RlpInteger::to_be_bytes32)
                .map_err(StateProofVerificationError::StorageValue)?;
            let value = Wei::from_be_bytes(bytes);
            if value == Wei::ZERO {
                return Err(StateProofVerificationError::ExplicitZeroStorageValue);
            }
            Some(value)
        }
        MptProofValue::Absent => None,
    };
    session
        .check_remaining_capacity(planned.charges)
        .map_err(proof_resource_error)?;
    let verified =
        verify_preflighted_key_value(root.into(), &key, proof_nodes, session, new_hasher)?;
    require_same_outcome(planned.value, verified)?;

    Ok(VerifiedStorageValue::new(
        slot,
        root,
        value.unwrap_or(Wei::ZERO),
        value.is_some(),
    ))
}

fn require_same_outcome(
    planned: MptProofValue<'_>,
    verified: MptProofValue<'_>,
) -> Result<(), StateProofVerificationError> {
    if planned == verified {
        Ok(())
    } else {
        Err(StateProofVerificationError::InconsistentProofTraversal)
    }
}

/// Verifies that `encoded_account` is included for `address` under `root`.
///
/// The trie key is `keccak256(address)`, matching Ethereum state tries. The
/// value is compared byte-for-byte with `encoded_account`; this function does
/// not decode account nonce, balance, storage root, or code hash fields.
pub fn verify_account_inclusion<H>(
    root: AccountTrieRoot,
    address: Address,
    encoded_account: &[u8],
    proof_nodes: &[&[u8]],
    limits: DecodeLimits,
    new_hasher: impl FnMut() -> H,
) -> Result<VerifiedAccountInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    let mut session = compatibility_session(limits)?;
    verify_account_inclusion_in_session(
        root,
        address,
        encoded_account,
        proof_nodes,
        &mut session,
        new_hasher,
    )
}

/// Verifies account inclusion through one shared decode/work session.
pub fn verify_account_inclusion_in_session<H>(
    root: AccountTrieRoot,
    address: Address,
    encoded_account: &[u8],
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
    mut new_hasher: impl FnMut() -> H,
) -> Result<VerifiedAccountInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    let address_bytes = address.to_bytes();
    preflight_proof(
        proof_nodes,
        encoded_account,
        1,
        address_bytes.len(),
        session,
    )?;
    session
        .account_hashes(1, address_bytes.len())
        .map_err(proof_resource_error)?;
    let key = hash_one(new_hasher(), &address_bytes).to_bytes();
    check_preflighted_key_inclusion_capacity(&key, encoded_account, proof_nodes, session)?;
    verify_preflighted_key_inclusion(
        root.into(),
        &key,
        encoded_account,
        proof_nodes,
        session,
        new_hasher,
    )?;
    Ok(VerifiedAccountInclusion::new(address, root))
}

/// Verifies that `encoded_storage_value` is included for `slot` under `root`.
///
/// The trie key is `keccak256(slot_key)`, matching Ethereum account storage
/// tries. The value is compared byte-for-byte with `encoded_storage_value`;
/// this function does not interpret the RLP scalar or prove account ownership
/// of the storage root.
///
/// Callers must bind `root` to a verified account proof or another trusted
/// account-state source before treating this as a full `eth_getProof`
/// verification result.
pub fn verify_storage_inclusion<H>(
    root: StorageTrieRoot,
    slot: StorageSlotKey,
    encoded_storage_value: &[u8],
    proof_nodes: &[&[u8]],
    limits: DecodeLimits,
    new_hasher: impl FnMut() -> H,
) -> Result<VerifiedStorageInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    let mut session = compatibility_session(limits)?;
    verify_storage_inclusion_in_session(
        root,
        slot,
        encoded_storage_value,
        proof_nodes,
        &mut session,
        new_hasher,
    )
}

/// Verifies storage inclusion through one shared decode/work session.
pub fn verify_storage_inclusion_in_session<H>(
    root: StorageTrieRoot,
    slot: StorageSlotKey,
    encoded_storage_value: &[u8],
    proof_nodes: &[&[u8]],
    session: &mut DecodeSession,
    mut new_hasher: impl FnMut() -> H,
) -> Result<VerifiedStorageInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    let slot_bytes = slot.to_b256().to_bytes();
    preflight_proof(
        proof_nodes,
        encoded_storage_value,
        1,
        slot_bytes.len(),
        session,
    )?;
    session
        .account_hashes(1, slot_bytes.len())
        .map_err(proof_resource_error)?;
    let key = hash_one(new_hasher(), &slot_bytes).to_bytes();
    check_preflighted_key_inclusion_capacity(&key, encoded_storage_value, proof_nodes, session)?;
    verify_preflighted_key_inclusion(
        root.into(),
        &key,
        encoded_storage_value,
        proof_nodes,
        session,
        new_hasher,
    )?;
    Ok(VerifiedStorageInclusion::new(slot, root))
}

#[cfg(test)]
#[path = "state_proof_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "state_proof_composed_tests.rs"]
mod composed_tests;
