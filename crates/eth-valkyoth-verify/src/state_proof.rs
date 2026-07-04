use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::{Address, B256};

use crate::mpt_proof::{MptProofRoot, MptProofVerificationError, verify_key_inclusion};

/// Ethereum state trie root hash domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountTrieRoot(B256);

impl AccountTrieRoot {
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
    mut new_hasher: impl FnMut() -> H,
) -> Result<VerifiedAccountInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    let key = hash_one(new_hasher(), &address.to_bytes()).to_bytes();
    verify_key_inclusion(
        root.into(),
        &key,
        encoded_account,
        proof_nodes,
        limits,
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
    mut new_hasher: impl FnMut() -> H,
) -> Result<VerifiedStorageInclusion, MptProofVerificationError>
where
    H: Keccak256,
{
    let key = hash_one(new_hasher(), &slot.to_b256().to_bytes()).to_bytes();
    verify_key_inclusion(
        root.into(),
        &key,
        encoded_storage_value,
        proof_nodes,
        limits,
        new_hasher,
    )?;
    Ok(VerifiedStorageInclusion::new(slot, root))
}

#[cfg(test)]
#[path = "state_proof_tests.rs"]
mod tests;
