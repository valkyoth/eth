use eth_valkyoth_primitives::{Address, B256, Nonce, Wei};

use super::{AccountTrieRoot, StorageSlotKey, StorageTrieRoot};

/// Canonically decoded Ethereum state account.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EthereumAccount {
    nonce: Nonce,
    balance: Wei,
    storage_root: StorageTrieRoot,
    code_hash: B256,
}

impl EthereumAccount {
    pub(crate) const FIELD_COUNT: usize = 4;

    pub(crate) const fn new(
        nonce: Nonce,
        balance: Wei,
        storage_root: StorageTrieRoot,
        code_hash: B256,
    ) -> Self {
        Self {
            nonce,
            balance,
            storage_root,
            code_hash,
        }
    }

    /// Returns the account transaction nonce.
    #[must_use]
    pub const fn nonce(self) -> Nonce {
        self.nonce
    }

    /// Returns the account balance.
    #[must_use]
    pub const fn balance(self) -> Wei {
        self.balance
    }

    /// Returns the storage trie root embedded in the account value.
    #[must_use]
    pub const fn storage_root(self) -> StorageTrieRoot {
        self.storage_root
    }

    /// Returns the account bytecode hash.
    #[must_use]
    pub const fn code_hash(self) -> B256 {
        self.code_hash
    }
}

/// Cryptographic authority for one present or absent state account.
///
/// Fields and constructors are private. Values can only be obtained by
/// verifying a proof against a caller-trusted account trie root.
#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedAccount {
    address: Address,
    root: AccountTrieRoot,
    account: Option<EthereumAccount>,
}

impl VerifiedAccount {
    pub(super) const fn present(
        address: Address,
        root: AccountTrieRoot,
        account: EthereumAccount,
    ) -> Self {
        Self {
            address,
            root,
            account: Some(account),
        }
    }

    pub(super) const fn absent(address: Address, root: AccountTrieRoot) -> Self {
        Self {
            address,
            root,
            account: None,
        }
    }

    /// Returns the proven address.
    #[must_use]
    pub const fn address(&self) -> Address {
        self.address
    }

    /// Returns the trusted account trie root used by the proof.
    #[must_use]
    pub const fn root(&self) -> AccountTrieRoot {
        self.root
    }

    /// Returns the decoded account, or `None` for proven canonical absence.
    #[must_use]
    pub const fn account(&self) -> Option<EthereumAccount> {
        self.account
    }

    /// Returns true when the address was proven absent.
    #[must_use]
    pub const fn is_absent(&self) -> bool {
        self.account.is_none()
    }

    /// Returns the only storage root authorized by this account proof.
    #[must_use]
    pub const fn storage_root(&self) -> StorageTrieRoot {
        match self.account {
            Some(account) => account.storage_root(),
            None => StorageTrieRoot::EMPTY,
        }
    }
}

/// Verified storage value derived from a [`VerifiedAccount`] capability.
#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedStorageValue {
    slot: StorageSlotKey,
    root: StorageTrieRoot,
    value: Wei,
    present: bool,
}

impl VerifiedStorageValue {
    pub(super) const fn new(
        slot: StorageSlotKey,
        root: StorageTrieRoot,
        value: Wei,
        present: bool,
    ) -> Self {
        Self {
            slot,
            root,
            value,
            present,
        }
    }

    /// Returns the proven storage slot.
    #[must_use]
    pub const fn slot(&self) -> StorageSlotKey {
        self.slot
    }

    /// Returns the account-derived storage trie root.
    #[must_use]
    pub const fn root(&self) -> StorageTrieRoot {
        self.root
    }

    /// Returns the storage value. Proven absence maps to canonical zero.
    #[must_use]
    pub const fn value(&self) -> Wei {
        self.value
    }

    /// Returns true only when a nonzero value was present in the storage trie.
    #[must_use]
    pub const fn is_present(&self) -> bool {
        self.present
    }
}
